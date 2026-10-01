# Fob — the Castellan browser companion

One codebase (WXT) → Chrome/Edge/Brave/Vivaldi (chromium) and Firefox
(gecko). Speaks the same protocol as the desktop and mobile apps over native
messaging; the host is the app binary itself, so app/proxy/extension version
drift — the classic KeePass breakage — cannot happen.

## Run

```console
$ bun run dev            # chromium, loads the unpacked extension
$ bun run dev:firefox
```

## Layout

| Path | What it is |
| --- | --- |
| `entrypoints/background.ts` | the root: one transport, one client, reconnect policy |
| `entrypoints/content.ts` | passkey interception at document_start, MAIN world |
| `entrypoints/popup/` | the Fob popup |
| `src/transport.ts` | the Transport implementation over connectNative |
| `native-hosts/` | template of the host manifest the app installs |

The host manifest template lives here because the *extension* defines the
extension ID the app must allow; the app writes the actual manifests.
