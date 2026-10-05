# Castellan mobile

The mobile face is a Tauri 2 Android/iOS shell around SvelteKit. It shares the
same `rpc` adapter, generated face-scoped client, and `@castellan/tauri`
transport as desktop while owning phone-specific layout, lifecycle, and future
biometric integrations.

## Run and verify

Run from the repository root through Pixi. Generate each platform once on its
host, then use a device or emulator:

```console
$ pixi run bun run --cwd apps/mobile android:init
$ pixi run bun run --cwd apps/mobile android
$ pixi run bun run --cwd apps/mobile ios:init       # macOS only
$ pixi run bun run --cwd apps/mobile ios
$ pixi run bun run --cwd apps/mobile typecheck
$ pixi run bun run --cwd apps/mobile e2e
```

## Layout

| Path | Responsibility |
| --- | --- |
| `src/routes/` | Static SvelteKit shell and touch-oriented composition/metrics |
| `src/generated/client.ts` | Committed xtask output for mobile-allowed RPC methods |
| `src-tauri/` | Tauri workspace member, platform lifecycle, and native adapter |
| `e2e/` | Phone-viewport browser shell tests |

`src/generated/client.ts` is generated from the Rust protocol; regenerate it
with `pixi run codegen`, never by hand. `src-tauri/gen/` is platform-generated
and ignored. Future biometric support belongs in Tauri plugins that wrap vault
key material with Keystore/Secure Enclave facilities; the UI continues to call
the generated client rather than platform APIs directly.
