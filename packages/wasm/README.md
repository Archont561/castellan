# `@castellan/wasm`

The TypeScript wrapper around shared Rust logic compiled for browser use. The
extension imports it for otpauth preview/validation and origin matching instead
of reimplementing security-sensitive rules in TypeScript. The native apps link
the same Rust functions directly.

## Public surface

- `loadWasm()` lazily imports and caches the wasm-bindgen module.
- `protocolVersion()` returns the shared-logic protocol version.
- `parseOtpauth(uri)` returns a secret-free `OtpAuthInfo` projection.
- `originMatches(origin, candidate)` uses the same rule as the app.

The package deliberately has no vault, filesystem, socket, or secret-bearing
API. If `src/pkg/` artifacts are absent, the loader names the required build
command instead of silently falling back to TypeScript.

## Build and verify

```console
$ pixi run wasm
$ pixi run bun run --cwd packages/wasm typecheck
$ pixi run bun run --cwd packages/wasm test
```

`pixi run wasm` runs `wasm-pack` against `crates/wasm`; JavaScript/WASM output
is ignored while generated declarations remain committed so typechecking works
before a local WASM build.
