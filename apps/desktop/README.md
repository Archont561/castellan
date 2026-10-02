# Castellan desktop

Tauri 2 + SvelteKit (static adapter). One `rpc` command bridges the frontend
to the Rust dispatcher in `src-tauri/src/lib.rs`; everything else — vault,
TOTP, framing — comes from the workspace crates.

## Run

```console
$ bun run tauri dev      # from this directory
```

## First-time setup

- Icons: `app-icon.png` is a placeholder. Replace it with a real 1024x1024
  PNG and run `bunx tauri icon app-icon.png` to rewrite `src-tauri/icons/`;
  `tauri.conf.json`'s `bundle.icon` array already lists the five files that
  command produces. `apps/mobile` carries the same pair of files.
- Linux needs the usual Tauri system libraries (webkit2gtk-4.1, librsvg,
  libssl); macOS and Windows need Xcode / WebView2 respectively.

## Layout

| Path | What it is |
| --- | --- |
| `src-tauri/` | the Tauri app crate — workspace member, RPC dispatcher |
| `src/lib/transport.ts` | the Transport implementation over `invoke` |
| `src/routes/` | SvelteKit routes (`ssr = false`, prerendered shells) |
