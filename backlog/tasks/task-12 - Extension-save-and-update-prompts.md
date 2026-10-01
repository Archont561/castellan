---
id: TASK-12
title: "Extension save and update prompts"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:40'
updated_date: '2026-10-01 20:40'
labels:
  - extension
  - ux
dependencies: [TASK-11]
references:
  - apps/extension/entrypoints
priority: medium
ordinal: 1200
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Capture new and changed credentials from the page on submit, offer save/update through the save prompt, store via save_entry with NewEntry — including otpauth URIs detected in QR flows where reachable.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 New credential detected on submit → save prompt; changed password → update prompt with a diff view
- [ ] #2 Generated passwords filled by the extension are remembered so the save prompt is pre-filled
- [ ] #3 Duplicate detection by origin+username before offering save
- [ ] #4 Save lands in the vault with the copy-aside rule and fires EntryChanged to other faces
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Form submit interception in the content script; prompt UI in the extension's own components; save through the protocol only — the extension never writes KDBX.
<!-- SECTION:PLAN:END -->
