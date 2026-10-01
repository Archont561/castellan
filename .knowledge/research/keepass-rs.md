---
type: Research
title: "Castellan — keepass-rs internals"
description: "Version-0.15 API facts, the write-safety question, and the Value<T: Zeroize> hygiene model."
tags:
  - kdbx
  - keepass-rs
  - vault
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: research/keepass-rs
category: research
refs:
  - features/vault
---
# keepass-rs internals (0.15, as of 2026-10-01)

## API facts (learned the hard way — verified against the source)

- `Database::open(&mut dyn Read, DatabaseKey)` — takes a **reader**, not a
  path. `DatabaseKey::new().with_password(...).with_keyfile(&mut Read)?`.
- Errors live at `keepass::db::DatabaseOpenError` (not the crate root).
- Mutation: `Database::new(key)`, `root_mut()` **before** `add_entry()`;
  `entry.id().uuid().to_string()` is the stable entry id.
- Entry/Group access goes through `EntryRef`/`GroupRef` views that `Deref`
  to the underlying types.
- Field values are `Value<T: Zeroize>` — dropping the database handle wipes
  decrypted field material. Locking is "drop it", and the vault crate says
  so in its docs.

## The write-safety question

The crate labels KDBX saving experimental. The 0.15 line is a full model
rewrite (EntryId/GroupId, far more modeled: custom data, icons, autotype,
history, attachments, recycle bin) with a real `DatabaseSaveError` — more
mature than the 0.6-era label suggests. But "modeled" ≠ "round-trips
byte-faithfully", and the historical loss mode (unmodeled fields dropped on
rewrite) is exactly what third-party files exercise.

## Conclusion (drives decision-3 and task-8)

Copy-aside before every write + the round-trip harness over a fixture
corpus (KeePassXC-, Strongbox-, KeePassDX-authored, KDBX 3.1 legacy, one
deliberately-weird file). What the harness catches never reaches users.
Watch upstream; the day saving is declared stable the harness becomes
belt-and-suspenders. Full spike: `backlog/docs/spikes/keepass-rs-write/doc-11`.
