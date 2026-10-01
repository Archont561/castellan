//! The one native channel: message framing and the well-known address.
//!
//! Every browser's native-messaging host connects here, and the CLI will too.
//! One socket, many clients, one protocol — the app multiplexes sessions and
//! shows the connected faces in its UI, so a broken integration is visible
//! rather than silent (the failure mode that made KeePass browser
//! integration a troubleshooting genre).
//!
//! Framing mirrors Chromium's native-messaging wire format exactly — 4-byte
//! little-endian length prefix, then the payload — so the host process is a
//! *byte pump*: it never parses a message, it just moves frames between the
//! browser's stdio and this socket. That is the whole reason the host can be
//! the app binary itself (`castellan --native-host`), and the reason a host
//! can never be "the wrong version" for the app.
//!
//! std-only on purpose: no tokio, no serde, nothing. A framing layer that
//! depends on an async runtime is a framing layer the host binary has to
//! drag along for no reason.

use std::io::Read;
use std::path::PathBuf;

/// The cap Chromium puts on messages a host may send *to* the browser
/// (~1 MB). The app paginates rather than ever approaching it; the constant
/// lives here so the check is enforced where the bytes actually flow.
pub const MAX_MESSAGE_BYTES: u32 = 1024 * 1024;

/// The well-known socket path on this platform.
///
/// Unix: `$XDG_RUNTIME_DIR/castellan/castellan.sock`, falling back to
/// `/tmp/castellan-<uid>/` (mode 0700) when there is no runtime dir — the
/// fallback is created by the app, not by this function; this only names it.
/// Windows: a named pipe. The name is fixed so the host does not need to be
/// told where to connect; per-user isolation on Windows comes from the
/// pipe's security descriptor, not from the name.
pub fn socket_path() -> PathBuf {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        if let Some(dir) = std::env::var_os("XDG_RUNTIME_DIR") {
            return PathBuf::from(dir).join("castellan").join("castellan.sock");
        }
        let uid = nix_like_uid();
        let mut name = std::ffi::OsString::from_vec(Vec::new());
        name.push(format!("/tmp/castellan-{uid}/castellan.sock"));
        PathBuf::from(name)
    }
    #[cfg(windows)]
    {
        PathBuf::from(r"\\.\pipe\castellan")
    }
}

/// The UID, without depending on libc. `/proc/self` carries it on Linux; on
/// the other Unixes the runtime dir is set in practice, so the fallback path
/// is a name, not a promise.
#[cfg(unix)]
fn nix_like_uid() -> u32 {
    std::fs::read_link("/proc/self")
        .ok()
        .and_then(|p| p.to_str().and_then(|s| s.rsplit('/').next()?.parse().ok()))
        .unwrap_or(0)
}

/// Append a framed message to `out`: 4-byte LE length, then the payload.
///
/// Allocation-free from the caller's point of view; used by both the host
/// (writing to stdout buffers) and the app's socket writer.
pub fn encode_frame(payload: &[u8], out: &mut Vec<u8>) {
    let len = u32::try_from(payload.len()).expect("payload larger than the protocol allows");
    out.extend_from_slice(&len.to_le_bytes());
    out.extend_from_slice(payload);
}

/// Read one framed message from `read`.
///
/// Returns the payload bytes. Errors on EOF (a closed peer is a closed
/// connection, not a zero-length message), on frames larger than
/// [`MAX_MESSAGE_BYTES`], and on frames that are not valid UTF-8 — the
/// channel carries JSON and nothing else, and passing invalid bytes upstream
/// would only move the error somewhere harder to read.
pub fn decode_frame(read: &mut impl Read) -> std::io::Result<Vec<u8>> {
    let mut len_bytes = [0u8; 4];
    read.read_exact(&mut len_bytes)?;
    let len = u32::from_le_bytes(len_bytes);
    if len > MAX_MESSAGE_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("frame of {len} bytes exceeds the {MAX_MESSAGE_BYTES}-byte cap"),
        ));
    }
    let mut payload = vec![0u8; len as usize];
    read.read_exact(&mut payload)?;
    if std::str::from_utf8(&payload).is_err() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "frame payload is not valid UTF-8; the channel carries JSON and nothing else",
        ));
    }
    Ok(payload)
}
