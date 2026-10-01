---
type: Research
title: "Castellan — WASM bindings: wasm-bindgen vs napi-rs"
description: "Why the extension's wasm face stays on wasm-bindgen/wasm-pack, and the trigger that would reopen napi-rs (v3)."
tags:
  - wasm
  - extension
  - architecture
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T19:40:00Z"
created: "2026-10-01T19:40:00Z"
updated: "2026-10-01T19:40:00Z"
id: research/wasm-bindings
category: research
refs:
  - infrastructure/monorepo
  - infrastructure/codegen
---
# WASM bindings: wasm-bindgen vs napi-rs (as of 2026-10-01)

**Question:** should `crates/wasm`'s bindings move from wasm-bindgen/wasm-pack
to napi-rs (v3), whose wasm target promises one Rust API for Node and the
browser?

**Verdict: no.** napi-rs v3's wasm support is real but aimed at a different
problem. Facts from the napi-rs docs and v3 announcement, checked 2026-10-01:

1. **The browser target needs cross-origin isolation.** napi-rs compiles to
   `wasm32-wasip1-threads`; the browser loader requires shared memory, so
   their own support matrix marks "browser without isolation: unsupported".
   Castellan's primary wasm consumer is the extension's MV3 service worker,
   which is not cross-origin isolated — the target row of the matrix is
   exactly us.
2. **The payload is WASI + emnapi, not minimal glue.** The browser entry
   pulls `@napi-rs/wasm-runtime` (+ `@emnapi/runtime`/`@emnapi/core`, with
   version-alignment constraints), fetches the .wasm with top-level await,
   and carries WASI/threads machinery. wasm-bindgen's `--target bundler`
   output is the purpose-built minimal path for code that loads in a
   service worker on every browser start; serde-wasm-bindgen interop is
   already wired and tested.
3. **The benefit has no consumer here.** napi-rs's whole value is ONE
   `#[napi]` surface serving native Node addons AND wasm. Castellan has
   zero Node faces: the apps reach the crates natively through Tauri
   commands (not N-API), the extension through the browser, the future CLI
   is a Rust binary. There is nothing for the N-API half to attach to.
4. **It would not even remove our one wart.** The bun test pain is bun's
   missing wasm-ESM interop for wasm-bindgen output (worked around by
   manual instantiation in `packages/wasm/test`). napi-rs's own docs flag
   Bun/Deno WASI loading as an open incompatibility — switching trades a
   solved problem for an unsolved one.
5. **Our wasm surface uses none of what WASI buys.** The crate is
   deliberately tiny, synchronous, pure logic (otpauth parsing, origin
   matching); no fs, no threads, no tokio. `wasm32-unknown-unknown` is the
   honest target for that.

**When this reopens:** a Node/Bun SDK face — e.g. the 1.0 dev tools
shipping as a Node CLI that wants native speed with prebuilt per-platform
binaries and the wasm fallback from one bindings layer. That is additive:
a `#[napi]` layer over the same crates next to the wasm-bindgen one, not a
replacement. Nothing in the current seams blocks adding it.

Sources: napi.rs "WebAssembly and WASI" docs (support matrix, browser
requirements, runtime packages), napi.rs "Announcing NAPI-RS v3"
(2025-07), both checked 2026-10-01.
