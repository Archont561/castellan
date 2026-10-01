---
id: TASK-14
title: "Search, command palette, keyboard-first navigation"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:40'
updated_date: '2026-10-01 20:40'
labels:
  - ux
  - desktop
dependencies: [TASK-7]
references:
  - apps/desktop/src/routes
priority: medium
ordinal: 1400
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Instant fuzzy search over titles, usernames, URLs and tags — never secret values — plus a command palette on a global shortcut. The KeePassXC bar to clear: everything reachable without the mouse.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Search indexes metadata only; protected fields never enter the index (test proves it)
- [ ] #2 Palette: create entry, generate passphrase, lock vault, open database, jump to entry
- [ ] #3 List virtualizes for 10k+ entries at 60fps
- [ ] #4 Saved smart filters (tag + expiry + has-2FA) survive restart
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Index built in Rust on unlock (metadata projection), streamed to the face; palette as a shared ui component; virtualized list with svelte's each + windowing.
<!-- SECTION:PLAN:END -->
