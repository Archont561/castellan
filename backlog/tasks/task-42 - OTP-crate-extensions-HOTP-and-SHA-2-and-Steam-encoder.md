---
id: TASK-42
title: 'OTP crate extensions: HOTP and SHA-2 and Steam encoder'
status: To Do
assignee: []
created_date: '2026-10-01 22:14'
updated_date: '2026-10-01 22:15'
labels:
  - otp
dependencies: []
priority: medium
ordinal: 7800
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The otp crate's own documented future work: counter-based HOTP (RFC 4226) where the interesting part is counter persistence and increment-once-per-revealed-code in the vault entry; SHA-256/SHA-512 algorithms behind the parser that currently refuses them with a named error; and Steam's 5-character display encoder behind the encoder parameter that is refused today. Refusal was the design until each of these could compute correctly rather than silently wrong.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 otpauth://hotp/ URIs parse with their counter parameter and compute RFC 4226 codes matching the RFC appendix vectors
- [ ] #2 HOTP counters persist in the vault entry and increment exactly once per revealed code (never per computation)
- [ ] #3 algorithm=SHA256 and algorithm=SHA512 compute correct RFC 6238 codes
- [ ] #4 encoder=steam displays the 5-character Steam Guard alphabet
- [ ] #5 Property tests cover round-trips and code lengths for every new mode
<!-- AC:END -->
