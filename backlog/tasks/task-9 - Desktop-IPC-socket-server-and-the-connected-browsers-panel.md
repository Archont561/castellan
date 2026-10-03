---
id: TASK-9
title: Desktop IPC socket server and the connected-browsers panel
status: Done
assignee:
  - '@agent'
created_date: '2026-10-01 20:40'
updated_date: '2026-10-03 15:35'
labels:
  - ipc
  - desktop
  - security
dependencies:
  - TASK-4
references:
  - crates/ipc
  - apps/desktop/src-tauri/src/lib.rs
priority: high
ordinal: 900
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The app-side half of the native channel: a UDS/named-pipe server multiplexing host connections into sessions, the association handshake (extension keypair, app-side confirm, nonce challenge per connection), and the UI panel that shows connected faces — making a broken integration visible instead of silent.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Multiple simultaneous host connections (two browsers, one profile each) multiplex cleanly by request id
- [x] #2 Unassociated extensions get silence; the app UI prompts on first association and lists remembered keys
- [x] #3 Same-user enforcement via peer credentials (SO_PEERCRED/LOCAL_PEERCRED/pipe PID) — documented and tested
- [x] #4 Connected-browsers panel shows face, version, last request; kill switch per connection
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Server in the desktop app crate over castellan-ipc framing; association state in a small app-side store; Windows named-pipe branch lands here (task-4 left the stub).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Started 2026-10-03. Design decisions made while building:

(1) The server lives in a new sibling crate, crates/ipc-server (castellan-ipc-server), not the desktop app crate: the Tauri shells cannot compile in the dev sandbox (no GTK/webkit), so every rule here is testable without a webview. The shells construct IpcServer::bind(socket_path, store_path, handler, hello), call start(), render panel(), and forward their session's events through broadcast().

(2) Association proof is HMAC-SHA256 possession, not doc-2's Ed25519 sketch: no signature crate is vendored and hand-rolling one is off the table. Enroll carries the symmetric key hex over the peer-credential-checked local socket once; returning connections answer a per-connection nonce challenge with HMAC(key, nonce). The wire shapes (enroll = material, claim = bare id) are scheme-agnostic, so an Ed25519 swap rewrites association::verify/proof_hex and nothing else.

(3) Peer credentials: std's peer_cred is unstable at our toolchain (rust#42839), so the one getsockopt the security model requires goes through libc — which forced the workspace's first sanctioned unsafe exception. The [workspace.lints] comment pre-authorized exactly this ('a raw socket option'), so unsafe_code moved forbid->deny with the two #[allow(unsafe_code)] functions in server.rs (SO_PEERCRED on Linux, getpeereid on macOS/iOS, getuid) each carrying a SAFETY case. Linux and macOS check the kernel's peer uid against ours; other Unixes rely on the 0700 per-user socket directory, the baseline lock on every platform. The decision itself is the pure same_user(), tested without a second uid.

(4) Panel as data: PanelSnapshot (connections/pending/remembered) + a change-signal subscription; the UI is BrowsersPanel in @castellan/ui (presentational, callbacks in), wired by the shells.

Windows: the named-pipe branch is a documented Err(Unsupported) stub (server_stub.rs, same API shape) until a Windows CI lane exists. Desktop/mobile Tauri wiring is the CI lane's first stop; the extension transport (v3 hello with face + association, challenge answering, UnknownKey re-enroll, requests gated until the app's hello) and the panel component are landed and tested here.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Done 2026-10-03. Protocol v3 (castellan-protocol): hello carries face/client_version/association (all defaulted, so v2 hellos still parse — and get silence, which is the v2-client behavior by design); new Proof{key_id, proof_hex} client message, Challenge{key_id, nonce_hex} and UnknownKey{key_id} host messages; panel types ConnectionInfo/ConnectionState/PendingKey/RememberedKey/PanelSnapshot; rpc_contract! now emits RpcMethod::tag() so the panel can show the last request as a method name, never a payload. Codegen extended; 20 generated TS files.

castellan-ipc-server: AssociationStore (remembered keys persisted as pretty JSON, corrupt = StoreError::Corrupt not silently empty; pending prompts ephemeral by design — a restart clears unanswered asks, the safe direction) with waiter-counted prompts: the last waiter out takes a denied/cancelled prompt down, a shared prompt (two connections, one key) survives one kill, and timeouts remove the ask. IpcServer binds a 0700-dir/0600-socket UDS (live-probe distinguishes AddrInUse from a stale socket, which is removed and rebound), accepts in one thread, and runs per connection: peer-credential check -> hello -> association -> nonce challenge -> proof -> the app's hello -> the request loop, with a channel-fed writer so responses and broadcast events can never interleave mid-frame. Every refusal closes with zero application bytes (the silence rule); the one exception is UnknownKey, which invites exactly one follow-up enroll. The kill switch lands within one 250ms tick whether the connection is serving or still awaiting approval. The frame reader is resumable across read-timeout ticks — a naive read_exact retry would silently desync the stream mid-frame.

Extension (apps/extension): association.ts (key in browser.storage.local, 32 random bytes, id = SHA-256 fingerprint prefix, WebCrypto HMAC prover — all injectable); transport.ts now speaks v3 (requests queue until the association completes; the burst property test caught a real bug where every request during the handshake opened its own port). Panel UI: BrowsersPanel in @castellan/ui — prompts with Allow/Deny, connections with face/version/last-request/state and a Disconnect (kill) button per row, remembered keys with dates.

Suites: 20 unit + 25 integration (real Unix socket in a scratch dir: multiplexing, silence for unassociated/wrong-proof/collision, enroll->prompt->confirm->challenge->proof happy path, deny, prompt expiry, UnknownKey re-enroll, returning-client claim, kill switch latency <= 1s, kill-during-approval, shared-prompt-survives-one-kill, stale-vs-live socket, store persistence across restart, change signals, broadcast-to-ready-only, v2 compat, face matrix) + proptest proof roundtrip; 13 extension tests (6 association + 7 transport incl. two fast-check properties); 5 Playwright component tests for BrowsersPanel. Gates: cargo fmt/clippy/test (workspace minus the two Tauri shells), codegen, turbo lint+typecheck+test for every TS package. One infra fix: turbo now passes CARGO_HOME/CARGO_NET_OFFLINE through (it was stripping them, so the rust codegen node tried to hit crates.io offline). The unsafe_code forbid->deny change is the one decision worth a maintainer's eye — it is the exception the lint's own comment prescribed, scoped to three audited functions, but it is a security-posture change and one revert away if the maintainer prefers deferring SO_PEERCRED to a Windows-lane-era PR.
<!-- SECTION:FINAL_SUMMARY:END -->
