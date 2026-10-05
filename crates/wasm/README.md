# `castellan-wasm`

The browser-compiled face of shared Rust logic. It exposes only operations the
extension may safely run without the app: protocol-version reporting,
secret-free otpauth preview/validation, and origin matching. The same ordinary
Rust functions are linked natively by the app-side crates.

## Boundary

The WASM module must not pull in a vault, filesystem, IPC socket, or raw TOTP
seed. `OtpAuthInfo` mirrors parsed metadata without secret bytes, and live
codes continue to come from the native app through the protocol.

`wasm-pack` compiles this crate into `packages/wasm/src/pkg/`; the TypeScript
wrapper provides the lazy public API used by the extension. Release `wasm-opt`
is intentionally disabled because Binaryen is not part of the locked offline
toolchain.

## Verify and build

```console
$ pixi run cargo nextest run -p castellan-wasm
$ pixi run cargo test --doc -p castellan-wasm
$ pixi run wasm
```
