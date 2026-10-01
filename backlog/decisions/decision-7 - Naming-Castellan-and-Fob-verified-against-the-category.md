---
id: decision-7
title: "Naming — Castellan (product) and Fob (companion), verified against the category"
date: '2026-10-01 20:35'
status: accepted
---
## Context

Password-manager naming is a minefield: the Kee-/Pass-/Vault-/Warden- families are legally
and cognitively crowded (KeePass, Bitwarden, Proton Pass, Padloc, Strongbox, Azure Key
Vault). A name that reads as a clone is a name nobody recommends. Candidates were checked
against the live landscape in October 2026.

## Decision

**Castellan** — the keeper of the castle keys — for the product; **Fob** — the thing you
carry that grants access — for the browser companion and passkey layer. Verified at the
time: `Fob` clean in-category; `Castellan` has only unrelated hobby projects (an AI
Windows monitoring tool, an archived smart-home repo); rejected for direct collisions:
Latchkey (active local-first password manager), Coffer (active Rust Apple-Passwords
client), Keysmith (KDE TOTP app), Zamek (dormant Polish hardware password keychain, same
category and country), Scytale (known Android keystore library).

## Consequences

- Before any public artifact: EUIPO/WIPO search (classes 9, 42), GitHub org, crates.io/npm
  names, store searches, and the `.app` domain. The research doc (research/naming in the
  knowledge base) carries the full collision table.
- The `castellan-*` crate prefix and `@castellan/*` npm scope are already locked in by the
  scaffold; renaming later would be cheap now and expensive after first release.
- `Fob` doubles as CLI-friendly branding for the companion surface; the CLI binary stays
  `castellan`.
