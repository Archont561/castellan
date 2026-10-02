---
id: doc-20
title: "Mobile and Platform Integration Feature Specification"
type: specification
created_date: '2026-10-02 21:05'
updated_date: '2026-10-02 21:05'
tags:
  - mobile
  - platform
  - autofill
  - biometrics
  - specification
---
# Specification — mobile and platform integration features

**Scope.** Mobile-native and OS integration features that make Castellan useful
outside the desktop app: mobile quick actions, Android/iOS autofill/passkeys,
biometric/PIN unlock, local notifications and platform privacy behavior.

## §1 Mobile quick-action sheets

User job: act on entries without navigating deep detail pages.

Sheet actions:

- fill where platform/browser context exists;
- copy username;
- copy password;
- copy TOTP;
- open URL;
- edit;
- move to trash.

Rules:

- Bottom sheet on mobile, dropdown/context menu on desktop.
- Destructive actions are separated and styled as danger.
- Copy actions emit receipt and use clipboard hygiene.

## §2 Android credential provider and autofill

Requirements:

- Android 14+ Credential Manager provider for passwords and passkeys where
  supported.
- Autofill service or provider path for password fill.
- BiometricPrompt gate according to lock policy.
- App-side origin/RP-ID enforcement; provider UI is not authoritative.
- Local action receipts for fill/passkey operations.

## §3 iOS AutoFill and passkey provider

Requirements:

- AutoFill credential provider extension.
- ProvidesPasskeys capability where available.
- Face ID/Touch ID via platform APIs.
- App Group/keychain handoff without cloud accounts.
- Availability gates for iOS-version-specific APIs.

## §4 Biometric quick unlock

States:

- unlocked;
- soft locked: biometric/PIN may unlock;
- hard locked: master password required;
- no database.

Hard-lock triggers:

- reboot;
- app update according to policy;
- biometric enrollment change;
- N days since password entry;
- too many failed biometric attempts.

References: existing biometric unlock specification (`doc-5`).

## §5 PIN quick unlock

Requirements:

- Device-local and short-lived.
- Does not replace master password.
- Failed attempts hard-lock after configured threshold.
- Expires on reboot, timeout or policy event.
- PIN enrollment/change/removal logged.

## §6 Local-only notifications

Allowed notifications:

- vault locked/unlocked;
- clipboard cleared;
- sync completed/failed;
- backup created;
- expiration reminders;
- recovery drill due;
- extension/browser repair needed.

Rules:

- No remote push service required for local events.
- Notification text omits secrets and can omit entry titles in privacy mode.
- Tapping notification opens relevant local app surface.

## §7 Platform privacy behavior

Requirements:

- Respect safe-area insets and OS accessibility settings.
- Reduced motion support.
- Hide/blur secrets in app switcher where possible.
- Android screenshot-blocking support where available and user-enabled.
- Best-effort labels where the platform cannot enforce privacy.

## §8 Mobile-first onboarding

Requirements:

- Explain local vault ownership and no-account/no-cloud stance.
- Offer open existing KDBX, create vault, pair with desktop and import.
- Pairing with desktop uses QR/phrase flow from `doc-19`.
- Biometric enrollment is optional and explains hard-lock fallback.
