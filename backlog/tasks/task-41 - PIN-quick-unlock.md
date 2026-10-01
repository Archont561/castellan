---
id: TASK-41
title: PIN quick-unlock
status: To Do
assignee: []
created_date: '2026-10-01 22:14'
updated_date: '2026-10-01 22:15'
labels:
  - security
  - ux
dependencies:
  - TASK-32
documentation:
  - backlog/docs/specifications/biometric-unlock/doc-5
priority: medium
ordinal: 6800
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
A per-device PIN that gates K_hw in the doc-5 architecture - biometrics' slot without a sensor and KeePassDX parity. The PIN never decrypts anything and is never the only factor: it releases the hardware-wrapped vault key exactly as a fingerprint does and hard-locks to password under the same rules.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A PIN can be enrolled per device and unlocks the vault by unwrapping K_master through K_hw exactly as biometric unlock does (the doc-5 flow)
- [ ] #2 Wrong-PIN throttling matches the biometric path: 3 failures hard-lock to password
- [ ] #3 PIN change or removal invalidates the wrap the same way a biometric enrollment change does
- [ ] #4 The UI labels the PIN as a convenience factor and never as a second master password
<!-- AC:END -->
