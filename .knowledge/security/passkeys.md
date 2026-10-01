---
type: Specification
title: "Castellan — passkeys (the soft security key)"
description: "Soft WebAuthn authenticator with app-side RP-ID enforcement; hardware binding opt-in per credential."
tags:
  - passkeys
  - webauthn
  - security
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: security/passkeys
category: security
refs:
  - research/passkey-providers
  - security/threat-model
  - faces/extension
---
# Passkeys — the soft security key

The flagship (decision-6): a WebAuthn authenticator whose keys live in the
vault, usable everywhere, recoverable via the mesh — **soft by default,
hardware-bound opt-in per credential**.

## Where enforcement lives

The extension (or mobile provider) routes a ceremony; **the app** validates
the requesting origin against the stored credential's RP ID before any key
is touched. The page's `navigator.credentials` override is untrusted input;
the authenticator's decisions are made in the trusted process. A malicious
page cannot ask for another site's key because it never reaches the key.

## Storage

Keys are vault entries: `castellan.passkey` custom field carrying the RP
ID, protected fields carrying key material + metadata. Foreign KeePass
clients render them as inert custom fields — no parse errors, no lock-in.

## Ceremony policy

- Per-use verification (biometric/platform auth) configurable, default on
  for assertion.
- `residentKey`/discoverable credentials supported (keys are in the vault,
  not in a fixed slot budget).
- `excludedCredentials` honored on both platforms that expose it (Android,
  iOS 18+).

## The honest trade (product copy, not fine print)

A software authenticator is only as strong as the device. Hardware-bound
mode stores the key in TPM/Secure Enclave/StrongBox — it never leaves, and
it **dies with the device**; the UI says exactly that and recommends a
second, synced credential for accounts that matter. Platform paths:
[passkey provider research](../research/passkey-providers.md).
