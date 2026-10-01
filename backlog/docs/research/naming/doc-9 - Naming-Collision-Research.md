---
id: doc-9
title: "Naming Collision Research"
type: research
created_date: '2026-10-01 21:40'
updated_date: '2026-10-01 21:40'
tags:
  - naming
  - research
---
# Research — the name, checked against the landscape

**Question.** What can this product be called that survives a trademark
glance, a web search, and a password-manager-savvy reader's cynicism?

## §1 The crowded prefixes

Anything Kee-/Pass-/Vault-/Warden- reads as a KeePass clone and sits next to
registered marks: KeePass (D. Reichl), Bitwarden, Proton Pass, Padloc,
Strongbox, Enpass, Passbolt, Azure Key Vault. Avoided by policy.

## §2 Candidates checked (October 2026)

| Name | Verdict | Finding |
| --- | --- | --- |
| **Fob** | ✅ clean in-category | only a car-key security product, unrelated; short enough for a CLI and a sub-brand |
| **Castellan** | ✅ usable | unrelated hobby projects only (AI security monitor, archived smart-home repo); no password manager, no commercial mark found |
| Latchkey | ❌ | active local-first password manager (jkalend/latchkey, v0.2.0 2025) |
| Coffer | ❌ | active Rust Apple-Passwords client for Linux (dahlia/coffer) |
| Keysmith | ❌ | KDE's TOTP authenticator |
| Zamek | ⚠️ | dormant Polish hardware password keychain (jareklupinski/zamek) — same category, same market |
| Scytale | ⚠️ | established Android keystore/crypto library |
| Jackdaw | ⚠️ | Clojure Kafka library + a Bevy editor |
| Strongbox, Haven, Aegis, Sesame, KeyNest | ❌ | existing security products / auth apps |

## §3 Chosen: Castellan + Fob

Castellan — the keeper of the castle keys; works in Polish (*kastelan*, a
real Polish word with castle history); mascot headroom (a warden with a
keyring); team features later ("the castellan manages the castle for the
household"). Fob — the thing you carry that grants access — names the
companion and the soft security key in three letters.

## §4 Before anything public (the checklist)

EUIPO + WIPO trademark search (classes 9, 42); USPTO if the US matters;
GitHub org; `castellan` on crates.io; `@castellan` npm scope; App Store /
Play Store searches; `castellan.app` / `fob.app` domains. Re-verify the
table above — this research is dated and names get taken.
