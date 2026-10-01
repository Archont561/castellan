---
type: Research
title: "Castellan — naming research"
description: "The Castellan/Fob collision table (October 2026) and the pre-launch re-verification checklist."
tags:
  - naming
  - branding
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: research/naming
category: research
refs:
  - project/overview
---
# Naming research

*Checked October 2026; names get taken — re-verify before anything public.*

## Policy

Avoid Kee-/Pass-/Vault-/Warden- prefixes entirely: they read as KeePass
clones and sit on registered marks (KeePass, Bitwarden, Proton Pass,
Padloc, Strongbox, Enpass, Passbolt, Azure Key Vault).

## The table

| Name | Verdict | Finding |
| --- | --- | --- |
| **Fob** | ✅ clean | unrelated car-security product only |
| **Castellan** | ✅ usable | hobby projects only, none in-category |
| Latchkey | ❌ | active local-first password manager (2025) |
| Coffer | ❌ | active Rust Apple-Passwords client for Linux |
| Keysmith | ❌ | KDE TOTP authenticator |
| Zamek | ⚠️ | dormant Polish hardware password keychain — same category, same market |
| Scytale | ⚠️ | established Android keystore library |
| Jackdaw | ⚠️ | Clojure Kafka library, Bevy editor |
| Strongbox / Haven / Aegis / Sesame / KeyNest | ❌ | existing security products |

## Why these two

**Castellan** — the keeper of the castle keys; a real Polish word
(*kastelan*), works for the Warsaw-built, local-first positioning; team
headroom later. **Fob** — the thing you carry that grants access; three
letters, names both the companion extension and the soft security key.

## Pre-launch checklist (decision-7)

EUIPO + WIPO (classes 9, 42); USPTO if US-first; GitHub org; `castellan` on
crates.io; `@castellan` npm scope; App Store / Play Store searches;
`castellan.app` / `fob.app` domains. The `castellan-*` crate prefix and
`@castellan/*` scope are already committed in-repo; a rename after first
release would be a migration, so this checklist runs before 0.1 ships
anywhere public.
