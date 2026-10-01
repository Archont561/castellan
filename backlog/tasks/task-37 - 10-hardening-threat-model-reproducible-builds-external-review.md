---
id: TASK-37
title: "1.0 hardening: threat model, reproducible builds, external review"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:50'
updated_date: '2026-10-01 20:50'
labels:
  - security
  - release
dependencies: [TASK-29,TASK-30,TASK-32]
references:
  - backlog/docs/plans/roadmap-v1/doc-4
priority: high
ordinal: 3700
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The gate to 1.0: publish the threat model, make release builds reproducible (verified byte-identical from two machines), and commission an external review of the crypto-relevant paths (KDBX handling, the native channel, the passkey enforcement, biometric key wrapping).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Threat model published (assets, adversaries, boundaries, residual risks incl. soft-passkey honesty)
- [ ] #2 Reproducible builds for desktop on all three platforms with a verification script
- [ ] #3 External review findings triaged; anything open is listed in the README with severity
- [ ] #4 security.txt, signed release artifacts, and a disclosure policy
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Determinism work likely lands as toolchain pinning + vendored deps; the review scope doc comes from the knowledge base's threat-model entry.
<!-- SECTION:PLAN:END -->
