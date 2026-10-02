# Native messaging host manifests

`host.json` is the single source for the native host identity and the browser
extension IDs allowed to connect. `bun run codegen` derives:

- `generated/chromium/app.castellan.host.json` with `allowed_origins`;
- `generated/firefox/app.castellan.host.json` with `allowed_extensions`; and
- `apps/extension/src/generated/native-host.ts`, used by `connectNative`.

The generated manifests are templates for the desktop app's browser-integration
installer. It replaces `{path}` with the absolute app binary path and installs
the browser-family-specific file as `app.castellan.host.json`. Replace the
store-ID placeholders when those IDs are assigned; the Firefox development ID
is pinned in `wxt.config.ts`.
