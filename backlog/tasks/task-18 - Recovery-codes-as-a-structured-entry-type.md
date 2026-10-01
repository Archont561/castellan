---
id: TASK-18
title: "Recovery codes as a structured entry type"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:45'
updated_date: '2026-10-01 20:45'
labels:
  - vault
  - ux
  - security
dependencies: [TASK-8]
references:
  - backlog/docs/specifications/protocol/doc-1
priority: high
ordinal: 1800
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Recovery codes stop being secure notes: a structured type (issuer, account, codes with per-code usedAt, generatedAt), grid UI with tap-to-copy and mark-used, regenerate nudges by age, and smart paste that recognizes a block of similar tokens and offers to create the entry.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Structured storage degrades to readable custom fields in KeePassXC/KeePassDX (no parse errors)
- [ ] #2 Grid UX: concealed by default, tap copies with the clipboard timer, checkbox marks used with undo
- [ ] #3 Smart paste: ~10 similar tokens matching the common service patterns offers structured creation
- [ ] #4 Entries older than the threshold (default 6 months) show a regenerate-on-site badge; codes are excluded from the search index
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Storage shape in the protocol (NewEntry extension) + vault mapping; UI as a shared ui component; smart paste heuristics in Rust so the extension gets them too.
<!-- SECTION:PLAN:END -->
