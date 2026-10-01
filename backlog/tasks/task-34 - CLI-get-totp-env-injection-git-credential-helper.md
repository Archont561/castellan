---
id: TASK-34
title: "CLI: get, totp, env injection, git credential helper"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:50'
updated_date: '2026-10-01 20:50'
labels:
  - devtools
  - cli
dependencies: [TASK-9,TASK-7]
references:
  - crates
priority: medium
ordinal: 3400
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The developer face: castellan get -f password github.com | pbcopy, castellan totp npm, castellan env -- npm test (secrets as env vars for local dev — a local Doppler), and git credential helper wiring. Same socket, same protocol, same dispatcher.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 get/totp print or copy with the same lock semantics as the GUI (locked = clear error, not a hang)
- [ ] #2 env spawns a child process with injected variables, cleaned up after exit; the values never appear in ps output
- [ ] #3 git credential helper implements fill/approve/erase against the vault
- [ ] #4 One binary: the CLI and --native-host are the same artifact (decision-2)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Small clap CLI crate reusing the IPC client path; env injection via direct exec with a scrubbed environment, not shell interpolation.
<!-- SECTION:PLAN:END -->
