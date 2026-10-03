//! The Castellan mobile application's Tauri shell.
//!
//! Platform plugins and lifecycle stay here; protocol operations execute
//! in the same dispatcher as every other face, over the same one session.
//!
//! Beyond the single `rpc` door, the shell owns what cannot cross the
//! protocol: the master password (unlocked in, never round-tripped), the
//! status the unlock face renders from, the lock tiers' heartbeat, and
//! the forwarding of session events onto the webview event bus.

use std::{path::PathBuf, sync::Arc, time::Duration};

use castellan_dispatch::Dispatcher;
use castellan_protocol::{Event, RpcErrorCode, RpcRequest, RpcResponse, VaultStatus};
use castellan_vault::{LockPolicy, LockReason, LockTrigger, VaultSession};
use tauri::Emitter;

/// The event channel every face listens on for protocol [`Event`] values;
/// the shared transport package's `onEvent` subscribes to the same name.
const EVENT_CHANNEL: &str = "castellan://event";

/// The single Tauri command the frontend may call for protocol traffic.
///
/// The capability file grants `core:default` and nothing else: one surface,
/// one review. Adding a protocol operation changes the shared contract and
/// dispatcher, never Tauri's command allowlist.
#[tauri::command]
fn rpc(dispatcher: tauri::State<'_, Dispatcher>, request: RpcRequest) -> RpcResponse {
    dispatcher.dispatch(request)
}

/// Unlock a vault into the session.
///
/// The password crosses this boundary once and dies with the call; the
/// derived keys live only inside the session. Failure is the stable
/// protocol error code — `bad_credentials` for a wrong password or
/// keyfile, `vault_unreadable` for a file that is not a vault this build
/// understands — so the unlock face never parses error strings.
#[tauri::command]
fn unlock_vault(
    dispatcher: tauri::State<'_, Dispatcher>,
    path: PathBuf,
    password: Option<String>,
    keyfile: Option<PathBuf>,
) -> Result<VaultStatus, RpcErrorCode> {
    dispatcher
        .session()
        .unlock(&path, password.as_deref(), keyfile.as_deref())
        .map_err(|error| error.error_code().unwrap_or(RpcErrorCode::VaultUnreadable))
}

/// Lock the session now, the user-action way. The RPC `LockDatabase`
/// operation reaches the same session lock through the dispatcher; one
/// lock path, two doors.
#[tauri::command]
fn lock_vault(dispatcher: tauri::State<'_, Dispatcher>) -> bool {
    dispatcher.session().lock(LockReason::User)
}

/// The session's state, for the unlock face's first render.
#[tauri::command]
fn vault_status(dispatcher: tauri::State<'_, Dispatcher>) -> VaultStatus {
    dispatcher.session().status()
}

/// The lock tiers' heartbeat: once a second the session decides, by its
/// own clock, whether the idle window or the hard-lock horizon has
/// elapsed. Blur and wake triggers arrive as they happen once the faces
/// land; this tick is the floor that needs no listener.
fn spawn_lock_ticker(session: Arc<VaultSession>) {
    std::thread::spawn(move || {
        loop {
            let policy = LockPolicy::default();
            session.auto_lock(&policy, LockTrigger::Idle);
            session.hard_lock_if_due(&policy);
            std::thread::sleep(Duration::from_secs(1));
        }
    });
}

/// Builds and runs the mobile Tauri application.
///
/// # Panics
///
/// Panics if the webview runtime cannot start—there is no usable fallback for
/// a GUI app whose window never opens.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // One session for the whole app: every command, the dispatcher, the
    // event forwarder and the lock ticker look at the same lock state.
    let session = Arc::new(VaultSession::new());
    let dispatcher = Dispatcher::new(session.clone());

    tauri::Builder::default()
        .manage(dispatcher)
        .setup({
            let session = session.clone();
            move |app| {
                // Session events land on the webview event bus so faces
                // reflect lock state live (task-7's event-bus promise).
                // Emitting only fails when the window is gone, which a
                // missing listener for one event does not make fatal.
                let handle = app.handle().clone();
                session.subscribe(Arc::new(move |event: &Event| {
                    let _ = handle.emit(EVENT_CHANNEL, event.clone());
                }));

                spawn_lock_ticker(session.clone());
                Ok(())
            }
        })
        .invoke_handler(tauri::generate_handler![
            rpc,
            unlock_vault,
            lock_vault,
            vault_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
