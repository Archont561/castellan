---
id: doc-7
title: "Passkey Provider Research"
type: research
created_date: '2026-10-01 21:30'
updated_date: '2026-10-01 21:30'
tags:
  - passkeys
  - webauthn
  - research
---
# Research — how third parties become passkey providers

**Question.** How does an app that is not the OS browser become the thing
that creates and asserts passkeys, per platform, in October 2026?

## §1 Desktop: extension-side interception (proven path)

1Password and Bitwarden implement software authenticators *inside* their
browser extensions: a content script injected at `document_start` overrides
`navigator.credentials.create/get` before the page reads them, and the
ceremony is routed to the vault. Caveats we inherit honestly: no secure
element isolation; phishing resistance is preserved only as long as the
override enforces RP ID — which is why Castellan enforces it in the app
(decision-6), not in the extension. Chrome MV3 requires MAIN-world injection
(`world: "MAIN"` in the manifest); Firefox needs the injected-`<script>`
technique; WXT abstracts the build difference, not the runtime one.

## §2 Android 14+: Credential Manager providers (first-class)

Android's Credential Manager allows third-party credential providers —
passwords *and* passkeys — system-wide (1Password, Enpass, Dashlane ship
this). The provider is a service returning `GetCredentialResponse` /
`CreateEntry` structures; user verification flows through
`BiometricPrompt`. Castellan's integration is a Kotlin Tauri plugin (task-30)
backed by the same dispatcher as every face.

## §3 iOS 17+: Password Manager API (AutoFill + ProvidesPasskeys)

Third-party password apps become providers through the AutoFill credential
provider extension; declaring `ProvidesPasskeys` in the extension's
capabilities adds passkey registration (`prepareInterface(forPasskeyRegistration:)`)
and assertion (`prepareInterfaceToProvideCredential` for passkey requests),
Face ID via `LAContext` throughout; excludedCredentials is iOS 18+ — gate it.
Data handoff between app and extension is the App Group + keychain; on iOS
the extension cannot spawn helpers, so the vault reader is embedded (task-31).

## §4 Windows: third-party providers announced, arrival TBD

Microsoft announced synced third-party passkey support for Windows Hello;
until it ships, Windows rides the extension path. Re-evaluate at m-4.

## §5 Rejected approaches

- **USB HID CTAP2 emulation** (pretending to be a dongle): needs kernel
  drivers on Windows, udev games on Linux; no cross-platform userland story.
- **DIY caBLE/hybrid** (phone as roaming key): spec-compliant but requires
  BLE proximity plus a relay tunnel; infrastructure, not a feature.
