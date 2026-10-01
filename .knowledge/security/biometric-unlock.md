---
type: Specification
title: "Castellan — biometric unlock"
description: "Biometrics gate a hardware key that wraps the vault key; the master password stays the root."
tags:
  - security
  - biometrics
  - key-management
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: security/biometric-unlock
category: security
refs:
  - features/vault
  - faces/mobile
  - security/threat-model
---
# Biometric unlock

**Invariant: biometrics never decrypt anything and are never the only
factor.** They gate a hardware-protected key (`K_hw`) that wraps the vault
key (`K_master`). The master password remains the root; every convenience
layer is revocable.

## Flow

```
unlock (password):  K_master = Argon2id(pw, kdf params)
enroll biometric:   K_hw  = platform secure hardware (biometric-gated)
                    store Enc(K_hw, K_master) per device, never synced
unlock (biometric): prompt → hardware releases K_hw → unwrap K_master
                    (no Argon2 re-run; password always the fallback)
```

## Platform matrix

| Platform | K_hw | New-biometric invalidation |
| --- | --- | --- |
| iOS | Secure Enclave, `biometryCurrentSet`, `WhenUnlockedThisDeviceOnly` | automatic |
| Android | Keystore AES, StrongBox when available | automatic (KeyInvalidated) |
| macOS | Secure Enclave / keychain ACL + Touch ID | automatic |
| Windows | Hello TPM key (Platform Crypto Provider) | Hello identity |
| Linux | fprintd + libsecret — software gate, labeled | best effort |

## Lock tiers interplay

Soft lock (blur/sleep/OS lock) → biometric quick-unlock. Hard lock →
password: reboot, app update, biometric enrollment change, N days (default
7), 3 failed biometric attempts. Enrollment change invalidating `K_hw` is
the property that makes a new household fingerprint not a new vault key.

Full policy in `backlog/docs/specifications/biometric-unlock/doc-5`;
implementation tasks 23 and 32.
