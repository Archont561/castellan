---
type: Reference
title: "Castellan — vault"
description: "KDBX as the format, copy-aside as the save rule, unlock methods and what each exposes."
tags:
  - vault
  - kdbx
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: features/vault
category: features
refs:
  - project/data-model
  - security/biometric-unlock
  - research/keepass-rs
---
# Vault

The vault is a KDBX file the user owns (decision-3). Read: KDBX 3.1 + 4,
password / keyfile / both, inner-stream fields, custom data, attachments,
history, recycle bin — proven against a fixture corpus of KeePassXC- and
Strongbox-authored files.

## Save policy (the rule)

Copy-aside before every write → temp-file write → fsync → rename. The
original is never the experiment; a crashed save leaves it byte-identical.
Retention: 20 copies or 30 days, prunable. The round-trip harness (open →
save → reopen → compare every field) is the regression test that turns
"keepass-rs dropped a field" into a red build instead of a lost database —
findings in [keepass-rs research](../research/keepass-rs.md).

## Unlock methods

| Method | What it derives | When it re-prompts |
| --- | --- | --- |
| Master password (+ keyfile) | `K_master` via Argon2id | hard lock: reboot, update, N days |
| Biometric | unwraps stored `Enc(K_hw, K_master)` | soft lock, enrollment change |
| YubiKey challenge-response | part of the KDBX key (KeePassXC-compatible) | with the password |

## Session and locking

Unlock opens an in-memory session; drop wipes derived keys (keepass 0.15's
`Value<T: Zeroize>` makes this a property of the types). Lock tiers: soft
(blur/sleep/OS lock/timer), hard (reboot/update/enrollment change/N days),
panic hotkey. `lock_database` never fails; locked answers are
`vault_locked`, never errors. Events `database_locked`/`unlocked` keep
faces live.

## Importers (arrival path)

Bitwarden JSON (encrypted + plain), Chrome/Edge CSV, KeePassXC CSV —
preview-and-confirm, nothing written before approval, TOTP seeds preserved.
Full policy doc: `backlog/docs/specifications/vault-format-policy/doc-3`.
