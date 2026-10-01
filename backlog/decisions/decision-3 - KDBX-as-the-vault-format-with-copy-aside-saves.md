---
id: decision-3
title: "KDBX as the vault format, with copy-aside saves until keepass-rs writing is trustworthy"
date: '2026-10-01 20:15'
status: accepted
---
## Context

A secrets manager must pick its compatibility posture early: own format (Bitwarden,
1Password) or KeePass's KDBX (KeePassXC, KeePassDX, Strongbox). Castellan's pitch is
local-first ownership; KDBX means users arrive with existing vaults and leave with them
intact. Against that: keepass-rs 0.15's save path is experimental and has a documented
history of dropping fields it does not parse.

## Decision

**KDBX 4 is the on-disk format.** Read via keepass-rs today. Writing is allowed only behind
the copy-aside rule, enforced in the vault crate: a save copies the existing file aside
(timestamped), writes the new one, and keeps the copy until the user prunes. The round-trip
harness (task-8) parses saved files field-by-field against the original across a corpus of
KeePassXC-authored databases.

A native format stays **out of scope** for v1; if KDBX writing proves unmaintainable, the
fallback is import/export parity, not a second format.

## Consequences

- Zero-migration adoption for KeePass/KeePassXC/KeePassDX users; the value proposition
  lands on features, not on lock-in escape.
- Custom concepts (passkeys as `castellan.passkey` custom fields, recovery codes) must
  degrade gracefully in other KeePass clients — stored as ordinary fields a foreign client
  renders as "a weird custom field", never as a parse error.
- Save is slower and the directory grows; acceptable for a file that changes dozens of
  times a day at most. The audit task (task-37) includes reviewing the copy retention
  policy.
