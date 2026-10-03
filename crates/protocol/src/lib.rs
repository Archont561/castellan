//! The Castellan protocol: the one message language every face speaks.
//!
//! Desktop and mobile reach it through a single Tauri `rpc` command; the
//! browser extension reaches it through native messaging and the IPC socket;
//! a future CLI and the LAN sync channel will reach it through the same
//! socket. One set of types, one serialization, every surface — which is why
//! the TypeScript definitions in `packages/protocol/src/generated` are
//! *derived* from this crate by `castellan-xtask`, never written by hand.
//! A second copy of a message shape is a second thing to keep correct, and
//! the copy is the one the other half of the product speaks.
//!
//! This crate depends on nothing inside the workspace. The protocol must not
//! depend on the engine that executes it: a client that can only express a
//! request by asking the app what it means is a client that cannot be built
//! ahead of the app. (The same rule, and the same reason, as geoquery's
//! `types` crate.)
//!
//! Naming: fields stay snake_case across the language boundary. One naming
//! rule everywhere beats the "right" rule per language, and `ts-rs` mirrors
//! serde exactly, so the wire and the types can never disagree about a field.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The protocol version this build speaks.
///
/// Every connection starts with a [`ClientMessage::Hello`] carrying the
/// client's number, and the app answers with its own. A mismatch is not an
/// error: it is the input to capability negotiation — an old extension
/// talking to a new app degrades to the methods both sides know, rather than
/// dying with "cannot connect" the way version-skewed proxy setups do.
///
/// Version 3 added the association handshake: the client hello carries a
/// face, a client version, and an association claim; the app answers with a
/// nonce challenge before it will send anything else (doc-2 §3).
pub const PROTOCOL_VERSION: u32 = 3;

/// What a connected face can be asked to do.
///
/// Sent in [`Hello`] so the *client* can adapt its UI before the first
/// request, rather than discovering a missing capability by error path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum Capability {
    /// Fill passwords and forms.
    Autofill,
    /// Generate TOTP codes for entries that carry an `otpauth` seed.
    Totp,
    /// Act as a WebAuthn authenticator (the "Fob" passkey layer).
    Passkeys,
    /// Structured one-time recovery codes with used/unused tracking.
    RecoveryCodes,
    /// LocalSend-compatible transfer between paired devices.
    Beam,
}

/// Which face a connected client is, for the connected-browsers panel.
///
/// The client self-reports this in its hello; the panel displays it, and
/// the app-side origin enforcement (doc-1) never trusts it for a security
/// decision — it is a label, not a credential.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum FaceKind {
    /// Google Chrome (and, for manifest purposes, Chromium).
    Chrome,
    /// Microsoft Edge.
    Edge,
    /// Brave.
    Brave,
    /// Vivaldi.
    Vivaldi,
    /// Firefox (and derivatives that share its native-messaging layout).
    Firefox,
    /// Safari, through the app-extension transport rather than a host.
    Safari,
    /// The CLI, over the same socket (task-34).
    Cli,
    /// Anything else — future browsers, test harnesses.
    Other,
}

/// The association claim in a client hello: who the client says it is,
/// key-wise (doc-2 §3).
///
/// The first connection from an extension enrolls — it presents its key
/// material so the app can prompt "Chrome wants to connect" and remember
/// the key. Every later connection only claims the key id and proves
/// possession with an HMAC over the app's nonce challenge. The two shapes
/// share one type so the protocol can grow an asymmetric proof
/// (Ed25519) without changing the handshake's message structure: the
/// enroll variant carries whatever public material the proof scheme
/// needs, the claim variant stays a bare key id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum AssociationClaim {
    /// First contact: here is my key material, remember me.
    ///
    /// The key crosses this channel once, on the local socket, after the
    /// app has checked the peer credentials — the same-user rule is what
    /// makes carrying the material acceptable. With a symmetric proof the
    /// material is the HMAC key itself; with an asymmetric proof it would
    /// be the public key only.
    Enroll {
        /// The key's stable identifier (a fingerprint of the material).
        key_id: String,
        /// The key material, hex-encoded.
        key_hex: String,
        /// A human label for the prompt ("Chrome on this machine").
        label: String,
    },
    /// Returning: I am the key with this id; challenge me.
    Claim {
        /// The key's stable identifier.
        key_id: String,
    },
}

/// The vault, as a client sees it from the outside.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum VaultStatus {
    /// A database is open and unlocked.
    Unlocked,
    /// A database is loaded but locked; requests that need secrets fail with
    /// [`RpcErrorCode::VaultLocked`].
    Locked,
    /// No database has been created or opened on this device yet.
    NoDatabase,
}

/// The handshake answer: who the app is and what it can do right now.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Hello {
    /// The protocol version the app speaks.
    pub protocol_version: u32,
    /// The app's own version, for the "connected browsers" panel and diagnostics.
    pub app_version: String,
    /// Vault state as of the handshake.
    pub vault: VaultStatus,
    /// What this app can be asked to do.
    pub capabilities: Vec<Capability>,
}

/// A vault entry, flattened to what a fill or a list view needs.
///
/// Never carries secret material: passwords, seeds and private keys cross the
/// channel only through explicit `Get*` methods, one entry at a time, so a
/// list request can never leak the database wholesale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct EntrySummary {
    /// Stable entry identifier (the KDBX UUID).
    pub id: String,
    /// Display title.
    pub title: String,
    /// Username, when the entry has one.
    pub username: Option<String>,
    /// Login URL, when the entry has one.
    pub url: Option<String>,
    /// Whether the entry carries a TOTP seed.
    pub has_totp: bool,
    /// Whether the entry carries a passkey.
    pub has_passkey: bool,
}

/// A new entry, as the extension's save prompt produces it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct NewEntry {
    /// Display title.
    pub title: String,
    /// Username to store.
    pub username: Option<String>,
    /// Password to store, already protected in transit by the channel.
    pub password: Option<String>,
    /// Login URL to store.
    pub url: Option<String>,
    /// An `otpauth://` URI, exactly as scanned. Parsed by `castellan-otp` on
    /// the app side; stored raw so a future format change never loses the
    /// original seed text.
    pub otpauth: Option<String>,
}

/// A generated TypeScript client surface.
///
/// Operations declare their intended faces next to their wire shape. The
/// repository xtask reads this metadata and emits one client per face; adding
/// a desktop-only action therefore cannot accidentally expose it to a browser
/// extension merely because both clients share the same transport core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientTarget {
    /// The desktop Tauri face.
    Desktop,
    /// The iOS/Android Tauri face.
    Mobile,
    /// The Fob browser extension.
    WebExtension,
}

/// Code-generation metadata for one request/result pair.
///
/// This is intentionally data emitted by the same macro that emits
/// [`RpcMethod`] and [`RpcResult`], rather than a second operation registry in
/// `castellan-xtask`. The field names let the generator preserve the wire's
/// snake_case while presenting idiomatic camelCase TypeScript arguments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RpcOperation {
    /// The snake_case method tag used on the wire.
    pub method: &'static str,
    /// Faces whose generated clients expose this operation.
    pub clients: &'static [ClientTarget],
    /// Request payload fields, excluding the `method` tag.
    pub request_fields: &'static [&'static str],
    /// Success payload fields, excluding the `type` tag.
    pub response_fields: &'static [&'static str],
}

impl RpcOperation {
    /// Whether this operation belongs on `target`'s generated client.
    #[must_use]
    pub fn supports(self, target: ClientTarget) -> bool {
        self.clients.contains(&target)
    }
}

/// Declare the protocol's operations once.
///
/// Every entry expands to a request variant, its correlated success variant,
/// and the metadata consumed by `castellan-xtask`. Request and result tags are
/// deliberately identical: TypeScript can express `ResultFor<M>` with
/// `Extract` instead of maintaining a handwritten method-to-result map.
macro_rules! rpc_contract {
    (
        $(
            $(#[$operation_meta:meta])*
            $variant:ident => $wire:literal for [$($client:ident),+ $(,)?] {
                request {
                    $(
                        $(#[$request_meta:meta])*
                        $request_field:ident: $request_type:ty
                    ),* $(,)?
                }
                response {
                    $(
                        $(#[$response_meta:meta])*
                        $response_field:ident: $response_type:ty
                    ),* $(,)?
                }
            }
        )+
    ) => {
        /// One RPC call, tagged by method name.
        ///
        /// Internally tagged (`{"method": "get_entries", ...}`) so a
        /// method is readable on the wire and a typo is a parse error, not a
        /// silent dispatch miss. The tag strings stay snake_case on the wire
        /// and in the generated TypeScript.
        #[derive(Debug, Clone, Serialize, Deserialize, TS)]
        #[serde(tag = "method")]
        #[ts(export)]
        pub enum RpcMethod {
            $(
                $(#[$operation_meta])*
                #[serde(rename = $wire)]
                $variant {
                    $(
                        $(#[$request_meta])*
                        $request_field: $request_type,
                    )*
                },
            )+
        }

        impl RpcMethod {
            /// The snake_case wire tag of this method — the name logs, the
            /// connected-browsers panel and diagnostics display. Generated
            /// with the enum so a new operation can never miss it.
            #[must_use]
            pub fn tag(&self) -> &'static str {
                match self {
                    $(RpcMethod::$variant { .. } => $wire,)+
                }
            }
        }

        /// The payload of a successful call, correlated with [`RpcMethod`]
        /// by the same snake_case operation tag.
        #[derive(Debug, Clone, Serialize, Deserialize, TS)]
        #[serde(tag = "type")]
        #[ts(export)]
        pub enum RpcResult {
            $(
                $(#[$operation_meta])*
                #[serde(rename = $wire)]
                $variant {
                    $(
                        $(#[$response_meta])*
                        $response_field: $response_type,
                    )*
                },
            )+
        }

        /// Every RPC operation, in declaration order, for client generation.
        pub const RPC_OPERATIONS: &[RpcOperation] = &[
            $(
                RpcOperation {
                    method: $wire,
                    clients: &[$(ClientTarget::$client),+],
                    request_fields: &[$(stringify!($request_field)),*],
                    response_fields: &[$(stringify!($response_field)),*],
                },
            )+
        ];
    };
}

rpc_contract! {
    /// Entries relevant to an origin, for the fill UI.
    GetEntries => "get_entries" for [Desktop, Mobile, WebExtension] {
        request {
            /// The page origin the fill is happening on.
            origin: String,
        }
        response {
            /// Entries relevant to the requested origin.
            entries: Vec<EntrySummary>,
        }
    }
    /// The current TOTP code for one entry.
    GetTotp => "get_totp" for [Desktop, Mobile, WebExtension] {
        request {
            /// Which entry to compute a code for.
            entry_id: String,
        }
        response {
            /// The code, right-aligned and zero-padded as services expect it.
            code: String,
            /// Seconds until this code expires; the UI draws its progress
            /// ring from this and never re-implements period arithmetic.
            seconds_remaining: u32,
        }
    }
    /// A generated passphrase for an app's entry editor.
    GeneratePassphrase => "generate_passphrase" for [Desktop, Mobile] {
        request {
            /// How many words to join.
            words: u32,
            /// What to join them with.
            separator: String,
        }
        response {
            /// The generated passphrase.
            value: String,
        }
    }
    /// Save an entry captured by the extension's save prompt.
    SaveEntry => "save_entry" for [WebExtension] {
        request {
            /// The captured entry.
            entry: NewEntry,
        }
        response {
            /// The id the saved entry received.
            id: String,
        }
    }
    /// Lock the vault now. Always available; never fails.
    LockDatabase => "lock_database" for [Desktop, Mobile, WebExtension] {
        request {}
        response {}
    }
    /// Liveness probe.
    Ping => "ping" for [Desktop, Mobile, WebExtension] {
        request {}
        response {}
    }
}

/// A request envelope. `id` is echoed in [`RpcResponse`] so a transport that
/// multiplexes (native messaging does) can pair answers to callers.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RpcRequest {
    /// Caller-chosen id, echoed by the response.
    /// Pinned to `number` on the field (not via xtask's export config) so
    /// every export path emits the same bytes; ids are small, and a
    /// `bigint` id would tax every TypeScript caller.
    #[ts(type = "number")]
    pub id: u64,
    /// The call itself; flattened onto the request object.
    #[serde(flatten)]
    pub method: RpcMethod,
}

/// Stable application errors carried by [`RpcError`].
///
/// This enum is the source of truth for both Rust and the generated
/// TypeScript union. Transport and client-validation failures are local to
/// the TypeScript client and deliberately do not cross the wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum RpcErrorCode {
    /// The vault exists but is locked.
    VaultLocked,
    /// The presented master password / keyfile did not open the vault.
    /// Carried by the app-side unlock flow (the extension cannot unlock —
    /// secrets stay app-side), so the prompt can offer "try again" rather
    /// than a generic failure.
    BadCredentials,
    /// The vault file exists but cannot be opened as a vault at all:
    /// unreadable, corrupt, or an unsupported format. Distinct from
    /// [`RpcErrorCode::BadCredentials`] so the UI treats it as fatal
    /// ("pick another file") rather than retryable.
    VaultUnreadable,
    /// The request named an entry that does not exist. Distinct from an empty
    /// list so the UI can say "nothing for this site" without guessing
    /// whether it queried the right database.
    NoSuchEntry,
    /// The protocol knows the operation but this build cannot execute it yet.
    NotImplemented,
}

/// The machine-readable half of a failure. Codes are stable strings the UI
/// can switch on; the message is for humans.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RpcError {
    /// Stable, machine-readable error code.
    pub code: RpcErrorCode,
    /// Human-readable explanation.
    pub message: String,
}

/// The answer to a request. Exactly one of `result`/`error` is `Some`; both
/// are `Option` rather than an enum so the wire shape stays `{id, result?}`
/// and the TypeScript stays one flat type.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RpcResponse {
    /// The id of the request this answers.
    /// Pinned to `number` on the field (not via xtask's export config) so
    /// every export path emits the same bytes; ids are small, and a
    /// `bigint` id would tax every TypeScript caller.
    #[ts(type = "number")]
    pub id: u64,
    /// The successful payload, when the call worked.
    pub result: Option<RpcResult>,
    /// The failure, when it did not.
    pub error: Option<RpcError>,
}

/// Events the app pushes at connected faces. Lock state is pushed, not
/// polled: the extension's icon reflects the vault *live*, and the stale
/// "reconnect to your database" class of failure cannot exist because there
/// is no state to go stale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum Event {
    /// The vault went from unlocked to locked.
    DatabaseLocked,
    /// The vault went from locked to unlocked.
    DatabaseUnlocked,
    /// One entry was created, changed or deleted.
    EntryChanged {
        /// Which entry.
        id: String,
    },
}

/// What a connected client sends over the native channel.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum ClientMessage {
    /// First message on a fresh connection.
    Hello {
        /// The protocol version the client speaks.
        protocol_version: u32,
        /// Which face is connecting, for the panel. `None` on clients
        /// older than the association handshake (version 2).
        #[serde(default)]
        face: Option<FaceKind>,
        /// The client's own version string, for the panel.
        #[serde(default)]
        client_version: Option<String>,
        /// The association claim: enroll on first contact, claim after.
        #[serde(default)]
        association: Option<AssociationClaim>,
    },
    /// The answer to the app's nonce challenge: proof of key possession.
    Proof {
        /// Which key this proves possession of.
        key_id: String,
        /// HMAC-SHA256 over the challenge nonce, hex-encoded.
        proof_hex: String,
    },
    /// An RPC call.
    Request {
        /// The call.
        request: RpcRequest,
    },
}

/// What the app sends back over the native channel.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[ts(export)]
pub enum HostMessage {
    /// The answer to a client's hello — sent only after the association
    /// handshake completes (or for clients that predate it).
    Hello {
        /// The app's handshake.
        hello: Hello,
    },
    /// Prove you hold the key you claimed: HMAC-SHA256 over this nonce.
    Challenge {
        /// Which key to prove possession of.
        key_id: String,
        /// The nonce, hex-encoded. Fresh per connection; never reused.
        nonce_hex: String,
    },
    /// The key id you claimed is not remembered here — enroll again.
    UnknownKey {
        /// The key id the app does not know.
        key_id: String,
    },
    /// The answer to a request.
    Response {
        /// The response payload.
        response: RpcResponse,
    },
    /// A pushed event.
    Event {
        /// What happened.
        event: Event,
    },
}

/// Where a connection stands in the association handshake.
///
/// The panel renders this as the row's state: a connection waiting for
/// approval is the "Chrome wants to connect" prompt made visible; a ready
/// connection is a live browser integration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
#[ts(export)]
pub enum ConnectionState {
    /// The key was presented but the user has not confirmed it yet; the
    /// connection gets no data until they do (or it is denied).
    AwaitingApproval,
    /// The handshake completed; requests and events flow.
    Ready,
}

/// One row of the connected-browsers panel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ConnectionInfo {
    /// Connection identifier, stable for the connection's lifetime.
    pub id: u64,
    /// Which face is connected.
    pub face: FaceKind,
    /// The client's version string, when it sent one.
    pub client_version: Option<String>,
    /// The protocol version the client speaks.
    pub protocol_version: u32,
    /// Association state: awaiting approval or ready.
    pub state: ConnectionState,
    /// The last RPC method this connection sent, for "is it alive" and
    /// for debugging a silent integration. A method name, never a
    /// payload — the panel must not become a side channel.
    pub last_request: Option<String>,
    /// When the connection opened, unix seconds.
    pub connected_at: u64,
}

/// One remembered association key, for the settings list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RememberedKey {
    /// The key's stable identifier (a fingerprint of its material).
    pub key_id: String,
    /// The label chosen at enrollment ("Chrome on this machine").
    pub label: String,
    /// When the user confirmed the key, unix seconds.
    pub added_at: u64,
}

/// An enrollment waiting for the user's decision — the "Chrome wants to
/// connect" prompt, as data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PendingKey {
    /// The key's stable identifier.
    pub key_id: String,
    /// The label the client chose at enrollment.
    pub label: String,
}

/// Everything the connected-browsers panel renders in one read: live
/// connections, enrollments awaiting a decision, and the remembered keys
/// the settings list shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct PanelSnapshot {
    /// Live connections, in connect order.
    pub connections: Vec<ConnectionInfo>,
    /// Enrollments waiting for the user, oldest first.
    pub pending: Vec<PendingKey>,
    /// Keys the user has confirmed, oldest first.
    pub remembered: Vec<RememberedKey>,
}

/// Origin matching, shared by every face.
///
/// The extension asks "which entries fill at this origin?"; the app answers
/// using this function, and the WASM build lets the extension pre-filter with
/// the *same* rule. One implementation means the two can never disagree about
/// whether github.com matches accounts.github.com.
pub mod matching {
    /// Whether `candidate` (an entry URL) should be offered at `origin`.
    ///
    /// Exact host equality, or a subdomain of the candidate's host: an entry
    /// for `example.org` fills at `login.example.org`. The reverse (an entry
    /// for `login.example.org` at `example.org`) does *not* match — a login
    /// page on a subdomain is usually a different, less-trusted service.
    /// Schemes and paths are ignored; ports are compared when present.
    pub fn origin_matches(origin: &str, candidate: &str) -> bool {
        let (origin_host, origin_port) = split_host_port(origin);
        let (candidate_host, candidate_port) = split_host_port(candidate);
        if origin_port.is_some() && candidate_port.is_some() && origin_port != candidate_port {
            return false;
        }
        origin_host == candidate_host
            || origin_host
                .strip_suffix(&format!(".{candidate_host}"))
                .is_some_and(|prefix| !prefix.is_empty() && !prefix.contains('.'))
    }

    /// Split `host[:port]` from whatever the caller had: a bare host, a Web
    /// origin (`https://github.com`), or a full entry URL
    /// (`https://github.com/login`). Schemes, paths, queries and fragments
    /// are stripped. Userinfo (`user@host`) and IPv6 literals are passed
    /// through untouched — hosts here come from parsed URLs and origins
    /// upstream, not from raw sockets, and neither appears in practice.
    fn split_host_port(input: &str) -> (String, Option<String>) {
        let authority = match input.split_once("://") {
            Some((_, rest)) => rest,
            None => input,
        };
        let authority = authority.split(['/', '?', '#']).next().unwrap_or(authority);
        let s = authority.trim().to_ascii_lowercase();
        match s.rsplit_once(':') {
            // A colon with only digits after it is a port; anything else is
            // part of the host (or the IPv6 case above).
            Some((host, port)) if !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) => {
                (host.to_string(), Some(port.to_string()))
            }
            _ => (s, None),
        }
    }
}
