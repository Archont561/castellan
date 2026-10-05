# Castellan desktop

The desktop face is a Tauri 2 shell around a statically adapted SvelteKit UI.
It constructs a generated `DesktopClient` over the shared `@castellan/tauri`
transport; the single Rust `rpc` command delegates to `castellan-dispatch`.
The app shell does not implement vault, TOTP, or protocol behavior itself.

## Run and verify

Run from the repository root through Pixi:

```console
$ pixi run dev-desktop
$ pixi run bun run --cwd apps/desktop typecheck
$ pixi run bun run --cwd apps/desktop e2e
```

Linux development needs the Tauri WebKit/GTK system libraries; macOS needs
Xcode and Windows needs WebView2. See the root [README](../../README.md) for
the locked toolchain setup.

## Layout

| Path | Responsibility |
| --- | --- |
| `src/routes/` | Static SvelteKit shell and desktop-specific composition/metrics |
| `src/lib/qr.ts` | Desktop-only QR image and screen-capture acquisition |
| `src/generated/client.ts` | Committed xtask output for desktop-allowed RPC methods |
| `src-tauri/` | Tauri workspace member and thin adapter to the shared dispatcher |
| `e2e/` | Browser-level shell tests against the SvelteKit face |

## Generated and platform files

Do not hand-edit `src/generated/client.ts`; change the Rust protocol and run
`pixi run codegen`. `src-tauri/gen/` is generated platform scaffolding and is
ignored. The icon assets are placeholders: replace `app-icon.png` with a real
1024×1024 PNG, then run:

```console
$ pixi run bun run --cwd apps/desktop tauri icon app-icon.png
```

That command refreshes the icon files already named by `tauri.conf.json`.
