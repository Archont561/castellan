# `castellan-dispatch`

The transport-independent application dispatcher. Desktop and mobile Tauri
commands, the local IPC server, native messaging, and the future CLI all hand
one protocol envelope to `Dispatcher`; adapters move bytes or invoke values but
do not implement a second method switch.

## Responsibilities

- Dispatches the RPC operations defined by `castellan-protocol`.
- Connects vault session operations, TOTP calculation/import preview, and
  protocol-shaped results.
- Keeps a face-independent boundary that can be tested with a real in-memory
  vault without a webview or socket.

This crate depends on protocol, vault, and OTP. The reverse dependencies do not
reach back into the dispatcher; that direction keeps the protocol buildable by
any client.

## Verify

Run from the repository root:

```console
$ pixi run cargo nextest run -p castellan-dispatch
$ pixi run cargo test --doc -p castellan-dispatch
$ pixi run cargo clippy -p castellan-dispatch --all-targets -- -D warnings
```
