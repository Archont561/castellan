---
id: TASK-43
title: 'Release flow: changelog and versioning and tags'
status: To Do
assignee: []
created_date: '2026-10-01 22:14'
updated_date: '2026-10-01 22:15'
labels:
  - release
  - devtools
dependencies: []
priority: medium
ordinal: 10800
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The ground the reshape prepared: convco changelog generation from the conventional history with the .versionrc types, xtask version as the single version source every consumer reads, and the vX.Y.Z tag convention. The first tagged release is m-1 (v0.1); this task makes that a repeatable flow rather than a hand-edit.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 convco changelog generates a CHANGELOG from conventional commits using the .versionrc types
- [ ] #2 The workspace manifest version is the single source and xtask version is how scripts read it
- [ ] #3 The tag convention and first-release checklist are documented (repository init through history through tag)
- [ ] #4 A dry run of the full flow on a scratch repository is recorded in the task notes
<!-- AC:END -->
