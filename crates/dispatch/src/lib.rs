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
use std::time::{SystemTime, UNIX_EPOCH};

use castellan_protocol::{
    OtpImportCandidate, RpcError, RpcErrorCode, RpcMethod, RpcRequest, RpcResponse, RpcResult,
    VaultStatus,
};
use castellan_vault::{LockReason, TotpImport, VaultError, VaultSession};

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
            RpcMethod::GetTotp { entry_id } => self.get_totp(&entry_id),
            RpcMethod::PreviewOtpImport { payload } => {
                // Pure parsing, deliberately answered even when locked:
                // reviewing what a QR carries stores nothing and reads
                // no vault state, and an import is most naturally
                // prepared right after scanning, whatever the lock says.
                match castellan_otp::import::preview(&payload) {
                    Ok(accounts) => Ok(RpcResult::PreviewOtpImport {
                        accounts: accounts
                            .into_iter()
                            .map(|account| OtpImportCandidate {
                                issuer: account.issuer,
                                account: account.account,
                                otpauth: account.otpauth,
                                problem: account.problem,
                            })
                            .collect(),
                    }),
                    Err(error) => Err(bad_request(&error.to_string())),
                }
            }
            RpcMethod::ImportOtpAccounts { accounts } => self.import_otp_accounts(accounts),
            RpcMethod::SaveEntry { .. } => {
                // Locked comes first: "enter your password" is the answer
                // the fill UI needs, not "this build cannot". Unlocked,
                // this operation is still to come (task-12).
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

impl Dispatcher {
    /// `get_totp`: read the seed from the session, compute through
    /// `castellan-otp`. The period arithmetic happens here, on the app
    /// side of the wire — the UI receives `seconds_remaining` and only
    /// ever draws it (the TotpRing contract).
    fn get_totp(&self, entry_id: &str) -> Result<RpcResult, RpcError> {
        let uri = self
            .session
            .totp_uri(entry_id)
            .map_err(|error| session_error(&error))?;
        // A stored seed this build cannot compute (SHA-256, HOTP) is a
        // capability gap, not a caller mistake: not_implemented, with
        // the parser's message naming what is missing (task-42's scope).
        let config = castellan_otp::parse(&uri)
            .map_err(|error| not_implemented(&format!("cannot compute this seed: {error}")))?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let (code, seconds_remaining) = castellan_otp::code_now(&config, now);
        Ok(RpcResult::GetTotp {
            code,
            seconds_remaining,
        })
    }

    /// `import_otp_accounts`: validate the whole selection, then hand it
    /// to the session as one batch (one copy-aside save, events after).
    /// Validation is all-or-nothing on purpose — a half-imported review
    /// is the state the per-account checkboxes exist to prevent.
    fn import_otp_accounts(
        &self,
        accounts: Vec<castellan_protocol::OtpImportSelection>,
    ) -> Result<RpcResult, RpcError> {
        // Locked first, same precedence rule as every secret-touching
        // operation: nothing can be stored into a locked vault.
        if self.session.status() != VaultStatus::Unlocked {
            return Err(session_error(&VaultError::Locked));
        }
        let mut batch = Vec::with_capacity(accounts.len());
        for account in accounts {
            if let Err(error) = castellan_otp::parse(&account.otpauth) {
                return Err(bad_request(&format!(
                    "account \"{}\" carries an unusable seed: {error}",
                    account.title
                )));
            }
            batch.push(TotpImport {
                title: account.title,
                username: account.username,
                otpauth: account.otpauth,
            });
        }
        let imported = self
            .session
            .import_totp_entries(&batch)
            .map_err(|error| session_error(&error))?;
        Ok(RpcResult::ImportOtpAccounts {
            imported: u32::try_from(imported.len()).unwrap_or(u32::MAX),
        })
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

fn bad_request(why: &str) -> RpcError {
    RpcError {
        code: RpcErrorCode::BadRequest,
        message: why.to_string(),
    }
}
