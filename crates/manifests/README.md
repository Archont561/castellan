# `castellan-manifests`

Native-messaging manifest detection, installation, audit, and repair for the
supported browser families. It turns the extension-owned host identity into
browser-specific files pointing at the installed Castellan app binary.

## Responsibilities

- Models browser families, platforms, and native-messaging manifest locations.
- Uses `Installer` to detect installed browsers, write manifests in one pass,
  audit stale/missing/foreign files, and repair them idempotently.
- Returns `InstallReport`, `BrowserOutcome`, and `Change` values so UI code can
  report exactly what happened.
- Consumes generated host identity from `generated/native_host.rs`.

Do not hand-edit the generated host identity. Update
`apps/extension/native-hosts/host.json` and run `pixi run codegen`; the
extension declares allowed IDs and this crate installs the matching templates.

## Verify

```console
$ pixi run cargo nextest run -p castellan-manifests
$ pixi run cargo test --doc -p castellan-manifests
$ pixi run cargo clippy -p castellan-manifests --all-targets -- -D warnings
```
