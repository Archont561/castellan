//! The transport-independent Castellan RPC dispatcher.
//!
//! Desktop's Tauri command, mobile's Tauri command, the native-messaging
//! socket and the future CLI all pass the same [`RpcRequest`] through
//! [`dispatch`]. Platform crates own lifecycle and transport only; an action
//! is implemented here once and every face selected by the protocol contract
//! reaches that implementation.

use castellan_protocol::{
    EntrySummary, RpcError, RpcErrorCode, RpcMethod, RpcRequest, RpcResponse, RpcResult,
};

/// Execute one protocol request and preserve its correlation id.
///
/// A recognized operation always becomes an [`RpcResponse`], including an
/// application failure. Failure to deserialize an unknown operation happens
/// at the transport boundary before this function is called.
#[must_use]
pub fn dispatch(request: RpcRequest) -> RpcResponse {
    let RpcRequest { id, method } = request;
    let result = match method {
        RpcMethod::Ping {} => Ok(RpcResult::Ping {}),
        RpcMethod::GetEntries { .. } => {
            // No open-vault session until that state machine lands; an empty
            // collection is the honest answer for the scaffold's empty vault.
            Ok(RpcResult::GetEntries {
                entries: Vec::<EntrySummary>::new(),
            })
        }
        RpcMethod::GeneratePassphrase { words, separator } => {
            let words = words.clamp(1, 16) as usize;
            Ok(RpcResult::GeneratePassphrase {
                value: castellan_vault::passphrase(words, &separator),
            })
        }
        RpcMethod::LockDatabase {} => Ok(RpcResult::LockDatabase {}),
        RpcMethod::GetTotp { .. } | RpcMethod::SaveEntry { .. } => {
            Err(not_implemented("this build fills passphrases only"))
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

fn not_implemented(what: &str) -> RpcError {
    RpcError {
        code: RpcErrorCode::NotImplemented,
        message: format!("not implemented yet: {what}"),
    }
}
