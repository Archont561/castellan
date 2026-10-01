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
pub const PROTOCOL_VERSION: u32 = 1;

/// What a connected face can be asked to do.
///
/// Sent in [`Hello`] so the *client* can adapt its UI before the first
/// request, rather than discovering a missing capability by error path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
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

/// The vault, as a client sees it from the outside.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum VaultStatus {
    /// A database is open and unlocked.
    Unlocked,
    /// A database is loaded but locked; requests that need secrets fail with
    /// [`crate::VAULT_LOCKED`].
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

/// One RPC call, tagged by method name.
///
/// Internally tagged (`{"method": "get_entries", ...}`) so a method is
/// readable on the wire and a typo is a parse error, not a silent dispatch
/// miss. The tag strings stay snake_case on the wire even though the variants
/// are Rust-idiomatic — rename attributes are part of the derived TypeScript,
/// not a second hand-maintained mapping.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "method", rename_all = "snake_case")]
#[ts(export)]
pub enum RpcMethod {
    /// Entries relevant to an origin, for the fill UI.
    GetEntries {
        /// The page origin the fill is happening on.
        origin: String,
    },
    /// The current TOTP code for one entry.
    GetTotp {
        /// Which entry to compute a code for.
        entry_id: String,
    },
    /// A generated passphrase (word count and separator chosen by the UI).
    GeneratePassphrase {
        /// How many words to join.
        words: u32,
        /// What to join them with.
        separator: String,
    },
    /// Save an entry captured by the extension's save prompt.
    SaveEntry {
        /// The captured entry.
        entry: NewEntry,
    },
    /// Lock the vault now. Always available; never fails.
    LockDatabase,
    /// Liveness probe. Answers with [`RpcResult::Ok`].
    Ping,
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

/// The machine-readable half of a failure. Codes are stable strings the UI
/// can switch on; the message is for humans.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct RpcError {
    /// Stable, machine-readable error code.
    pub code: String,
    /// Human-readable explanation.
    pub message: String,
}

/// Error code: the vault exists but is locked.
pub const VAULT_LOCKED: &str = "vault_locked";

/// Error code: the request named an entry that does not exist. Distinct from
/// an empty list so the UI can say "nothing for this site" without guessing
/// whether it queried the right database.
pub const NO_SUCH_ENTRY: &str = "no_such_entry";

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

/// The payload of a successful call, tagged by result kind.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(tag = "type", rename_all = "snake_case")]
#[ts(export)]
pub enum RpcResult {
    /// Nothing to return; the call just worked.
    Ok,
    /// Answer to `get_entries`.
    Entries {
        /// Entries relevant to the requested origin.
        entries: Vec<EntrySummary>,
    },
    /// Answer to `get_totp`.
    Totp {
        /// The code, right-aligned and zero-padded as services expect it.
        code: String,
        /// Seconds until this code expires; the UI draws its progress ring
        /// from this and never re-implements the period arithmetic.
        seconds_remaining: u32,
    },
    /// Answer to `generate_passphrase`.
    Passphrase {
        /// The generated passphrase.
        value: String,
    },
    /// Answer to `save_entry`.
    Saved {
        /// The id the saved entry received.
        id: String,
    },
}

/// Events the app pushes at connected faces. Lock state is pushed, not
/// polled: the extension's icon reflects the vault *live*, and the stale
/// "reconnect to your database" class of failure cannot exist because there
/// is no state to go stale.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
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
    /// The answer to a client's hello.
    Hello {
        /// The app's handshake.
        hello: Hello,
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
