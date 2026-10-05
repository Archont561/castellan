# `@castellan/tauri`

The shared Tauri adapter for the desktop and mobile faces. It implements
`@castellan/core`'s `Transport` seam over one Tauri `rpc` command and the
`castellan://event` event channel, so both native faces have identical invoke,
disconnection, and event-subscription behavior.

## Public surface

- `createTauriTransport(invoke, listen)` builds an injectable transport for
  tests or alternate Tauri bindings.
- `tauriTransport()` returns the production adapter and reports a stable
  disconnected error when rendered in a browser preview without Tauri.
- `TauriInvoke` and `TauriListen` describe the narrow injectable platform
  boundaries.

The Rust shells forward a whole `RpcRequest` envelope to the shared dispatcher;
adding a protocol operation must not create another Tauri command.

## Commands

```console
$ pixi run bun run --cwd packages/tauri test
$ pixi run bun run --cwd packages/tauri typecheck
$ pixi run bun run --cwd packages/tauri lint
```
