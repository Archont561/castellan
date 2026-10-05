# `castellan-ipc-server`

The app-side local IPC server. It accepts browser-native-host connections,
checks same-user identity, multiplexes RPC requests, performs the association
handshake, and exposes the data rendered by the connected-browsers panel.

## Responsibilities

- Owns `IpcServer` plus injectable request, hello, and change callbacks.
- Stores/forgets association keys and coordinates enrollment decisions.
- Issues nonce challenges and verifies HMAC proofs; unassociated or refused
  clients receive silence rather than a protocol oracle.
- Publishes `PanelSnapshot`, `ConnectionInfo`, and related protocol data for
  the desktop/mobile settings surface.
- Uses Unix peer credentials (`SO_PEERCRED`/`getpeereid`) for same-user access;
  a platform stub preserves the public shape where the native server is not
  available.

The server does not own vault behavior. Its `RequestHandler` boundary sends an
already-authenticated request to the shared dispatcher.

## Verify

```console
$ pixi run cargo nextest run -p castellan-ipc-server
$ pixi run cargo test --doc -p castellan-ipc-server
$ pixi run cargo clippy -p castellan-ipc-server --all-targets -- -D warnings
```
