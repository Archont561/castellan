# Native messaging host manifests

`host.json` is the single source for the native host identity and the browser
extension IDs allowed to connect. `bun run codegen` derives:

- `generated/chromium/app.castellan.host.json` with `allowed_origins`;
- `generated/firefox/app.castellan.host.json` with `allowed_extensions`; and
- `apps/extension/src/generated/native-host.ts`, used by `connectNative`.

The generated manifests are templates for the manifest installer
(`crates/manifests`, task-10). It replaces `{path}` with the absolute app
binary path and writes the browser-family-specific file into every installed
browser's `NativeMessagingHosts` directory in one pass; its audit detects
stale manifests and the settings surface offers the repair that rewrites
them. Replace the store-ID placeholders when those IDs are assigned (the
installer picks the change up through the generated Rust identity on the next
`bun run codegen`); the Firefox development ID is pinned in `wxt.config.ts`.
