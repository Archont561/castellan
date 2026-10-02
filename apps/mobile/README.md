# Castellan mobile

Tauri 2 iOS/Android + SvelteKit. Its `rpc` command is the same thin adapter
to `castellan-dispatch` as desktop; its xtask-generated `MobileClient` exposes
only operations assigned to mobile. Its invoke adapter comes from the same
`@castellan/tauri` package as desktop, while this face owns phone metrics for
the shared `VaultHome` surface. Biometric unlock / credential-provider
integrations land here as Tauri plugins.

## Run

```console
$ bun run android:init   # once: generates src-tauri/gen/android
$ bun run android        # on a device or emulator
$ bun run ios:init       # once, on a mac: generates src-tauri/gen/ios
$ bun run ios
```

`src/generated/client.ts` is committed xtask output for the operations assigned
to mobile. `src-tauri/gen/` is platform-generated and git-ignored. Biometric
unlock wraps the vault key in Keystore/Secure Enclave per the design doc; the
plugin surface this app will grow is `createBioKey` / `wrap` / `unwrap`.
