---
id: doc-11
title: "keepass-rs Write Safety Spike"
type: spike
created_date: '2026-10-01 21:50'
updated_date: '2026-10-01 21:50'
tags:
  - spike
  - kdbx
  - vault
---
# Spike — how dangerous is keepass-rs 0.15's save path, really?

**Status.** Research spike for task-8 (the round-trip harness); the save
policy it informs is decision-3.

**Question.** The library says KDBX writing is experimental. Experimental
how? What exactly can be lost?

## §1 What was established (reading 0.15's source, 2026-10-01)

- The 0.15 line rewrote the database model (EntryId/GroupId, EntryRef/GroupRef
  views, `Database::new()` + `root_mut().add_entry()` mutation, Value<T:
  Zeroize> field hygiene) and carries a real save path with a
  `DatabaseSaveError` — more mature than the 0.6-era "experimental" label
  suggests, but the label is still the crate's own claim and the corpus of
  real-world files it round-trips is untested by us.
- Historical loss mode (0.6–0.7 era, the reason for the label): fields the
  crate does not model were dropped on rewrite. 0.15 models far more
  (custom data, icons, autotype, history, attachments, recycle bin), but
  "modeled" ≠ "round-trips byte-faithfully", and inner-header edge cases are
  exactly where third-party files live.

## §2 The safety net (conclusion for task-8)

1. Copy-aside before every write (the file on disk is never the experiment).
2. The round-trip harness over a committed corpus is the regression test:
   open → save → reopen, compare every field. What it catches never reaches
   users.
3. Save through rename: write temp, fsync, rename over. A crashed save
   leaves the original, not a half-file.
4. Watch upstream: the day keepass-rs declares writing stable, the harness
   becomes belt-and-suspenders instead of the seatbelt.

## §3 Corpus shopping list (fixtures for the harness)

KeePassXC-authored (icons, autotype, history, recycle bin, attachments),
Strongbox-authored (its YubiKey CR entries), KeePassDX-authored (Android
quirks), a KDBX 3.1 legacy file, and a deliberately weird one (custom data
everywhere, 5-level groups, expired entries, TOTP in both field conventions).
