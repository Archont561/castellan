//! The socket server: accept loop, per-connection handshake, and the
//! registry the panel reads. Unix only — on Windows the crate exports a
//! stub with the same shape ([`crate::server_stub`]) until the
//! named-pipe listener lands with a Windows CI lane.
//!
//! Threading is deliberately plain: one accept thread, and per connection
//! one reader (which owns the handshake and the request loop) plus one
//! writer (fed by a channel, so responses and broadcast events can never
//! interleave mid-frame). A browser connection is a handful of messages a
//! minute; an async runtime would buy nothing here and would cost the
//! std-only discipline of `castellan-ipc`.

use std::collections::BTreeMap;
use std::io::{ErrorKind, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use castellan_ipc::{MAX_MESSAGE_BYTES, encode_frame};
use castellan_protocol::{
    AssociationClaim, ClientMessage, ConnectionInfo, ConnectionState, Event, FaceKind, Hello,
    HostMessage, ManifestProblem, PanelSnapshot, PendingKey, RememberedKey, RpcRequest,
    RpcResponse,
};

use crate::association::{EnrollOutcome, WaitOutcome};
use crate::same_user;

/// What answers RPC requests: the shells close over their dispatcher.
pub type RequestHandler = Arc<dyn Fn(RpcRequest) -> RpcResponse + Send + Sync>;

/// What builds the app's hello on demand: version, capabilities, and the
/// vault status *at handshake time* — later changes arrive as events.
pub type HelloSource = Arc<dyn Fn() -> Hello + Send + Sync>;

/// A change listener: called (from any thread) whenever the panel's data
/// could have changed.
pub type ChangeSink = Arc<dyn Fn() + Send + Sync>;

/// How long a connection may sit without saying hello.
const HELLO_TIMEOUT: Duration = Duration::from_secs(10);

/// How long an enrollment waits for the user before its prompt expires.
const APPROVAL_TIMEOUT: Duration = Duration::from_secs(300);

/// How long a single write to a browser may block before the connection
/// is considered wedged.
const WRITE_TIMEOUT: Duration = Duration::from_secs(10);

/// The read tick: how often a connection's reader wakes to check whether
/// it has been killed. Sets the kill switch's worst-case latency.
const TICK: Duration = Duration::from_millis(250);

struct Conn {
    info: ConnectionInfo,
    killed: Arc<AtomicBool>,
    /// The writer's inbox. Dropping the reader drops its sender; the
    /// writer drains what is left and exits.
    outbox: mpsc::Sender<Vec<u8>>,
}

/// Deregisters a connection no matter how its thread ends — a panicked
/// handler must not leave a ghost row in the panel.
struct RowGuard<'a> {
    server: &'a IpcServer,
    id: Option<u64>,
}

impl Drop for RowGuard<'_> {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            self.server.deregister(id);
        }
    }
}

/// The server. Construct with [`IpcServer::bind`], call
/// [`IpcServer::start`] to begin accepting, and read the panel with
/// [`IpcServer::panel`].
pub struct IpcServer {
    path: PathBuf,
    handler: RequestHandler,
    hello: HelloSource,
    store: Arc<crate::association::AssociationStore>,
    registry: Mutex<BTreeMap<u64, Conn>>,
    listeners: Mutex<Vec<(u64, ChangeSink)>>,
    /// Manifest staleness the shell's audit found (task-10). The server
    /// owns no filesystem opinions about browser manifests — it surfaces
    /// what it was handed, so the panel stays data.
    manifest_problems: Mutex<Vec<ManifestProblem>>,
    next_id: AtomicU64,
    next_listener: AtomicU64,
    /// The handshake waits, in the open, behind a mutex so tests can
    /// shorten them through the shared `Arc`.
    timeouts: Mutex<(Duration, Duration)>,
    listener: Mutex<Option<UnixListener>>,
}

impl std::fmt::Debug for IpcServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IpcServer")
            .field("path", &self.path)
            .field("socket", &self.path().exists())
            .field("accepting", &self.listener.lock().is_ok())
            .field("connections", &self.panel().connections.len())
            .field("remembered", &self.panel().remembered.len())
            .finish()
    }
}

impl IpcServer {
    /// Bind the well-known socket (or a test path) and load the
    /// association store from `store_path`.
    ///
    /// The socket's parent directory is created with 0700 and the socket
    /// itself is 0600: the per-user directory is the baseline same-user
    /// enforcement on every platform (doc-2 §3); the in-process credential
    /// check on accept is the second lock, not the only one.
    ///
    /// # Errors
    ///
    /// Binding fails if the path is unusable, if the association store is
    /// corrupt, or if another process is already listening (a stale socket
    /// from a crashed app is removed and rebound).
    pub fn bind(
        path: &Path,
        store_path: PathBuf,
        handler: RequestHandler,
        hello: HelloSource,
    ) -> std::io::Result<Arc<Self>> {
        let store = Arc::new(
            crate::association::AssociationStore::persisted(store_path)
                .map_err(std::io::Error::other)?,
        );
        Self::bind_with_store(path, store, handler, hello)
    }

    /// Bind with an existing (e.g. in-memory) association store.
    ///
    /// # Errors
    ///
    /// As [`IpcServer::bind`].
    pub fn bind_with_store(
        path: &Path,
        store: Arc<crate::association::AssociationStore>,
        handler: RequestHandler,
        hello: HelloSource,
    ) -> std::io::Result<Arc<Self>> {
        use std::os::unix::fs::PermissionsExt;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
        }
        // A live server owns the path; a dead one left a stale socket.
        if path.exists() {
            match UnixStream::connect(path) {
                Ok(_) => {
                    return Err(std::io::Error::new(
                        ErrorKind::AddrInUse,
                        format!("another Castellan is listening at {}", path.display()),
                    ));
                }
                Err(_) => std::fs::remove_file(path)?,
            }
        }
        let listener = UnixListener::bind(path)?;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;

        Ok(Arc::new(Self {
            path: path.to_path_buf(),
            handler,
            hello,
            store,
            registry: Mutex::new(BTreeMap::new()),
            listeners: Mutex::new(Vec::new()),
            manifest_problems: Mutex::new(Vec::new()),
            next_id: AtomicU64::new(0),
            next_listener: AtomicU64::new(0),
            timeouts: Mutex::new((APPROVAL_TIMEOUT, HELLO_TIMEOUT)),
            listener: Mutex::new(Some(listener)),
        }))
    }

    /// Shorten the handshake timeouts — tests only; production never
    /// needs to wait less than the defaults.
    #[must_use]
    pub fn with_timeouts(self: &Arc<Self>, approval: Duration, hello: Duration) -> Arc<Self> {
        *self.timeouts.lock().expect("timeout config") = (approval, hello);
        Arc::clone(self)
    }

    /// Start accepting connections. Idempotent: a second call is a no-op.
    pub fn start(self: &Arc<Self>) {
        let listener = {
            let mut slot = self.listener.lock().expect("listener slot");
            match slot.take() {
                Some(listener) => listener,
                None => return, // already started
            }
        };
        let server = Arc::clone(self);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => server.admit(stream),
                    Err(_) => continue, // a failed accept never stops the server
                }
            }
        });
    }

    /// Everything the panel renders, in one read.
    #[must_use]
    pub fn panel(&self) -> PanelSnapshot {
        let connections: Vec<ConnectionInfo> = self
            .registry
            .lock()
            .expect("connection registry")
            .values()
            .map(|conn| conn.info.clone())
            .collect();
        let pending: Vec<PendingKey> = self.store.pending();
        let remembered: Vec<RememberedKey> = self
            .store
            .remembered()
            .into_iter()
            .map(Into::into)
            .collect();
        let manifest_problems = self
            .manifest_problems
            .lock()
            .expect("manifest problems")
            .clone();
        PanelSnapshot {
            connections,
            pending,
            remembered,
            manifest_problems,
        }
    }

    /// Replace the manifest-staleness rows the panel shows (task-10).
    /// The shell runs the audit — at startup, after every repair, and
    /// whenever it rebuilds the settings — and hands the findings here;
    /// empty means every installed browser's manifest is current.
    pub fn set_manifest_problems(&self, problems: Vec<ManifestProblem>) {
        *self.manifest_problems.lock().expect("manifest problems") = problems;
        self.changed();
    }

    /// Remembered keys, oldest first.
    #[must_use]
    pub fn remembered_keys(&self) -> Vec<RememberedKey> {
        self.store
            .remembered()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    /// Confirm a pending enrollment. Returns whether one was pending.
    pub fn confirm(&self, key_id: &str) -> bool {
        let confirmed = self.store.confirm(key_id, unix_now());
        if confirmed {
            self.changed();
        }
        confirmed
    }

    /// Deny a pending enrollment. Returns whether one was pending.
    pub fn deny(&self, key_id: &str) -> bool {
        let denied = self.store.deny(key_id);
        if denied {
            self.changed();
        }
        denied
    }

    /// Drop a connection: the panel's kill switch. The connection notices
    /// within one [`TICK`], whether it is serving requests or still
    /// waiting for approval. Returns whether such a connection existed.
    pub fn kill(&self, connection_id: u64) -> bool {
        let killed = {
            let registry = self.registry.lock().expect("connection registry");
            match registry.get(&connection_id) {
                Some(conn) => {
                    conn.killed.store(true, Ordering::Relaxed);
                    true
                }
                None => false,
            }
        };
        if killed {
            self.changed();
        }
        killed
    }

    /// Push an event to every *ready* connection (lock state, entry
    /// churn — the session's bus, forwarded). Connections still
    /// handshaking get nothing: their first application byte is the hello
    /// they earned.
    pub fn broadcast(&self, event: Event) {
        let outboxes: Vec<mpsc::Sender<Vec<u8>>> = {
            let registry = self.registry.lock().expect("connection registry");
            registry
                .values()
                .filter(|conn| conn.info.state == ConnectionState::Ready)
                .map(|conn| conn.outbox.clone())
                .collect()
        };
        let mut frame = Vec::new();
        encode_frame(
            &serde_json::to_vec(&HostMessage::Event { event }).expect("events serialize"),
            &mut frame,
        );
        for outbox in outboxes {
            let _ = outbox.send(frame.clone());
        }
    }

    /// Subscribe to panel changes. Returns the subscription id for
    /// [`IpcServer::unsubscribe_changes`].
    pub fn subscribe_changes(&self, sink: ChangeSink) -> u64 {
        let id = self.next_listener.fetch_add(1, Ordering::Relaxed) + 1;
        self.listeners
            .lock()
            .expect("change listeners")
            .push((id, sink));
        id
    }

    /// Stop hearing panel changes.
    pub fn unsubscribe_changes(&self, id: u64) {
        self.listeners
            .lock()
            .expect("change listeners")
            .retain(|(listener_id, _)| *listener_id != id);
    }

    /// The socket path this server is bound to.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    fn changed(&self) {
        let sinks: Vec<ChangeSink> = self
            .listeners
            .lock()
            .expect("change listeners")
            .iter()
            .map(|(_, sink)| Arc::clone(sink))
            .collect();
        for sink in sinks {
            sink();
        }
    }

    /// Admit a fresh connection: peer credentials first, then a thread.
    fn admit(self: &Arc<Self>, mut stream: UnixStream) {
        if !peer_is_this_user(&mut stream) {
            // Cross-user: close before any handshake byte. The 0700 socket
            // directory should have kept them out already; this is the
            // second lock (doc-2 §3, same-user enforcement).
            return;
        }
        let _ = stream.set_read_timeout(Some(TICK));
        let _ = stream.set_write_timeout(Some(WRITE_TIMEOUT));
        let server = Arc::clone(self);
        std::thread::spawn(move || server.run_connection(stream));
    }

    /// One connection's whole life. All failure paths close without one
    /// application byte — the refusal *is* silence (doc-2 §3).
    fn run_connection(self: Arc<Self>, mut stream: UnixStream) {
        // The writer owns a clone; the reader owns the original. All
        // application bytes flow through the channel so a response and a
        // broadcast event can never interleave mid-frame.
        let writer_stream = match stream.try_clone() {
            Ok(clone) => clone,
            Err(_) => return,
        };
        let (outbox, inbox) = mpsc::channel::<Vec<u8>>();
        let writer = std::thread::spawn(move || {
            let mut writer_stream = writer_stream;
            while let Ok(frame) = inbox.recv() {
                if writer_stream.write_all(&frame).is_err() {
                    break;
                }
            }
        });
        let killed = Arc::new(AtomicBool::new(false));

        // The handshake and request loop; every refusal inside returns
        // `Err`, and the cleanup below runs for all of them alike.
        let _ = self.connection_life(&mut stream, &outbox, &killed);

        drop(outbox);
        let _ = writer.join();
    }

    /// Everything between accept and disconnect: hello, association,
    /// challenge, then the request loop. Returns `Err(())` on every
    /// refusal — the caller closes in silence.
    fn connection_life(
        self: &Arc<Self>,
        stream: &mut UnixStream,
        outbox: &mpsc::Sender<Vec<u8>>,
        killed: &Arc<AtomicBool>,
    ) -> Result<(), ()> {
        let (_, hello_timeout) = *self.timeouts.lock().expect("timeout config");

        // ── Hello ─────────────────────────────────────────────────────────
        let (protocol_version, face, client_version, association) =
            match read_client_message(stream, killed, Some(hello_timeout)) {
                ReadOutcome::Message(ClientMessage::Hello {
                    protocol_version,
                    face,
                    client_version,
                    association,
                }) => (protocol_version, face, client_version, association),
                _ => return Err(()), // no hello, or nothing resembling a client
            };
        let face = face.unwrap_or(FaceKind::Other);

        // The panel row goes up the moment the connection identifies
        // itself — "visible instead of silent" starts here, before trust.
        let mut guard = RowGuard {
            server: self,
            id: None,
        };
        let info = ConnectionInfo {
            id: self.next_id.fetch_add(1, Ordering::Relaxed) + 1,
            face,
            client_version,
            protocol_version,
            state: ConnectionState::AwaitingApproval,
            last_request: None,
            connected_at: unix_now(),
        };
        self.registry.lock().expect("connection registry").insert(
            info.id,
            Conn {
                info: info.clone(),
                killed: Arc::clone(killed),
                outbox: outbox.clone(),
            },
        );
        guard.id = Some(info.id);
        self.changed();

        // ── Association ───────────────────────────────────────────────────
        let key_id = match association {
            // A client that claims nothing is a client that gets nothing.
            None => return Err(()),
            Some(claim) => self.associate(stream, killed, &info, claim)?,
        };

        // ── Challenge / proof ─────────────────────────────────────────────
        let nonce = crate::association::nonce_hex().map_err(|_| ())?;
        send_host_message(
            outbox,
            &HostMessage::Challenge {
                key_id: key_id.clone(),
                nonce_hex: nonce.clone(),
            },
        )?;
        let proof_ok = match read_client_message(stream, killed, None) {
            ReadOutcome::Message(ClientMessage::Proof {
                key_id: proven,
                proof_hex,
            }) => proven == key_id && self.store.verify(&key_id, &nonce, &proof_hex),
            _ => false,
        };
        if !proof_ok {
            return Err(()); // wrong proof: silence
        }

        // ── Ready ─────────────────────────────────────────────────────────
        {
            let mut registry = self.registry.lock().expect("connection registry");
            if let Some(conn) = registry.get_mut(&info.id) {
                conn.info.state = ConnectionState::Ready;
            }
        }
        self.changed();

        // The app's hello — the first application byte the client receives.
        send_host_message(
            outbox,
            &HostMessage::Hello {
                hello: (self.hello)(),
            },
        )?;

        // ── The request loop ──────────────────────────────────────────────
        loop {
            match read_client_message(stream, killed, None) {
                ReadOutcome::Message(ClientMessage::Request { request }) => {
                    let method = request.method.tag().to_string();
                    let response = (self.handler)(request);
                    {
                        let mut registry = self.registry.lock().expect("connection registry");
                        if let Some(conn) = registry.get_mut(&info.id) {
                            conn.info.last_request = Some(method);
                        }
                    }
                    self.changed();
                    send_host_message(outbox, &HostMessage::Response { response })?;
                }
                ReadOutcome::Message(_) => {
                    // A second hello or a stray proof mid-session: protocol
                    // violation, and the safest answer to those is silence.
                    return Err(());
                }
                ReadOutcome::Eof | ReadOutcome::Timeout | ReadOutcome::Closed => return Ok(()),
            }
        }
    }

    /// The association leg: resolve a claim to a key id the app will
    /// challenge, or `Err` for refusal.
    fn associate(
        &self,
        stream: &mut UnixStream,
        killed: &Arc<AtomicBool>,
        info: &ConnectionInfo,
        claim: AssociationClaim,
    ) -> Result<String, ()> {
        match claim {
            AssociationClaim::Enroll {
                key_id,
                key_hex,
                label,
            } => self.enroll(killed, key_id, key_hex, label),
            AssociationClaim::Claim { key_id } => {
                if self.store.is_remembered(&key_id) {
                    return Ok(key_id);
                }
                // Unknown key: tell the client to enroll again, and accept
                // exactly one follow-up hello on this connection.
                let outbox = {
                    let registry = self.registry.lock().expect("connection registry");
                    registry
                        .get(&info.id)
                        .map(|conn| conn.outbox.clone())
                        .ok_or(())?
                };
                send_host_message(
                    &outbox,
                    &HostMessage::UnknownKey {
                        key_id: key_id.clone(),
                    },
                )?;
                match read_client_message(stream, killed, None) {
                    ReadOutcome::Message(ClientMessage::Hello {
                        association:
                            Some(AssociationClaim::Enroll {
                                key_id,
                                key_hex,
                                label,
                            }),
                        ..
                    }) => self.enroll(killed, key_id, key_hex, label),
                    _ => Err(()),
                }
            }
        }
    }

    /// Enroll: prompt on first contact, wait for the decision, and keep
    /// the connection alive while the prompt is up. A killed connection
    /// leaves the wait early; the prompt stays for any sibling.
    fn enroll(
        &self,
        killed: &Arc<AtomicBool>,
        key_id: String,
        key_hex: String,
        label: String,
    ) -> Result<String, ()> {
        let (approval_timeout, _) = *self.timeouts.lock().expect("timeout config");
        match self.store.enroll(&key_id, &key_hex, &label) {
            EnrollOutcome::AlreadyRemembered => Ok(key_id),
            EnrollOutcome::KeyIdCollision => Err(()),
            EnrollOutcome::NewlyPending => {
                self.changed(); // the prompt appears in the panel
                let outcome = self
                    .store
                    .wait_for_decision(&key_id, approval_timeout, Some(killed));
                self.changed(); // the prompt is gone either way
                match outcome {
                    WaitOutcome::Confirmed => Ok(key_id),
                    WaitOutcome::Denied | WaitOutcome::TimedOut | WaitOutcome::Cancelled => Err(()),
                }
            }
        }
    }

    fn deregister(&self, connection_id: u64) {
        self.registry
            .lock()
            .expect("connection registry")
            .remove(&connection_id);
        self.changed();
    }
}

impl From<crate::association::RememberedKey> for RememberedKey {
    fn from(key: crate::association::RememberedKey) -> Self {
        Self {
            key_id: key.key_id,
            label: key.label,
            added_at: key.added_at,
        }
    }
}

enum ReadOutcome {
    Message(ClientMessage),
    Eof,
    Timeout,
    Closed,
}

/// Read one client message, honoring the kill switch on every tick and an
/// optional deadline (the first hello; `None` = no deadline).
///
/// The frame read is *resumable*: a read timeout mid-frame (the tick that
/// lets us check the kill switch) keeps its partial bytes, because
/// `read_exact`-style recovery would silently desync the stream.
fn read_client_message(
    stream: &mut UnixStream,
    killed: &Arc<AtomicBool>,
    deadline: Option<Duration>,
) -> ReadOutcome {
    let deadline_at = deadline.map(|duration| Instant::now() + duration);
    let mut frame = FrameReader::new();
    loop {
        if killed.load(Ordering::Relaxed) {
            return ReadOutcome::Closed;
        }
        match frame.read_more(stream) {
            Ok(Some(payload)) => match serde_json::from_slice(&payload) {
                Ok(message) => return ReadOutcome::Message(message),
                Err(_) => return ReadOutcome::Eof, // not the protocol: silence
            },
            Ok(None) => {
                if let Some(deadline_at) = deadline_at {
                    if Instant::now() >= deadline_at {
                        return ReadOutcome::Timeout;
                    }
                }
            }
            Err(error) => match error.kind() {
                ErrorKind::UnexpectedEof => return ReadOutcome::Eof,
                _ => return ReadOutcome::Eof, // garbage or I/O failure: silence
            },
        }
    }
}

/// A frame reader that survives read timeouts: `read_more` continues from
/// wherever the previous call stopped, so a tick that returned no bytes
/// costs nothing but a retry.
struct FrameReader {
    header: [u8; 4],
    header_read: usize,
    payload: Vec<u8>,
    payload_read: usize,
}

impl FrameReader {
    fn new() -> Self {
        Self {
            header: [0; 4],
            header_read: 0,
            payload: Vec::new(),
            payload_read: 0,
        }
    }

    /// Read as much of the frame as the socket gives without blocking past
    /// the stream's read timeout. `Ok(Some(payload))` = a whole frame;
    /// `Ok(None)` = tick elapsed, call again; `Err` = the stream is done
    /// or broken (the same conditions [`castellan_ipc::decode_frame`]
    /// rejects, plus I/O errors).
    fn read_more(&mut self, stream: &mut UnixStream) -> std::io::Result<Option<Vec<u8>>> {
        while self.header_read < 4 {
            match stream.read(&mut self.header[self.header_read..]) {
                Ok(0) => return Err(std::io::Error::from(ErrorKind::UnexpectedEof)),
                Ok(n) => self.header_read += n,
                // A tick elapsed (the kill-switch check), or a signal
                // interrupted the blocking read: resume, never treat a
                // spurious EINTR as a broken stream.
                Err(error)
                    if error.kind() == ErrorKind::WouldBlock
                        || error.kind() == ErrorKind::TimedOut
                        || error.kind() == ErrorKind::Interrupted =>
                {
                    return Ok(None);
                }
                Err(error) => return Err(error),
            }
        }
        if self.payload.is_empty() && self.payload_read == 0 {
            // First time past the header: size the payload, with the same
            // cap as `castellan_ipc::decode_frame`. The cap bounds this
            // allocation; a browser that lies about a longer frame is
            // refused before it buffers.
            let len = u32::from_le_bytes(self.header);
            if len > MAX_MESSAGE_BYTES {
                return Err(std::io::Error::new(
                    ErrorKind::InvalidData,
                    format!("frame of {len} bytes exceeds the {MAX_MESSAGE_BYTES}-byte cap"),
                ));
            }
            self.payload = vec![0; len as usize];
        }
        while self.payload_read < self.payload.len() {
            let filled = &mut self.payload[self.payload_read..];
            match stream.read(filled) {
                Ok(0) => return Err(std::io::Error::from(ErrorKind::UnexpectedEof)),
                Ok(n) => self.payload_read += n,
                // A tick elapsed (the kill-switch check), or a signal
                // interrupted the blocking read: resume, never treat a
                // spurious EINTR as a broken stream.
                Err(error)
                    if error.kind() == ErrorKind::WouldBlock
                        || error.kind() == ErrorKind::TimedOut
                        || error.kind() == ErrorKind::Interrupted =>
                {
                    return Ok(None);
                }
                Err(error) => return Err(error),
            }
        }
        Ok(Some(std::mem::take(&mut self.payload)))
    }
}

/// Send one host message, frame and all, to the connection's writer.
/// `Err(())` means the writer is gone — the connection is over either way.
fn send_host_message(outbox: &mpsc::Sender<Vec<u8>>, message: &HostMessage) -> Result<(), ()> {
    let payload = serde_json::to_vec(message).expect("host messages serialize");
    let mut frame = Vec::new();
    encode_frame(&payload, &mut frame);
    outbox.send(frame).map_err(|_| ())
}

/// Same-user enforcement, the in-process half (doc-2 §3).
///
/// On Linux the kernel reports the peer's uid via `SO_PEERCRED`; on macOS
/// and iOS `getpeereid` does the same. Either way the peer must match
/// this process's real uid ([`crate::same_user`]). On the remaining
/// Unixes there is no portable answer without more cfg than the check is
/// worth, so those rely on the 0700 per-user socket directory — the
/// baseline lock on every platform, as doc-2 §3 requires. The decision
/// itself is [`same_user`], kept pure because no test can spawn a second
/// uid in CI.
///
/// These two FFI calls are the workspace's entire sanctioned `unsafe`
/// surface (see the `[workspace.lints]` comment in the root manifest):
/// std's `peer_cred` is still unstable (rust#42839), so the one
/// `getsockopt` the security model requires goes through libc — audited
/// here rather than hand-rolled anywhere else.
fn peer_is_this_user(stream: &mut UnixStream) -> bool {
    use std::os::fd::AsRawFd;

    let peer_uid = peer_uid(stream.as_raw_fd());
    match peer_uid {
        Some(peer_uid) => same_user(peer_uid, own_uid()),
        None => false, // no credentials, no connection
    }
}

/// The peer's uid, straight from the kernel, or `None` where the platform
/// has no portable answer.
#[cfg(target_os = "linux")]
#[allow(unsafe_code)] // the sanctioned exception — see `peer_is_this_user`
fn peer_uid(fd: std::os::unix::io::RawFd) -> Option<u32> {
    let mut cred = libc::ucred {
        pid: 0,
        uid: 0,
        gid: 0,
    };
    let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: `fd` is a live, connected AF_UNIX socket (its `UnixStream`
    // is borrowed for this call), the target pointer addresses a
    // `libc::ucred` we own for the duration, and `len` carries the
    // kernel-expected size in and the kernel-reported size out. The kernel
    // writes at most `size_of::<ucred>()` bytes — `size_of_val` of the
    // same value we pass — so nothing can write out of bounds.
    let status = unsafe {
        libc::getsockopt(
            fd,
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            std::ptr::addr_of_mut!(cred).cast(),
            &mut len,
        )
    };
    (status == 0).then_some(cred.uid)
}

/// The peer's uid via `getpeereid` (macOS, iOS).
#[cfg(any(target_os = "macos", target_os = "ios"))]
#[allow(unsafe_code)] // the sanctioned exception — see `peer_is_this_user`
fn peer_uid(fd: std::os::unix::io::RawFd) -> Option<u32> {
    let mut uid = 0;
    let mut gid = 0;
    // SAFETY: `fd` is a live, connected AF_UNIX socket (its `UnixStream`
    // is borrowed for this call), and both out-parameters address owned
    // `uid_t`/`gid_t` locals valid for the duration. `getpeereid` writes
    // exactly one value into each and reads nothing else.
    let status = unsafe { libc::getpeereid(fd, &mut uid, &mut gid) };
    (status == 0).then_some(uid)
}

/// Other Unixes: no portable peer-uid call without a libc dependency per
/// platform; the 0700 socket directory is the enforcement.
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "ios")))]
fn peer_uid(_fd: std::os::unix::io::RawFd) -> Option<u32> {
    None
}

/// This process's real uid.
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "ios"))]
#[allow(unsafe_code)] // the sanctioned exception — see `peer_is_this_user`
fn own_uid() -> u32 {
    // SAFETY: `getuid` takes no arguments, returns a `uid_t` by value, and
    // cannot fail (the raw syscall interface has no error path).
    unsafe { libc::getuid() }
}

/// Other Unixes: no portable own-uid call; peer checks are disabled there
/// and the 0700 socket directory is the enforcement.
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "ios")))]
fn own_uid() -> u32 {
    u32::MAX // never matches a peer: refuse rather than admit
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or(0)
}
