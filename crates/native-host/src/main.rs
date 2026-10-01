//! The native-messaging host: the process each browser spawns.
//!
//! Deliberately, this is not a separate proxy binary — it is the app's own
//! binary, reached via `castellan --native-host` (see the desktop app's
//! arg handling when the CLI crate lands; for now this crate builds the same
//! program standalone). KeePass's classic breakage was version drift between
//! app, proxy and extension; a host that *is* the app can never be the wrong
//! version for it.
//!
//! The whole job is pumping bytes in both directions:
//!
//! ```text
//!   browser stdin ──frame──▶ IPC socket
//!   IPC socket   ──frame──▶ browser stdout
//! ```
//!
//! It parses nothing. Frames are opaque; the app on the other end of the
//! socket owns the protocol. That keeps the trusted surface of this process
//! at two syscalls and a buffer, which is the kind of small that stays
//! auditable.
//!
//! Windows: named-pipe support lands with the first Windows build; until
//! then the host exits with a clear message rather than failing obscurely.

use std::io::Write;
use std::net::Shutdown;
use std::os::unix::net::UnixStream;
use std::process::ExitCode;

use castellan_ipc::{decode_frame, encode_frame, socket_path};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            // stderr is invisible to the browser and precious to whoever is
            // debugging; it is the only diagnostics channel a host has.
            eprintln!("castellan-native-host: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let path = socket_path();
    let mut socket = UnixStream::connect(&path).map_err(|e| {
        format!(
            "cannot connect to {} (is the Castellan app running?): {e}",
            path.display()
        )
    })?;

    // Direction 1: browser → app, on its own thread, so a slow or idle
    // socket read can never stall direction 2.
    let writer = socket
        .try_clone()
        .map_err(|e| format!("duplicating the socket: {e}"))?;
    std::thread::spawn(move || pump_browser_to_app(writer));

    // Direction 2: app → browser, on the main thread. When the app closes
    // the socket the read fails, the loop ends, and the process exits —
    // which is exactly what "the app quit" should mean for the browser.
    pump_app_to_browser(&mut socket);
    let _ = socket.shutdown(Shutdown::Both);
    Ok(())
}

/// stdin → socket, one frame at a time. stdin EOF means the browser went
/// away; close the socket's write half so the app notices.
fn pump_browser_to_app(mut socket: UnixStream) {
    let mut stdin = std::io::stdin().lock();
    while let Ok(frame) = decode_frame(&mut stdin) {
        let mut wire = Vec::with_capacity(frame.len() + 4);
        encode_frame(&frame, &mut wire);
        if socket.write_all(&wire).is_err() || socket.flush().is_err() {
            break;
        }
    }
    let _ = socket.shutdown(Shutdown::Write);
}

/// socket → stdout, one frame at a time.
fn pump_app_to_browser(socket: &mut UnixStream) {
    let mut stdout = std::io::stdout().lock();
    while let Ok(frame) = decode_frame(socket) {
        let mut wire = Vec::with_capacity(frame.len() + 4);
        encode_frame(&frame, &mut wire);
        if stdout.write_all(&wire).is_err() || stdout.flush().is_err() {
            break;
        }
    }
}
