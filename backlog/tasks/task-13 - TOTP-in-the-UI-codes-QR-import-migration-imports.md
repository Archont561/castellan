---
id: TASK-13
title: "TOTP in the UI: codes, QR import, migration imports"
status: Done
assignee:
  - '@agent'
created_date: '2026-10-01 20:40'
updated_date: '2026-10-05 08:15'
labels:
  - otp
  - ux
dependencies: [TASK-3,TASK-7]
references:
  - crates/otp
  - packages/ui/src/components/TotpRing.svelte
priority: high
ordinal: 1300
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Live codes with the TotpRing countdown (driven by seconds_remaining from the protocol, never client-computed periods), otpauth QR import from image file and screen capture, and the Google Authenticator migration QR format.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Codes tick in the ring component from protocol answers; no period arithmetic in TS
- [x] #2 otpauth:// URIs import from pasted text, image files, and screen capture (desktop)
- [x] #3 Google Authenticator export QRs (multi-account otpauth-migration:// payloads) parse and import as a batch with per-account review
- [x] #4 Aegis and andOTP exports import (their file formats are JSON/zip with documented layouts)
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Migration URI parsing in castellan-otp (it is protocol-shaped logic, shared with the WASM face); QR decode in the app via a Rust barcode crate; batch import UI with review.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
- **castellan-otp** grew two modules. `migration` parses
  `otpauth-migration://offline?data=…` with a hand-rolled protobuf reader
  (varints + length-delimited fields — the whole message grammar), because
  the airlocked vendor set carries no protobuf crate and the schema is
  seven scalar fields pinned by Google's own importer. `import::preview`
  is the one sniffing front door: pasted `otpauth://` lines, migration
  URIs, Aegis plain JSON, andOTP plain JSON. Every account the payload
  carried comes back as an `AccountCandidate` — importable ones with the
  URI to store, refused ones (HOTP, SHA-256/512, Steam) *named with the
  reason*, because a migration that silently shrinks is how people lose
  accounts. Encrypted Aegis exports are recognized (`db` as a base64
  string) and refused by name; decryption is task-24's AC, as is andOTP's
  encrypted blob (binary, never valid JSON, reports as unrecognized).
- **One deliberate deviation from the plan**: QR *pixel* decoding is jsqr
  (npm) on the desktop face, not a Rust barcode crate — the airlock
  vendors no QR/image crates and crates.io is unreachable, while npm is
  the one reachable registry. The boundary stays right: the face only
  turns pixels into text (`apps/desktop/src/lib/qr.ts`, image files via
  `createImageBitmap`, screen via `getDisplayMedia` with the track always
  stopped); every *meaning* — otpauth, migration batch, export JSON — is
  parsed app-side in castellan-otp behind `preview_otp_import`.
- **Protocol**: `preview_otp_import` (pure parsing, deliberately answers
  while locked) and `import_otp_accounts` (locked-first precedence, whole
  batch validated before anything lands) for Desktop+Mobile;
  `OtpImportCandidate`/`OtpImportSelection` wire types; new stable error
  code `bad_request` ("fix what you pasted", distinct from
  `not_implemented`). Additive, so PROTOCOL_VERSION stays 3. `get_totp`
  is implemented: the dispatcher reads the seed via the session, computes
  through castellan-otp, and the UI receives `seconds_remaining` — the
  period arithmetic never crosses the wire.
- **Vault**: `VaultHandle::otp_uri` (per-entry, by-id secret read — the
  list projection still carries only `has_totp`), `add_totp_entry`
  (stores the scanned URI verbatim under the KeePass `otp` field), and
  `VaultSession::import_totp_entries` — one copy-aside save per batch,
  rollback of the in-memory additions when the save fails (memory and
  disk never disagree), `entry_changed` events only after the save
  succeeds. New errors `NoSuchEntry`/`NoTotpSeed` both map to the wire's
  `no_such_entry`, with messages that keep "no 2FA on this entry" and
  "no such row" distinguishable. Locked answers first, before entry
  existence — a locked vault must not confirm which ids exist.
- **UI**: `TotpRing` gained `onExpired`; `TotpCode` owns the
  fetch→draw→refetch loop (remounts the ring per answer, keyed by the
  answer object, so equal-looking consecutive answers still restart the
  countdown); `VaultHome` shows a code per picked 2FA row, on demand only
  — no seed sweep of the vault to render a list. `OtpImport` renders the
  review: checkbox per importable account, named reasons for refused
  ones, import sends exactly what stayed checked. The desktop page wires
  the two QR callbacks; mobile's client already exposes the ops for when
  its import surface lands (task-24).
- **Found during the gates, filed as task-49**: the shells env cannot
  compile gio-sys in the airlock. Two layered defects — pkg-config's
  baked pc_path dangles after pixi-sandbox's staged restore (fixed
  in-repo: the gtk-shells feature now exports PKG_CONFIG_PATH from its
  activation), and the env carries libzlib but not zlib's .pc/headers,
  which needs a connected-side relock. The `@castellan/rust` lint lane
  is red in this sandbox for that pre-existing reason; every other lint
  lane (21/22) is green.
- Validation: otp 41, vault 79, dispatch 48+, protocol — 281 Rust tests
  total (baseline 242), all passing; 23/23 turbo lanes; 23 Playwright
  component tests (ticking + refetch proven against a scripted 1-second
  answer; review/import flows); desktop e2e 2/2 including the new import
  surface; desktop/mobile/ui/protocol/core typechecks; biome; cargo fmt;
  clippy -D warnings (non-Tauri workspace); storybook build; codegen
  byte-idempotent. Screen capture itself has no automated test —
  `getDisplayMedia` needs a fake-media-flagged browser; the capture
  button's presence is e2e-pinned and the decode path shares its code
  with the tested image-file path.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
TOTP is now end-to-end: `get_totp` computes app-side and the ring only
draws (and re-asks) — no period arithmetic in TS; the desktop face
imports authenticators from pasted text, QR images, and screen capture,
with the Google migration batch and plain Aegis/andOTP exports parsed in
castellan-otp behind `preview_otp_import`, reviewed per-account, and
stored through one copy-aside save that announces each entry. Refused
accounts (HOTP, SHA-2, Steam, encrypted exports) are shown with reasons
and routed to task-42/task-24, which own exactly those gaps.
<!-- SECTION:FINAL_SUMMARY:END -->
