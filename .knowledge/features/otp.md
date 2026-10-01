---
type: Reference
title: "Castellan — authenticator and OTP"
description: "otpauth parsing rules, RFC 6238 implementation policy, migration import formats."
tags:
  - otp
  - totp
  - authenticator
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: features/otp
category: features
refs:
  - project/standards
  - project/data-model
---
# Authenticator and OTP

`castellan-otp` parses and computes; apps use it natively, the extension
uses its WASM build for *validation only* (parsing without the secret ever
crossing into JS is enforced by the wasm face's return types).

## otpauth grammar (the landmines)

`otpauth://totp/Issuer:account?secret=…&issuer=…&digits=…&period=…&algorithm=…`
— an **unregistered** scheme, so parsers disagree. Castellan's rules:
percent-decode labels (real providers emit encoded colons and spaces);
the `issuer` query parameter wins over the path issuer (provider behavior);
digits/period defaults 6/30; `algorithm=SHA256/512` and `encoder=steam` are
**refused with `UnsupportedParameter`** — a wrong code computed silently is
worse than an honest error.

## Computation

RFC 6238 SHA-1 with dynamic truncation, tested against the RFC's Appendix B
vectors (T=59 → 287082 …). `code_at` exists separately from `code_now` so
tests pin vectors instead of freezing clocks. `code_now` returns remaining
seconds; **faces never reimplement period arithmetic** — the TotpRing
component is driven by protocol answers.

## Import formats

- **Google Authenticator export** — `otpauth-migration://` batch QRs with
  per-account review (task-13/24).
- **Aegis** — JSON/vault exports, including AES-GCM password-encrypted
  (documented format).
- **andOTP** — plain + encrypted.
- KeePassXC convention (`TOTP Seed` field) read alongside keepass-rs's
  `otp` field; writes use `otp`.

Steam guard seeds import with a note where the display encoder is
unsupported — the code computation remains correct.
