//! The transport-independent Castellan RPC dispatcher.
//!
//! Desktop's Tauri command, mobile's Tauri command, the native-messaging
//! socket and the future CLI all pass the same [`RpcRequest`] through
//! [`Dispatcher::dispatch`]. Platform crates own lifecycle and transport
//! only; an action is implemented here once and every face selected by the
//! protocol contract reaches that implementation.
//!
//! Since task-7 the dispatcher is session-aware: it holds the one
//! [`VaultSession`] and answers from it — locked vaults fail with
//! `vault_locked` (which takes precedence over `not_implemented`: the
//! first fact a caller needs is that there is nothing to compute *from*),
//! and `lock_database` locks the session the faces share.

use std::sync::Arc;

use castellan_protocol::{
    RpcError, RpcErrorCode, RpcMethod, RpcRequest, RpcResponse, RpcResult, VaultStatus,
};
use castellan_vault::{LockReason, VaultError, VaultSession};

/// The dispatcher: one per app, over the app's one vault session.
#[derive(Clone, Debug)]
pub struct Dispatcher {
    session: Arc<VaultSession>,
}

impl Dispatcher {
    /// Build the dispatcher over the session it will answer from. The
    /// apps construct the session once, share it with this dispatcher and
    /// their event wiring, and hold both for the process lifetime.
    #[must_use]
    pub fn new(session: Arc<VaultSession>) -> Self {
        Self { session }
    }

    /// The session this dispatcher answers from — the handle the app's
    /// unlock command and event wiring need.
    #[must_use]
    pub fn session(&self) -> &VaultSession {
        &self.session
    }

    /// Execute one protocol request and preserve its correlation id.
    ///
    /// A recognized operation always becomes an [`RpcResponse`], including
    /// an application failure. Failure to deserialize an unknown operation
    /// happens at the transport boundary before this function is called.
    pub fn dispatch(&self, request: RpcRequest) -> RpcResponse {
        let RpcRequest { id, method } = request;
        let result = match method {
            RpcMethod::Ping {} => Ok(RpcResult::Ping {}),
            RpcMethod::GetEntries { .. } => self
                .session
                .entries()
                .map(|entries| RpcResult::GetEntries { entries })
                .map_err(|error| session_error(&error)),
            RpcMethod::GeneratePassphrase { words, separator } => {
                let words = words.clamp(1, 16) as usize;
                Ok(RpcResult::GeneratePassphrase {
                    value: castellan_vault::passphrase(words, &separator),
                })
            }
            RpcMethod::LockDatabase {} => {
                self.session.lock(LockReason::User);
                Ok(RpcResult::LockDatabase {})
            }
            RpcMethod::GetTotp { .. } | RpcMethod::SaveEntry { .. } => {
                // Locked comes first: "enter your password" is the answer
                // the fill UI needs, not "this build cannot". Unlocked,
                // these operations are still to come (task-13, task-12).
                if self.session.status() != VaultStatus::Unlocked {
                    Err(session_error(&VaultError::Locked))
                } else {
                    Err(not_implemented("this build fills passphrases only"))
                }
            }
        };

        match result {
            Ok(result) => RpcResponse {
                id,
                result: Some(result),
                error: None,
            },
            Err(error) => RpcResponse {
                id,
                result: None,
                error: Some(error),
            },
        }
    }
}

/// A vault error as the wire sees it: the stable code when the protocol
/// has one for it. The fallback is unreachable from every current call
/// path (a save failure has no RPC surface yet); it exists so a new
/// [`VaultError`] variant cannot compile into a dispatcher that forgets
/// to map it.
fn session_error(error: &VaultError) -> RpcError {
    let code = error.error_code().unwrap_or(RpcErrorCode::NotImplemented);
    RpcError {
        code,
        message: error.to_string(),
    }
}

fn not_implemented(what: &str) -> RpcError {
    RpcError {
        code: RpcErrorCode::NotImplemented,
        message: format!("not implemented yet: {what}"),
    }
}
