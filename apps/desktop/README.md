# Castellan desktop

Tauri 2 + SvelteKit (static adapter). One `rpc` command in
`src-tauri/src/lib.rs` adapts Tauri invoke to the shared `castellan-dispatch`
crate; everything else—vault, TOTP, framing—comes from workspace crates.

## Run

```console
$ pixi run dev-desktop   # from the repository root
```

## First-time setup

- Icons: `app-icon.png` is a placeholder. Replace it with a real 1024x1024
  PNG and run `pixi run bun run --cwd apps/desktop tauri icon app-icon.png`
  from the repository root to rewrite `src-tauri/icons/`;
  `tauri.conf.json`'s `bundle.icon` array already lists the five files that
  command produces. `apps/mobile` carries the same pair of files.
- Linux needs the usual Tauri system libraries (webkit2gtk-4.1, librsvg,
  libssl); macOS and Windows need Xcode / WebView2 respectively.

## Layout

| Path | What it is |
| --- | --- |
| `src-tauri/` | the Tauri shell—workspace member, thin adapter to the shared dispatcher |
| `src/generated/client.ts` | xtask-generated operations assigned to the desktop face |
| `@castellan/tauri` | the shared desktop/mobile Transport implementation over `invoke` |
| `src/routes/` | SvelteKit routes (`ssr = false`, prerendered shells) |
