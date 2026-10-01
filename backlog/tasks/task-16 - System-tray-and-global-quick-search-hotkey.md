---
id: TASK-16
title: "System tray and global quick-search hotkey"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:40'
updated_date: '2026-10-01 20:40'
labels:
  - desktop
  - ux
dependencies: [TASK-15]
references:
  - apps/desktop/src-tauri
priority: medium
ordinal: 1600
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Tray presence with lock state mirroring the protocol events, quick-search popup on a global hotkey (the KeePassXC-style flow: hotkey → type → enter → field copied and clipboard timer armed), autostart option.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Tray icon reflects locked/unlocked live from DatabaseLocked/Unlocked events
- [ ] #2 Global hotkey opens quick search over the metadata index; Enter copies password, C copies username, T copies TOTP
- [ ] #3 Autostart works on all three desktop platforms and is a setting, not a default
- [ ] #4 Quick search never shows secret values in results, only titles
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Tauri tray + global-shortcut plugins; quick search reuses the search index and the ui package's palette component.
<!-- SECTION:PLAN:END -->
