---
id: doc-3
title: "KDBX Compatibility and Save Policy"
type: specification
created_date: '2026-10-01 21:10'
updated_date: '2026-10-01 21:10'
tags:
  - vault
  - kdbx
  - security
  - specification
---
# Specification — KDBX compatibility and the save policy

**Scope.** What it means to promise KeePass compatibility, and the rules
that make saving safe while keepass-rs writing is experimental (decision-3).

## §1 Read compatibility (the promise)

KDBX 3.1 and KDBX 4, password / keyfile / both; inner-stream-protected
fields; custom data; attachments; history; recycle bin. Tested against a
corpus of KeePassXC- and Strongbox-authored files. If a real-world file opens
in KeePassXC, it opens in Castellan — and a file that does not is a bug with
a fixture attached.

## §2 Save policy (the rule)

1. **Copy aside** the current file (timestamped, beside the vault) before any
   write.
2. Write the new file (temp name, then rename — never a partial overwrite).
3. Keep the copy until retention prunes it (default: 20 copies or 30 days).
4. A failed save must leave the original byte-identical (test asserts it).

## §3 Round-trip harness (the proof)

Integration test over the fixture corpus: open → save → reopen → compare
every parsed field (titles, protected values, custom data, icons, autotype,
history, attachments). Any field loss fails CI. The harness exists so
"keepass-rs dropped a field" is a red build, not a lost database.

## §4 Custom concepts, foreign-client-safe

- Passkeys: `castellan.passkey` custom field (value: RP ID) + encrypted key
  material in a protected field. KeePassXC renders "a weird custom field";
  nothing errors.
- Recovery codes: structured custom fields (`castellan.recovery.<n>`), used
  state in `castellan.recovery.used`.
- TOTP: keepass-rs's `otp` field first, `TOTP Seed` (KeePassXC convention)
  second — read both, write `otp`.

## §5 Emergency kit

Printable kit from unlock-time data: vault path, KDF parameters, keyfile QR.
The master password is written by hand on paper — it is not recoverable by
design, and the kit says exactly that (task-35).
