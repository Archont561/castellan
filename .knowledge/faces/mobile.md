---
type: Reference
title: "Castellan — mobile face"
description: "Tauri mobile apps; the credential-provider work; biometric gating of everything."
tags:
  - mobile
  - android
  - ios
  - passkeys
status: draft
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: faces/mobile
category: faces
refs:
  - security/biometric-unlock
  - research/passkey-providers
  - project/architecture
---
# Mobile face

Same SvelteKit UI kit as desktop, packaged by Tauri's Android/iOS targets.
The app-specific work is not UI — it is the platform provider surfaces that
make Castellan a system credential source.

## Android (task-23, task-30)

- Biometric unlock plugin: Keystore AES key with
  `setUserAuthenticationRequired` + `setInvalidatedByBiometricEnrollment`
  (StrongBox where available) wrapping the vault key.
- Credential Manager provider (Android 14+): passkey create/assert +
  password autofill, system-wide, gated by the biometric prompt.
- Tauri mobile plugins are Kotlin shims over the same dispatcher — no
  second protocol.

## iOS (task-31)

- AutoFill credential-provider extension declaring `ProvidesPasskeys`;
  registration + assertion with Face ID via `LAContext`; QuickType
  suggestions.
- App Group + keychain handoff between main app and extension; the
  extension embeds the vault reader (it cannot spawn helpers).
- `excludedCredentials` handling is iOS 18+ — gate by availability.

## Status honesty

This concept is **draft**: the provider integrations are researched (see
[passkey providers](../research/passkey-providers.md)) and specified, but
none of the code exists yet — tasks 23/30/31 carry the work. The scaffold
contains the mobile app crate with the UI kit wired, not the providers.
