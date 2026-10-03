# Castellan mobile

Tauri 2 iOS/Android + SvelteKit. Its `rpc` command is the same thin adapter
to `castellan-dispatch` as desktop; its xtask-generated `MobileClient` exposes
only operations assigned to mobile. Its invoke adapter comes from the same
`@castellan/tauri` package as desktop, while this face owns phone metrics for
the shared `VaultHome` surface. Biometric unlock / credential-provider
integrations land here as Tauri plugins.

## Run

```console
$ pixi run bun run --cwd apps/mobile android:init   # once: generate Android
$ pixi run bun run --cwd apps/mobile android        # device or emulator
$ pixi run bun run --cwd apps/mobile ios:init       # once, on a Mac
$ pixi run bun run --cwd apps/mobile ios
```

`src/generated/client.ts` is committed xtask output for the operations assigned
to mobile. `src-tauri/gen/` is platform-generated and git-ignored. Biometric
unlock wraps the vault key in Keystore/Secure Enclave per the design doc; the
plugin surface this app will grow is `createBioKey` / `wrap` / `unwrap`.
