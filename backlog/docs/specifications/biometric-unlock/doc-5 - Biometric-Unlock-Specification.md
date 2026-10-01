---
id: doc-5
title: "Biometric Unlock Specification"
type: specification
created_date: '2026-10-01 21:15'
updated_date: '2026-10-01 21:15'
tags:
  - security
  - biometrics
  - specification
---
# Specification — biometric unlock

**Scope.** How a fingerprint or Face ID unlocks the vault without weakening
it (task-23, task-32). One architecture, five platforms.

## §1 The invariant

**Biometrics never decrypt anything and are never the only factor.** They
gate a hardware-protected key that wraps the vault key. The master password
remains the root; every derived convenience layer is revocable and
re-derivable from it.

## §2 Enrollment (after a successful password unlock)

1. Derive `K_master` (Argon2id, the file's parameters).
2. Ask the platform's secure hardware to create `K_hw` under a biometric
   access-control policy.
3. Store `Enc(K_hw, K_master)` + policy metadata, per device, never synced.

## §3 Unlock

Biometric prompt → hardware releases `K_hw` → unwrap `K_master` → vault
opens with no Argon2 re-run. Password is always the fallback path.

## §4 Platform matrix

| Platform | K_hw lives in | Invalidated on new fingerprint |
| --- | --- | --- |
| iOS | Secure Enclave key, `biometryCurrentSet`, `WhenUnlockedThisDeviceOnly` | automatic |
| Android | Keystore AES, `setUserAuthenticationRequired`, `setInvalidatedByBiometricEnrollment`, StrongBox when available | automatic |
| macOS | Secure Enclave / keychain ACL + Touch ID | automatic |
| Windows | Hello TPM key (Platform Crypto Provider) | Hello identity |
| Linux | fprintd verify + libsecret — **software gate, labeled as such** | best effort |

## §5 Lock policy interplay (task-15)

Soft lock (blur/sleep/OS lock, minutes) → biometric quick-unlock. Hard lock
→ password: after reboot, app update, biometric enrollment change, N days
(default 7), or 3 failed biometric attempts. Settings state the tradeoff
plainly: enabling biometrics makes the device itself a key.

## §6 Threat model honesty

A software-gated platform (Linux path) provides UX, not isolation — the
settings copy says so. On all platforms, a device that is stolen *unlocked*
is a device that is open; auto-lock exists to shrink that window, not to
eliminate it.
