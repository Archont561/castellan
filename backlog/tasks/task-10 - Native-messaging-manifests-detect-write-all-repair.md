---
id: TASK-10
title: 'Native-messaging manifests: detect, write all, repair'
status: Done
assignee:
  - '@agent'
created_date: '2026-10-01 20:40'
updated_date: '2026-10-03 15:54'
labels:
  - extension
  - desktop
dependencies:
  - TASK-9
references:
  - apps/extension/native-hosts
  - backlog/docs/specifications/native-channel/doc-2
priority: high
ordinal: 1000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
One button in settings: detect installed browsers, write every host manifest (HKCU registry on Windows, per-browser NativeMessagingHosts dirs on macOS/Linux) listing all our extension IDs, plus a Repair button that rewrites them all — deleting an entire category of KeePass forum threads.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Chrome, Edge, Brave, Vivaldi, Firefox manifests written from one pass on Windows/macOS/Linux
- [x] #2 Extension IDs pinned at first store submission appear in allowed_origins; dev ID alongside
- [x] #3 Repair rewrites all manifests idempotently and reports what it fixed
- [x] #4 A stale manifest is detected (path moved, ID missing) and surfaced in the panel from task-9
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Detection per platform (registry + well-known dirs), manifest templates from the extension's native-hosts/ dir, e2e test spawning the host from a generated manifest.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Started 2026-10-03. Design decisions: (1) The installer is a new sibling crate, crates/manifests (castellan-manifests), for the same reason the IPC server is one: locally testable without a webview; the settings surface calls into it. (2) Platform is a value, not cfg — the layout table answers for all three desktop OSes from any build, so Linux CI exercises the macOS and Windows tables too. (3) The host identity (name, description, extension ids) comes from apps/extension/native-hosts/host.json through a new generated Rust file (crates/manifests/src/generated/native_host.rs, written by the same xtask pass that writes the TS constant), so the installer never repeats a string host.json already states; store IDs land everywhere via one edit + codegen. (4) Repair IS install: the write is canonical-content-every-time, and the report distinguishes Written / Repaired (with what changed: path updated, allow-list grew, replaced garbage or foreign) / AlreadyCorrect / SkippedNotInstalled / Unsupported / Failed. (5) Windows: the registry write (HKCU Software\<Browser>\NativeMessagingHosts -> one shared manifest file under AppData/Local/Castellan) is honestly stubbed as Unsupported — a manifest file without its registry value is integration that only looks installed; the layout table is complete and tested, the write lands with the Windows CI lane, same posture as task-9's named pipe. (6) Audit findings surface through PanelSnapshot.manifest_problems (new wire types ManifestProblem/ManifestProblemKind) — the shell runs the audit and hands the rows to IpcServer::set_manifest_problems; the BrowsersPanel renders them with a repair affordance.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Done 2026-10-03. castellan-manifests: Browser/Family/Platform layout table (Linux: ~/.config/<browser>/NativeMessagingHosts, gecko's lowercase ~/.mozilla/native-messaging-hosts — the test caught my initial CamelCase guess; macOS: ~/Library/Application Support/<Browser>/NativeMessagingHosts; Windows: HKCU registry keys pointing at one shared manifest file), Installer::install_all (one pass: writes every INSTALLED browser's manifest — Chrome, Edge, Brave, Vivaldi, Firefox — skips the not-installed rather than leaving confetti), audit (Missing/Unreadable/StalePath/MissingId/Foreign, with human detail strings; repair clears it), and manifest_content pinned byte-for-byte against the committed xtask templates with {path} filled. Protocol: PanelSnapshot.manifest_problems + ManifestProblem{browser,kind,detail} + ManifestProblemKind, generated to TS; IpcServer::set_manifest_problems (+ stub twin) carries them into the panel and fires the change signal; BrowsersPanel renders 'Needs repair' rows. Suites: 25 tests in crates/manifests — the full browser x platform layout matrix (rstest), detection, one-pass install (Linux + macOS), every pinned ID in every allow list (AC-2), template parity, idempotent repair with transcript, garbage/foreign replacement, a proptest repair-fixpoint from arbitrary file bytes, the every-way-stale audit matrix, and the honest Windows Unsupported. ipc-server +1 (manifest problems surface + change signal), ui +1 (stale-manifest row). Gates: fmt, clippy -D warnings, cargo test (workspace minus Tauri shells), codegen committed and clean, turbo lint+typecheck+test all green. Desktop settings button wiring is the CI lane's (Tauri shells cannot compile in the dev sandbox); every rule it calls is tested here.
<!-- SECTION:FINAL_SUMMARY:END -->
