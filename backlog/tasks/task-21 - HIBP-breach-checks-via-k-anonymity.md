---
id: TASK-21
title: "HIBP breach checks via k-anonymity"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:45'
updated_date: '2026-10-01 20:45'
labels:
  - security
dependencies: [TASK-20]
references:
  - apps/desktop/src-tauri
priority: medium
ordinal: 2100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
HaveIBeenPwned password-range checks (k-anonymity: 5-char SHA-1 prefix, suffix compared locally) per password, and breach notifications per monitored email. The one place the product talks to a third-party network service, so it is opt-in, proxied nowhere, and visible in the audit log.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Password checks send prefix only; the suffix never leaves the machine (test asserts the request shape)
- [ ] #2 Email breach checks are opt-in per address with the reply cached and dated
- [ ] #3 Findings appear in the audit dashboard with the breach name and date
- [ ] #4 The feature is off by default and its network calls are listed in the threat model
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Small HTTP client behind a capability; all network egress in one module so the audit story is 'look here, nowhere else'.
<!-- SECTION:PLAN:END -->
