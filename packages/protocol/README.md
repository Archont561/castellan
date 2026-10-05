# `@castellan/protocol`

The generated TypeScript surface of `castellan-protocol`, the Rust crate that
defines every RPC message, event, association handshake, and connected-browser
panel shape. It intentionally contains types and constants only: client
behavior belongs in `@castellan/core`, and UI behavior belongs in
`@castellan/ui`.

## Generated source

Everything under `src/generated/` is committed xtask output. Never edit it by
hand: a local type change would diverge from the Rust wire contract and be
overwritten. Instead:

1. change the contract in `crates/protocol`;
2. implement the corresponding dispatcher behavior when needed; and
3. run `pixi run codegen` followed by `pixi run codegen-check`.

Consumers import protocol types from this package's barrel:

```ts
import type { EntrySummary, RpcRequest } from "@castellan/protocol";
```

## Commands

```console
$ pixi run bun run --cwd packages/protocol typecheck
$ pixi run bun run --cwd packages/protocol test
$ pixi run codegen-check
```
