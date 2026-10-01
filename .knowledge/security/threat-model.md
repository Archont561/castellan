---
type: Playbook
title: "Castellan — threat model"
description: "Assets, adversaries, trust boundaries, and the residual risks stated honestly."
tags:
  - security
  - threat-model
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: security/threat-model
category: security
refs:
  - faces/extension
  - security/passkeys
  - features/device-mesh
---
# Threat model

## Assets

The KDBX file at rest; the unlocked vault in memory; secrets in transit
between app, extension, and page; TOTP seeds; passkey private keys; the
sync channel between devices.

## Adversaries

1. **Malicious web page** — wants credentials for itself or another origin.
2. **Compromised/malicious extension** (ours or another's) — wants vault
   bulk exfiltration.
3. **Local malware on a locked session** — wants clipboard, memory, or the
   unlocked vault.
4. **Device thief** — wants the unlocked window or the file at rest.
5. **Network attacker on the LAN** — wants sync traffic or mesh injection.
6. **Curious third party** — wants vault backups, emergency kit, or
   localhost services to query.

## Trust boundaries

- **Trusted**: the Rust core (vault, protocol, ipc) and the app process.
- **Semi-trusted**: the webview UI (render-only, no vault), the extension
  (bounded by the containment contract in
  [faces/extension](../faces/extension.md)).
- **Untrusted**: web pages, the LAN, other local processes.

Boundary rules: origin/RP-ID enforcement in the app, never only in the
extension; no localhost TCP anywhere; peer-credential checks on the socket;
same-user enforcement.

## Residual risks (said out loud)

- **Soft passkeys have no secure-element isolation.** A fully compromised
  device can export soft keys. Hardware-bound mode exists per-credential;
  two-credential guidance is the mitigation users can actually follow.
- **The webview renders secrets.** Any webview exploit is a memory
  exposure; auto-lock tiers shrink, not close, that window.
- **KDBX at rest is only as strong as the master password + Argon2id**;
  the audit's weak-password check is a security feature, not hygiene.
- **The Linux biometric path is a software gate** and is labeled as such
  in its settings copy.

## Rejected-by-design

KeePassHTTP-style localhost servers (any local process can query); DIY
caBLE; CTAP2 USB HID emulation; cloud relay for the mesh (a permanent
non-goal, roadmap do-not-cut #5).
