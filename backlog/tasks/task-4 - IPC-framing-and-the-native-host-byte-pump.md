---
id: TASK-4
title: "IPC framing and the native-host byte pump"
status: Done
assignee:
  - '@agent'
created_date: '2026-10-01 19:15'
updated_date: '2026-10-01 20:00'
labels:
  - ipc
  - extension
dependencies: []
references:
  - crates/ipc/src/lib.rs
  - crates/native-host/src/main.rs
priority: high
ordinal: 400
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The one native channel: 4-byte LE framing identical to Chromium's native-messaging format, the 1 MB cap, the well-known socket path per platform, and the host binary that pumps frames between browser stdio and the socket without parsing anything.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Frames round-trip; oversize frames refused as InvalidData; empty frames are messages, not EOF (tested)
- [x] #2 Host pumps both directions on separate threads; app disconnect ends the process cleanly
- [x] #3 No localhost TCP, no JSON parsing, no unsafe anywhere in the path
- [x] #4 Named-pipe branch on Windows exits with a clear not-yet message rather than failing obscurely
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
std-only ipc crate (no runtime, no serde), host as a thin bin crate with two pump loops and good stderr for debugging.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
clippy required while-let loops over the match-and-break shape. Windows named pipes land with task-9's socket server; the stub is honest about it.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The host is ~120 lines and parses nothing; version drift between host and app is structurally impossible (decision-2).
<!-- SECTION:FINAL_SUMMARY:END -->
