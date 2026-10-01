---
type: Reference
title: "Castellan — audit"
description: "Health findings computed in Rust, HIBP k-anonymity breach checks, offline 2FA availability hints."
tags:
  - audit
  - security
  - hibp
status: draft
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: features/audit
category: features
refs:
  - project/data-model
  - security/threat-model
---
# Audit

The vault's health view (tasks 20, 21): findings computed on unlock in
Rust — the face only renders. Audit results are protocol types, so the
extension and CLI can surface them too.

## Findings

- **Weak**: zxcvbn-class scoring per password.
- **Reused**: clusters grouped by password digest; the UI shows clusters,
  never a password next to a foreign entry.
- **Old**: entries with unchanged passwords beyond a threshold.
- **Expiring**: password-expiry fields honored.
- **2FA available but unused**: bundled 2fa.directory data, checked
  **offline** — no network lookup for this finding.

## Breach checks — HIBP k-anonymity (task-21)

The one third-party network feature, off by default: password checks send a
**5-character SHA-1 prefix**, candidate suffixes are compared locally; the
suffix never leaves the machine (a test asserts the request shape). Email
breach checks opt-in per address, cached with dates. All egress lives in
one module so the audit story is "look here, nowhere else" — and the
feature is listed in the [threat model](../security/threat-model.md).

Status: **draft** — tasked (m-2), not yet built.
