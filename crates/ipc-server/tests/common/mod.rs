//! Shared scaffolding for the IPC server suite: scratch dirs, a framed
//! test client, and a poll-with-deadline helper. Everything eats the
//! public API only, exactly like the desktop shell will.

use std::io::{ErrorKind, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use castellan_ipc::{decode_frame, encode_frame};
use castellan_ipc_server::{AssociationStore, HelloSource, IpcServer, RequestHandler, proof_hex};
use castellan_protocol::{
    AssociationClaim, ClientMessage, FaceKind, Hello, HostMessage, PROTOCOL_VERSION, RpcError,
    RpcErrorCode, RpcMethod, RpcRequest, RpcResponse, RpcResult, VaultStatus,
};

/// A fresh directory per test, under the OS temp dir.
pub(crate) fn scratch(tag: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let unique = NEXT.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "castellan-ipc-server-{}-{tag}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

/// The test handler: ping answers, entries answer empty, everything else
/// answers the locked-vault error so error paths stay exercised.
pub(crate) fn handler() -> RequestHandler {
    Arc::new(|request: RpcRequest| {
        let id = request.id;
        match request.method {
            RpcMethod::Ping {} => RpcResponse {
                id,
                result: Some(RpcResult::Ping {}),
                error: None,
            },
            RpcMethod::GetEntries { .. } => RpcResponse {
                id,
                result: Some(RpcResult::GetEntries {
                    entries: Vec::new(),
                }),
                error: None,
            },
            other => RpcResponse {
                id,
                result: None,
                error: Some(RpcError {
                    code: RpcErrorCode::VaultLocked,
                    message: format!("{} is not wired up in tests", other.tag()),
                }),
            },
        }
    })
}

/// The test app hello.
pub(crate) fn hello_source() -> HelloSource {
    Arc::new(|| Hello {
        protocol_version: PROTOCOL_VERSION,
        app_version: "test-app".to_string(),
        vault: VaultStatus::Locked,
        capabilities: Vec::new(),
    })
}

/// Bind and start a server with an in-memory association store, on a
/// fresh socket. Returns the server and its scratch dir (for cleanup).
pub(crate) fn spawn_server(tag: &str) -> (Arc<IpcServer>, PathBuf) {
    let dir = scratch(tag);
    let server = IpcServer::bind_with_store(
        &dir.join("castellan.sock"),
        Arc::new(AssociationStore::in_memory()),
        handler(),
        hello_source(),
    )
    .expect("bind test server");
    server.start();
    (server, dir)
}

/// Poll `predicate` until it holds, or panic. The suite's answer to
/// "the server thread is async to the test thread".
pub(crate) fn wait_until(what: &str, mut predicate: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if predicate() {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("timed out waiting for {what}");
}

/// A framed client, speaking the same wire as the extension's host.
pub(crate) struct TestClient {
    stream: UnixStream,
}

impl TestClient {
    /// Connect to a running server.
    pub(crate) fn connect(server: &IpcServer) -> Self {
        let stream = UnixStream::connect(server.path()).expect("connect");
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("read timeout");
        Self { stream }
    }

    /// Send one client message.
    pub(crate) fn send(&mut self, message: &ClientMessage) {
        let payload = serde_json::to_vec(message).expect("client messages serialize");
        let mut frame = Vec::new();
        encode_frame(&payload, &mut frame);
        self.stream.write_all(&frame).expect("send frame");
    }

    /// Read one host message, or `None` if nothing arrives in time.
    pub(crate) fn recv(&mut self) -> Option<HostMessage> {
        match decode_frame(&mut self.stream) {
            Ok(payload) => serde_json::from_slice(&payload).ok(),
            Err(error) if error.kind() == ErrorKind::WouldBlock => None,
            Err(_) => None,
        }
    }

    /// Read one host message, panicking with `what` if nothing arrives.
    pub(crate) fn expect(&mut self, what: &str) -> HostMessage {
        self.recv()
            .unwrap_or_else(|| panic!("expected a host message: {what}"))
    }

    /// The refusal answer: the connection closes and *zero* application
    /// bytes precede the close. Panics if bytes arrive or the connection
    /// merely stalls.
    pub(crate) fn expect_silence(&mut self) {
        let mut one_byte = [0u8; 1];
        match self.stream.read(&mut one_byte) {
            Ok(0) => {} // EOF: closed, silent
            Ok(_) => panic!("expected silence, received application bytes"),
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                panic!("expected silence, connection stayed open instead")
            }
            Err(_) => {} // a hard close is a close
        }
    }

    /// Assert that nothing arrives within `quiet` — the "still waiting,
    /// correctly" check for handshakes in progress.
    pub(crate) fn expect_nothing_for(&mut self, quiet: Duration) {
        self.stream
            .set_read_timeout(Some(quiet))
            .expect("read timeout");
        let mut one_byte = [0u8; 1];
        match self.stream.read(&mut one_byte) {
            Ok(0) => panic!("expected quiet, connection closed"),
            Ok(_) => panic!("expected quiet, received application bytes"),
            Err(error) if error.kind() == ErrorKind::WouldBlock => {}
            Err(error) => panic!("expected quiet, got {error}"),
        }
        self.stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("read timeout");
    }

    /// Send a v3 hello.
    pub(crate) fn hello(&mut self, face: FaceKind, version: &str, association: AssociationClaim) {
        self.send(&ClientMessage::Hello {
            protocol_version: PROTOCOL_VERSION,
            face: Some(face),
            client_version: Some(version.to_string()),
            association: Some(association),
        });
    }

    /// Enroll as `key`.
    pub(crate) fn enroll(&mut self, face: FaceKind, key_id: &str, key_hex: &str) {
        self.hello(
            face,
            "test-extension",
            AssociationClaim::Enroll {
                key_id: key_id.to_string(),
                key_hex: key_hex.to_string(),
                label: format!("{face:?} on this machine"),
            },
        );
    }

    /// Claim a remembered key.
    pub(crate) fn claim(&mut self, face: FaceKind, key_id: &str) {
        self.hello(
            face,
            "test-extension",
            AssociationClaim::Claim {
                key_id: key_id.to_string(),
            },
        );
    }

    /// Receive the challenge and answer it with the right proof.
    /// Returns the key id the challenge named.
    pub(crate) fn answer_challenge(&mut self, key_hex: &str) -> String {
        match self.expect("challenge") {
            HostMessage::Challenge { key_id, nonce_hex } => {
                let proof = proof_hex(key_hex, &nonce_hex);
                self.send(&ClientMessage::Proof {
                    key_id: key_id.clone(),
                    proof_hex: proof,
                });
                key_id
            }
            other => panic!("expected a challenge, got {other:?}"),
        }
    }

    /// The tail of a ready connection's handshake: the app's hello.
    pub(crate) fn expect_app_hello(&mut self) {
        match self.expect("app hello") {
            HostMessage::Hello { hello } => {
                assert_eq!(hello.protocol_version, PROTOCOL_VERSION);
                assert_eq!(hello.app_version, "test-app");
            }
            other => panic!("expected the app's hello, got {other:?}"),
        }
    }

    /// One RPC round trip.
    pub(crate) fn ping(&mut self, id: u64) -> RpcResponse {
        self.send(&ClientMessage::Request {
            request: RpcRequest {
                id,
                method: RpcMethod::Ping {},
            },
        });
        match self.expect("response") {
            HostMessage::Response { response } => response,
            other => panic!("expected a response, got {other:?}"),
        }
    }
}
