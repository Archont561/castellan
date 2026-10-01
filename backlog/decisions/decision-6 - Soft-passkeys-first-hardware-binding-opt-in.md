---
id: decision-6
title: "Soft passkeys first; hardware binding is an opt-in mode, not the default"
date: '2026-10-01 20:30'
status: accepted
---
## Context

The product's flagship feature is being the "soft YubiKey": a WebAuthn authenticator whose
keys live in the vault. Two architectures compete: sync-friendly software keys (recoverable
via device mesh, usable everywhere) or hardware-bound keys (TPM/Secure Enclave/StrongBox,
non-exportable, die with the device). Users will want different answers for different
accounts, and the difference is a security trade they must be able to see.

## Decision

**Soft keys are the default**; keys stored encrypted in the vault, ceremonies re-verified
by biometrics, origin/RP-ID enforcement in the app (never only in the extension).
**Hardware-bound mode is per-credential opt-in** with an explicit "dies with the device"
warning and a recommendation to register a second, synced credential at relying parties
that matter.

The desktop implementation is extension-side interception of `navigator.credentials`
(the 1Password/Bitwarden technique); mobile uses the platform provider APIs (Android 14
Credential Manager, iOS 17 AutoFill + `ProvidesPasskeys`). USB HID CTAP2 emulation and DIY
caBLE are rejected as out of scope.

## Consequences

- The honest threat-model statement (software authenticator = no secure element isolation;
  a fully compromised device can export soft keys) is product copy, not fine print.
- The recovery story is coherent: lose a device with hardware-bound keys and the synced
  soft key still gets you in — hence "two credentials per critical site" guidance.
- Extension compromise is contained by app-side RP-ID enforcement; the extension never
  holds vault data, only answers.
