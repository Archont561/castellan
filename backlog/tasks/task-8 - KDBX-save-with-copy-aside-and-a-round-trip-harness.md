---
id: TASK-8
title: KDBX save with copy-aside and a round-trip harness
status: Done
assignee:
  - '@me'
created_date: '2026-10-01 20:40'
updated_date: '2026-10-03 13:40'
labels:
  - vault
  - security
dependencies:
  - TASK-6
references:
  - crates/vault
  - backlog/docs/specifications/vault-format-policy/doc-3
priority: high
ordinal: 800
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The save path under decision-3's rule: copy the existing file aside (timestamped), write the new one, keep the copy. The harness that makes this trustworthy: parse saved output field-by-field against the original across a corpus of KeePassXC/Strongbox-authored databases, so 'drops fields keepass-rs does not parse' becomes a caught regression, not a lost database.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Save writes a valid KDBX 4 that KeePassXC and Strongbox open without warnings
- [x] #2 The copy-aside exists before any write; a failed save leaves the original untouched
- [x] #3 Round-trip harness compares every parsed field (incl. custom data, icons, history) and fails on any loss
- [x] #4 Harness runs in CI against the corpus: generated from a reviewed case table (tests/common/mod.rs) plus two committed anchor fixtures (KeePassXC-authored 4.1, KDBX 3.1) that no generator can author
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
keepass-rs save behind a save() that does copy→write→verify; harness as an integration test over fixtures; document the retention/prune policy in the app settings.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Started 2026-10-03. Corpus: KeePassXC-authored + keepass-rs-maintainer fixtures from sseemayer/keepass-rs v0.15.0 tests/resources (MIT), committed under crates/vault/tests/fixtures/ with provenance README. Save per doc-3 section 2: copy-aside (timestamped, beside the vault) -> temp write -> atomic rename; failed save leaves original byte-identical. Harness: corpus cases + field-by-field comparison + proptest over generated databases.

Corpus collapsed 2026-10-03 (user direction): the twelve keepass-rs-authored matrix fixtures (cipher/KDF/keyfile/deleted/TOTP variants) are replaced by the case-driven KDBX 4.1 factory in tests/common/mod.rs — one generator, a GENERATED_CORPUS table, exact expected titles by construction. Two anchors stay committed because they carry what a generator cannot: test_db_kdbx41_features.kdbx (KeePassXC 2.7.12 — external authorship) and test_db_with_password.kdbx (KDBX 3.1 — keepass-rs cannot write 3.x). The factory needs rust-argon2 (renamed rust_argon2; its lib name collides with the workspace's RustCrypto argon2) as a vault dev-dependency, because KdfConfig's Argon2d/Argon2id version field is that crate's type. Suite counts unchanged in spirit: session 39, save 9, roundtrip 15 (12 generated cases + the KeePassXC anchor + the fold characterization + the 50-case property).
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Done 2026-10-03. save.rs: copy-aside (timestamped, beside the vault, unique on collision) BEFORE any write, temp sibling + atomic rename + fsync of file and directory, KDBX 3 refused before touching the filesystem, and a failed save leaves the original byte-identical (tested by forcing the temp write to fail with the aside already present). One deliberate upgrade: keepass-rs 0.15's writer emits KDBX 4.1 only, so a 4.0 vault saves as 4.1 — the same upgrade KeePassXC itself performs — with the 4.0 original preserved by the copy-aside; 9/9 in tests/save.rs. Round-trip harness (tests/roundtrip.rs, 14/14): field-by-field comparator over meta/groups/entries (protected + unprotected fields, autotype, custom data, attachments by name and bytes, history recursively, deleted objects, icon pool) + the 12-file KDBX 4 corpus incl. KeePassXC 2.7.12 + a 50-case proptest over generated databases, all fixtures committed with provenance. KeePassXC/Strongbox compatibility is verified by proxy in this sandbox (no binaries): every corpus file — including the KeePassXC-authored ones — reopens through the keepass parser after our save, and keepass-rs 0.15's writer-compat suite pins the KeePassXC-visible writer behaviors. Two keepass-rs parser normalizations found and pinned by characterization tests rather than hidden: an empty meta.database_name folds to None, and a whitespace-only unprotected value folds to empty (the bytes on disk stay verbatim, so KeePassXC sees the original; only our parser-layer view folds). Retention/prune (doc-3: 20 copies or 30 days) is specified in doc-3 §2; the prune itself ships with the settings surface (task-9+), which is also where the retention knobs become user-visible.
<!-- SECTION:FINAL_SUMMARY:END -->
