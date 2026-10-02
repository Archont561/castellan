//! The Castellan mobile application's Tauri shell.
//!
//! This crate adapts Tauri's `invoke` boundary to the shared
//! `castellan-dispatch` crate. Platform plugins and lifecycle stay here;
//! protocol operations execute in the same dispatcher as every other face.

use castellan_protocol::{RpcRequest, RpcResponse};

/// The single Tauri command the frontend may call.
///
/// The capability file grants `core:default` and nothing else: one surface,
/// one review. Adding a protocol operation changes the shared contract and
/// dispatcher, never Tauri's command allowlist.
#[tauri::command]
fn rpc(request: RpcRequest) -> RpcResponse {
    castellan_dispatch::dispatch(request)
}

/// Builds and runs the mobile Tauri application.
///
/// # Panics
///
/// Panics if the webview runtime cannot start—there is no usable fallback for
/// a GUI app whose window never opens.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![rpc])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
