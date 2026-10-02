# The Rust workspace

One Cargo workspace, the repository root as its manifest, every member under
`crates/` except the two Tauri app crates (which live beside their frontends
in `apps/*/src-tauri`). Turbo sees the whole thing as one node in the JS task
graph through the `package.json` in this directory (`@castellan/rust`) —
`bun run test` runs every suite once, not once per package.

## The rule that shapes the layout

`castellan-protocol` is the API and depends on nothing in the workspace; the
crates that *do* things depend on it, never the other way around. A protocol
that needs the engine to explain itself is a protocol no client can be built
against ahead of the app.

## Layout

| Crate | What it is |
| --- | --- |
| `protocol/` | the RPC contract macro, events, capability negotiation and origin matching. It drives generated wire types and face-scoped clients |
| `dispatch/` | the one transport-independent RPC dispatcher; desktop, mobile, native messaging and the future CLI terminate here |
| `otp/` | otpauth parsing + RFC 6238 TOTP (SHA-1; other algorithms refused with clear errors, not wrong codes) |
| `vault/` | KDBX core: open, entry projection, passphrase generation. Save = copy-aside-then-write, because keepass-rs writing is experimental |
| `ipc/` | the one native channel: 4-byte LE framing (Chromium's native-messaging wire format, byte for byte) + the well-known socket path. std-only |
| `native-host/` | the process each browser spawns: a byte pump between the browser's stdio and the app's socket. Parses nothing, so it can never be the wrong version |
| `wasm/` | the WASM face of shared logic (`castellan-otp` + `protocol::matching`) — what the extension runs, the apps link natively |
| `xtask/` | code generation: derives TypeScript bindings plus desktop/mobile/web-extension clients from `protocol` |
| `apps/desktop/src-tauri` | the desktop app crate: a Tauri `rpc` adapter into `castellan-dispatch` |
| `apps/mobile/src-tauri` | the mobile app crate: the same thin adapter plus platform lifecycle/plugins |

Each crate keeps its tests in its own `tests/` directory — integration
tests over the public API, one file per theme (`framing.rs`,
`origin_matching.rs`, …). Two things stay as `#[cfg(test)]` in the source:
the ts-rs export checks (derive-generated, they live in the lib by
construction) and a bin crate's unit tests (xtask's version-parsing tests
— a bin exports nothing an integration test could import). `cargo test
--workspace` (the façade's `test` script) runs them all.

## Commands

Everything runs from the repository root; the façade scripts `cd ..` first
because that is where the workspace manifest lives.

```console
$ cargo test --workspace      # every suite
$ cargo clippy --workspace --all-targets -- -D warnings
$ cargo fmt --all
$ cargo run -p castellan-xtask -- codegen   # regenerate the TS bindings
$ cargo deny --workspace check bans licenses sources
```

`build` and `typecheck` (`cargo check --workspace`) include the Tauri app
crates and therefore need the platform's webview libraries (on Linux:
webkit2gtk-4.1 and friends — see apps/desktop/README.md).
