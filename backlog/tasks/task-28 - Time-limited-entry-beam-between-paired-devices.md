---
id: TASK-28
title: "Time-limited entry beam between paired devices"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:50'
updated_date: '2026-10-01 20:50'
labels:
  - sync
  - ux
dependencies: [TASK-26]
references:
  - backlog/docs/specifications/device-mesh/doc-6
priority: medium
ordinal: 2800
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The LocalSend-spirit feature for secrets: beam one entry (or a TOTP seed) to a paired device for N minutes, for logins on a machine that is not yours. Receiver sees origin, sender, countdown; nothing persists after expiry.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Beam sends an encrypted entry that self-destructs after the countdown
- [ ] #2 Receiver sees a request first — acceptance is explicit, with the sender's device name
- [ ] #3 Beamed TOTP codes tick on the receiver for the remaining window, then stop
- [ ] #4 The beam is auditable on both ends (who, what, when — value never logged)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Push-style message over the pairing channel; expiry enforced by deletion on both ends; UI in the ui package shared by both faces.
<!-- SECTION:PLAN:END -->
