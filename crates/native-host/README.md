# `castellan-native-host`

The native-messaging host process spawned by a browser. It is deliberately a
byte pump between browser stdio and Castellan's local IPC socket: it does not
parse JSON, own a protocol version, open a vault, or make authorization
decisions.

That narrow boundary prevents proxy-version drift. Browser native messaging,
the local socket, and `castellan-ipc` all use the same framed bytes, so a host
can forward them unchanged while the app-side server performs association and
dispatch.

The installed app invokes this behavior for its native-host mode; contributors
normally exercise it through the extension and manifest installer rather than
as a standalone application.

## Verify

```console
$ pixi run cargo clippy -p castellan-native-host --all-targets -- -D warnings
$ pixi run bun run --cwd apps/extension e2e
```
