# `castellan-ipc`

The smallest common layer of Castellan's native channel. It defines the
well-known local socket address and the 4-byte little-endian framed byte format
shared by the app-side server and the browser-spawned native host—the same
framing Chromium native messaging uses.

## Public API

- `socket_path()` resolves the per-user Unix socket path.
- `encode_frame()` and `decode_frame()` write/read bounded length-prefixed
  frames.
- `MAX_MESSAGE_BYTES` limits a single message to 1 MiB.

The crate is intentionally std-only: no async runtime, protocol semantics, or
socket server behavior belongs here. It can therefore be reused by both halves
of the channel without creating a runtime dependency boundary.

## Verify

```console
$ pixi run cargo nextest run -p castellan-ipc
$ pixi run cargo test --doc -p castellan-ipc
$ pixi run cargo clippy -p castellan-ipc --all-targets -- -D warnings
```
