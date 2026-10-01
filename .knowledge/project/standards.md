---
type: Reference
title: "Castellan — standards and formats"
description: "Every external specification Castellan reads, writes, or conforms to."
tags:
  - standards
  - formats
  - reference
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: project/standards
category: project
refs:
  - features/otp
  - features/device-mesh
  - security/passkeys
---
# Standards and formats

| Standard | Role in Castellan | Notes |
| --- | --- | --- |
| **KDBX 4** (KeePass) | the vault format | KDBX 3.1 read-supported; save writes 4 |
| **RFC 6238 / RFC 4226** | TOTP/HOTP | SHA-1 tested against Appendix B vectors; SHA-256/512 and Steam encoder refused, not approximated |
| **otpauth:// URI** (de-facto) | TOTP seed interchange | unregistered scheme — parsed by hand; percent-decoding and issuer precedence are landmines |
| **WebAuthn L3** (W3C) | passkey ceremonies | RP ID enforcement in the app process is Castellan's security boundary |
| **Native messaging** (Chromium) | extension ↔ host framing | 4-byte LE length, 1 MB cap; also our socket framing |
| **LocalSend protocol v2.2** | mesh transport interop | UDP multicast 224.0.0.167:53317 + HTTPS :53317, TOFU pinning |
| **HIBP k-anonymity API** | breach checks | 5-char SHA-1 prefix only; suffix never leaves the machine |
| **2fa.directory** | 2FA-availability hints | data bundled locally, checked offline |
| **Google Authenticator migration QR** (`otpauth-migration://`) | authenticator import | batch import, per-account review |
| **Aegis / andOTP export formats** | authenticator import | documented JSON/zip layouts, incl. encrypted |
| **Bitwarden export JSON** | importer | encrypted + plaintext variants |
| **KeeAgent fields** | SSH keys in KDBX | KeeAgent-compatible naming |
| **OKF v0.2** | this knowledge bundle | validated by `tools/validate_okf.py` |

Standing rule: adopt a documented format, never a reverse-engineered one
without labeling it as such (LocalSend is documented; that is why it was
chosen over, say, Syncthing's protocol for the mesh's file layer).
