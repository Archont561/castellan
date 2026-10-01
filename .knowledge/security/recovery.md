---
type: Reference
title: "Castellan — recovery"
description: "Recovery codes, emergency kit, optional Shamir splitting, and the no-account-recovery stance."
tags:
  - recovery
  - security
status: draft
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: security/recovery
category: security
refs:
  - features/vault
  - security/threat-model
---
# Recovery

Two different problems live under this word; Castellan keeps them apart.

## 1. *Other services'* recovery codes (task-18, task-19)

Structured entries: issuer, account, codes with per-code `usedAt`, age. Grid
UI (concealed by default, tap-to-copy with clipboard timer, mark-used with
undo), regenerate nudges by age, smart paste that recognizes token blocks.
Codes are excluded from the search index (they are secrets, not metadata).

## 2. *Castellan's own* recovery (task-22, task-35)

There is **no account recovery by design** — there are no accounts. The
master password is not recoverable, and the product says so instead of
pretending. What exists:

- **Backups** (task-22): timestamped copies on every save, HMAC-verified,
  retention policy, restore with a review diff.
- **Emergency kit** (task-35): printable — vault path, KDF parameters,
  keyfile as QR, and a write-your-password-here field. The password is
  written by hand on paper because that is the only place it can live.
- **Optional Shamir 3-of-5** (task-35): the master password split for
  households; any 3 shares recombine, 2 shares provably refuse.

Status: **draft** — specified and tasked, not yet built. The stance
(no-account, no escrow, paper-kit honesty) is settled and drives all of it.
