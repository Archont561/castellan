//! The Castellan mobile app: the Tauri shell plus the RPC dispatcher.
//!
//! The frontend sees exactly one command — `rpc` — which takes a
//! [`RpcRequest`] and returns an [`RpcResponse`]. That indirection is the
//! whole architecture: the SvelteKit app, the browser extension and a future
//! CLI all speak the same protocol, so the fill logic, the TOTP logic and the
//! passphrase logic are written once, here, behind the same types
//! `castellan-protocol` derives the TypeScript from.
//!
//! What is deliberately *not* here yet: the IPC socket server (the half that
//! talks to `castellan-native-host` and the extension). It lands in this
//! crate, multiplexing sessions over `castellan-ipc`, with the app's
//! "connected browsers" panel as its UI. The stubs below keep the protocol
//! honest from day one: every method answers, even if the answer is
//! `not_implemented`, so no face ever sees a parse error for a method the
//! protocol defines.

use castellan_protocol::{
    EntrySummary, NewEntry, RpcError, RpcMethod, RpcRequest, RpcResponse, RpcResult,
};

/// Error code for methods the protocol defines but this build does not fill
/// in yet. A client can distinguish "not yet" from "never heard of it"
/// because an unknown method fails to *deserialize*, before dispatch runs.
pub const NOT_IMPLEMENTED: &str = "not_implemented";

/// The single Tauri command the frontend may call.
///
/// The capability file grants `core:default` and nothing else: one surface,
/// one review. Adding a second command means arguing for it in a diff that
/// also touches `capabilities/default.json`, which is the point.
#[tauri::command]
fn rpc(request: RpcRequest) -> Result<RpcResponse, String> {
    dispatch(request).map_err(|error| error.message)
}

/// Where a method name becomes an action.
///
/// Free function, not a method on some service object, so the mobile app's
/// dispatcher (identical today) can call into a shared crate the moment the
/// two grow apart — moving this body to `castellan-vault` or a new
/// `castellan-dispatch` is a mechanical refactor with no Tauri types in the
/// way.
pub fn dispatch(request: RpcRequest) -> Result<RpcResponse, RpcError> {
    let result = match request.method {
        RpcMethod::Ping => Ok(RpcResult::Ok),
        RpcMethod::GetEntries { .. } => {
            // No vault until the KDBX core lands; an empty list is the
            // honest answer for "a database with nothing in it yet".
            Ok(RpcResult::Entries {
                entries: Vec::<EntrySummary>::new(),
            })
        }
        RpcMethod::GeneratePassphrase { words, separator } => {
            let words = words.clamp(1, 16) as usize;
            Ok(RpcResult::Passphrase {
                value: castellan_vault::passphrase(words, &separator),
            })
        }
        RpcMethod::LockDatabase => Ok(RpcResult::Ok),
        RpcMethod::GetTotp { .. } | RpcMethod::SaveEntry { .. } => {
            Err(not_implemented("this build fills passphrases only"))
        }
    };

    match result {
        Ok(result) => Ok(RpcResponse {
            id: request.id,
            result: Some(result),
            error: None,
        }),
        Err(error) => Ok(RpcResponse {
            id: request.id,
            result: None,
            error: Some(error),
        }),
    }
}

fn not_implemented(what: &str) -> RpcError {
    RpcError {
        code: NOT_IMPLEMENTED.into(),
        message: format!("not implemented yet: {what}"),
    }
}

/// Unused parameters exist so this compiles against the full protocol shape;
/// the entries and new-entry types are what the vault core will consume.
#[allow(dead_code)]
fn reserved(_entries: &[EntrySummary], _new: &NewEntry) {}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![rpc])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
