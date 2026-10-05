# `@castellan/core`

Transport-independent RPC client mechanics shared by the generated desktop,
mobile, and extension clients. This package owns the invariants every face
must share—request IDs, response/result correlation, stable errors, and event
subscription—without importing Tauri or browser globals.

## Public surface

- `RpcClient` is the base class generated face clients extend.
- `Transport` is the physical-request seam implemented by `@castellan/tauri`
  and the extension's native-messaging adapter.
- `CastellanError`, `DISCONNECTED`, and `disconnected()` give every face the
  same error vocabulary.
- `RpcMethodName`, `RpcParams`, and `RpcResultFor` derive method-specific
  types from `@castellan/protocol`.

A transport implementation must pair responses by request ID, fan pushed
events out to subscribers, reject disconnections rather than hang, and permit
a later reconnect. Platform adapters belong outside this package.

## Commands

From the repository root:

```console
$ pixi run bun run --cwd packages/core test
$ pixi run bun run --cwd packages/core typecheck
$ pixi run bun run --cwd packages/core lint
$ pixi run bun run --cwd packages/core build
```

For protocol changes, edit `crates/protocol` and run `pixi run codegen`; do not
hand-author face methods here.
