---
id: doc-2
title: "Native Channel and Host Specification"
type: specification
created_date: '2026-10-01 21:05'
updated_date: '2026-10-01 21:05'
tags:
  - extension
  - ipc
  - security
  - specification
---
# Specification — the one native channel

**Scope.** How browsers reach the app without a localhost server, and why
that design point is non-negotiable (decision-2).

## §1 Components

- **Host** = the app binary, `castellan --native-host`, spawned per browser
  connection. ~120 lines: two pump threads (stdin→socket, socket→stdout),
  zero parsing. Frame format identical on both sides, so it is a byte pump.
- **IPC server** in the desktop app: Unix domain socket at
  `$XDG_RUNTIME_DIR/castellan/castellan.sock` (fallback `/tmp/castellan-<uid>/`,
  0700) or Windows named pipe `\\.\pipe\castellan`. Multiplexes sessions;
  one request-id space per connection.
- **Manifests** the app writes per browser (see doc-1 of task-10): HKCU
  registry on Windows, `NativeMessagingHosts/` dirs on macOS/Linux, all
  extension IDs (store + dev) in `allowed_origins`.

## §2 Connection lifecycle

1. Browser spawns the host; host connects the socket. Connection without the
   app running exits with a clear stderr message (visible to the app's log
   collector, invisible to the browser).
2. Client sends `hello` (protocol version). App answers `hello` + capability
   list. Unknown-version clients still get method-level degradation.
3. Requests multiplex by id; events push at any time.
4. Either side's EOF closes the connection; the host exits. A browser
   restarting its MV3 service worker reconnects transparently — the
   transport's job, invisible to client code.

## §3 Security properties

- **No TCP.** No firewall prompt, no port conflict, and nothing for other
  local processes to query — the KeePassHttp sin.
- **Same-user enforcement**: peer credentials checked (SO_PEERCRED /
  LOCAL_PEERCRED / named-pipe client PID). Cross-user connections refused.
- **Association handshake** (task-9): the extension generates an Ed25519
  keypair; the app shows "Chrome wants to connect"; the public key is stored
  and every subsequent connection proves possession with a nonce challenge.
  Unassociated hosts get silence.
- **Origin enforcement in the app** (doc-1 §2): the extension can lie about
  nothing that matters.

## §4 The one exception

Safari routes through the containing app's `SafariWebExtensionHandler` +
XPC, not a spawned host. Same protocol, different transport — the webkit
adapter's problem, macOS-only.
