# Castellan mobile

Tauri 2 iOS/Android + SvelteKit. The same `rpc` dispatcher as desktop (see
`src-tauri/src/lib.rs`), the same `@castellan/*` packages; the frontend is a
mobile layout, and the biometric unlock / credential-provider plugins land
here as Tauri mobile plugins.

## Run

```console
$ bun run android:init   # once: generates src-tauri/gen/android
$ bun run android        # on a device or emulator
$ bun run ios:init       # once, on a mac: generates src-tauri/gen/ios
$ bun run ios
```

`src-tauri/gen/` is generated and git-ignored. Biometric unlock wraps the
vault key in Keystore/Secure Enclave per the design doc; the plugin surface
this app will grow is `createBioKey` / `wrap` / `unwrap`.
