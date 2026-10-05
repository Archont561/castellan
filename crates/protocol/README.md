# `castellan-protocol`

The one message language every Castellan face speaks. It owns the RPC contract,
result/error envelopes, events, association/native-message shapes, panel data,
origin matching, and the metadata that constrains an operation to its allowed
client faces.

## Change the contract here

`rpc_contract!` is the source of truth for operations. To add or modify one:

1. change the Rust contract in this crate;
2. implement the behavior once in `castellan-dispatch`;
3. run `pixi run codegen`; and
4. commit the generated TypeScript types and face-scoped clients.

Do not hand-edit `packages/protocol/src/generated` or an app's generated
client. This crate depends on no workspace behavior crate, so a new client can
be built against the contract without pulling in a vault or app shell.

## Verify

```console
$ pixi run cargo nextest run -p castellan-protocol
$ pixi run cargo test --doc -p castellan-protocol
$ pixi run codegen-check
```
