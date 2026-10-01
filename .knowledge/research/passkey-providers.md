---
type: Research
title: "Castellan — passkey provider research"
description: "How a third-party app becomes a credential provider on each platform (findings, October 2026)."
tags:
  - passkeys
  - webauthn
  - android
  - ios
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: research/passkey-providers
category: research
refs:
  - security/passkeys
  - faces/mobile
  - faces/extension
---
# Passkey provider research

*Findings as of 2026-10-01 — re-verify before m-4 planning; this space
moves quarterly.*

## Desktop: extension-side interception (the proven path)

1Password and Bitwarden implement software authenticators inside their
browser extensions: a `document_start` content script overrides
`navigator.credentials.create/get` before the page reads them. Chrome MV3
needs MAIN-world injection (`world: "MAIN"` in the manifest); Firefox needs
the injected-`<script>` technique; WXT abstracts the build difference, not
the runtime one. The technique inherits an honest weakness — no secure
element — which is why Castellan enforces RP ID in the app process
([passkeys](../security/passkeys.md)).

## Android 14+: Credential Manager providers

Third-party credential providers are first-class: a provider service
returns `GetCredentialResponse` / `CreateEntry`; user verification flows
through `BiometricPrompt`; 1Password, Enpass, Dashlane ship this today.
Castellan's path: a Kotlin Tauri plugin over the shared dispatcher
(task-30).

## iOS 17+: Password Manager API

AutoFill credential-provider extension; `ProvidesPasskeys` capability adds
`prepareInterface(forPasskeyRegistration:)` and passkey assertion; Face ID
via `LAContext`; `excludedCredentials` is iOS 18+ — gate by availability.
The extension cannot spawn helpers: the vault reader is embedded, data
handoff via App Group + keychain (task-31).

## Windows: announced, watch

Microsoft announced synced third-party passkey support for Windows Hello;
not shipped when last checked. Until it is, Windows rides the extension
path. Re-check at m-4 kickoff.

## Rejected (with reasons)

- **USB HID CTAP2 emulation** — kernel drivers on Windows, udev on Linux;
  no cross-platform userland story.
- **DIY caBLE/hybrid roaming** — BLE proximity + relay tunnel; that is
  infrastructure, not a feature, and out of scope.
