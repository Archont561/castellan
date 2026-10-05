# Fob — the Castellan browser companion

Fob is the WXT browser-extension face for Chromium-family browsers and Firefox.
It speaks the same protocol as the native faces over browser native messaging:
the browser-spawned host only pumps bytes to the local app, while the extension
owns browser interaction, association-key persistence, and reconnect behavior.
It never opens or stores a vault.

## Run and verify

Use Pixi from the repository root:

```console
$ pixi run dev-extension
$ pixi run bun run --cwd apps/extension dev:firefox
$ pixi run bun run --cwd apps/extension typecheck
$ pixi run bun run --cwd apps/extension test
$ pixi run bun run --cwd apps/extension e2e
```

`e2e` first builds the unpacked Chromium extension, then starts full
Chrome-for-Testing with a persistent profile. It cannot use the default
headless shell because that shell cannot load extensions.

## Layout

| Path | Responsibility |
| --- | --- |
| `entrypoints/background.ts` | Native-messaging transport/client lifecycle |
| `entrypoints/content.ts` | Document-start content integration |
| `entrypoints/popup/` | The Fob status popup |
| `src/transport.ts` | Tested `connectNative` transport and association handshake |
| `src/association.ts` | Association key storage and WebCrypto proof helpers |
| `src/messages.ts` | Typed messages among extension surfaces |
| `src/generated/client.ts` | Generated web-extension RPC client |
| `native-hosts/` | Host identity and generated browser manifest templates |
| `test/`, `e2e/` | Unit/protocol and built-extension browser tests |

## Generated contract and host identity

`src/generated/client.ts`, `src/generated/native-host.ts`, and
`native-hosts/generated/` are xtask output. Update the Rust protocol or
`native-hosts/host.json`, then run `pixi run codegen`; do not edit the output.
The manifest installer in `crates/manifests` replaces the binary-path template
and installs/audits the platform-specific host files. See
[`native-hosts/README.md`](native-hosts/README.md) for that boundary.
