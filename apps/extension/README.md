# Fob — the Castellan browser companion

One codebase (WXT) → Chrome/Edge/Brave/Vivaldi (chromium) and Firefox
(gecko). Speaks the same protocol as the desktop and mobile apps over native
messaging; the host is the app binary itself, so app/proxy/extension version
drift — the classic KeePass breakage — cannot happen.

## Run

```console
$ pixi run dev-extension                              # Chromium
$ pixi run bun run --cwd apps/extension dev:firefox  # Firefox
```

## Layout

| Path | What it is |
| --- | --- |
| `entrypoints/background.ts` | the root: one transport, one client, reconnect policy |
| `entrypoints/content.ts` | passkey interception at document_start, MAIN world |
| `entrypoints/popup/` | the Fob popup |
| `src/generated/client.ts` | xtask-generated operations assigned to the web-extension face |
| `src/transport.ts` | the tested Transport implementation over connectNative |
| `src/messages.ts` | the typed popup/content-to-background message contract |
| `native-hosts/host.json` | canonical host identity and browser extension IDs |
| `native-hosts/generated/` | xtask-generated Chromium and Firefox manifest templates |

Host metadata lives here because the *extension* defines the IDs the app must
allow. Xtask emits each browser family's schema and the constant used by
`connectNative`; the app installs the appropriate generated manifest after
replacing its binary-path placeholder.
