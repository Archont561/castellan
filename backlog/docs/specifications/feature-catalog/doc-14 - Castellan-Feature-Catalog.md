---
id: doc-14
title: "Castellan Feature Catalog"
type: specification
created_date: '2026-10-02 21:05'
updated_date: '2026-10-02 21:05'
tags:
  - features
  - planning
  - product
  - specification
---
# Specification — feature catalog

**Scope.** Index of the convenience, privacy, local-first, sync, mobile and
developer features proposed during product brainstorming. Each feature is
recorded in a canonical backlog doc so future tasks can point at stable product
behavior instead of chat history.

This is not a delivery schedule. Task status and acceptance criteria live in
`backlog/tasks/`; this catalog names the product surface and routes it to the
right specification.

## §1 Canonical docs

| Doc | Area | Covers |
| --- | --- | --- |
| `doc-15` | Daily convenience | quick search, tray/menu bar, entry templates, generators, entry metadata helpers |
| `doc-16` | Browser extension | connection repair, current-site panel, fill policy, save/update, recovery capture |
| `doc-17` | Privacy, audit and recovery | audit dashboard, privacy report, permissions, clipboard, backups, diagnostics, emergency kit |
| `doc-18` | Developer workstation | CLI, env injection, project profiles, Git/SSH, signing, SOPS, containers, CI secrets |
| `doc-19` | Device mesh and local sharing | LAN sync, pairing, device trust, per-device policy, beam/share flows |
| `doc-20` | Mobile and platform integration | mobile quick actions, autofill/passkeys, biometrics, PIN, local notifications |
| `doc-12` | UI system | component contracts and ASCII layouts for app surfaces |
| `doc-13` | Local action log | every consequential action gets a user-visible local receipt |

## §2 Feature index

| Feature | Canonical doc | Summary |
| --- | --- | --- |
| Nearby device sync | `doc-19` | LAN-only paired-device sync with review and conflict handling |
| Quick receive / auto sync | `doc-19` | Auto-accept non-conflicting sync only from paired devices |
| Global quick search | `doc-15` | Keyboard launcher for entries and safe actions |
| Menu bar / system tray | `doc-15` | Lock, search, TOTP, generator and connection status from OS chrome |
| Current-site smart panel | `doc-16` | Extension/app panel centered on the browser's current origin |
| Extension repair flow | `doc-16` | Diagnose app/native-host/browser permission failures without manual debugging |
| Site-specific fill policy | `doc-16` | Per-origin fill, submit and preferred-entry rules |
| Save/update prompt with diff | `doc-16` | User-reviewed credential save/update decisions |
| Multi-step login helper | `doc-16` | Username-first/password-second flow without long-lived extension secrets |
| Recovery-code capture | `doc-16`, `doc-17` | Detect, review and store recovery codes structurally |
| Mobile quick-action sheets | `doc-20` | Touch-first copy/fill/TOTP/edit actions from rows |
| Native mobile autofill provider | `doc-20` | Android/iOS system credential-provider surfaces |
| Biometric quick unlock | `doc-20`, existing `doc-5` | Soft unlock via platform biometrics, hard-lock rules explicit |
| PIN quick unlock | `doc-20` | Device-local short-lived PIN quick unlock |
| Smart entry templates | `doc-15` | Structured login, note, token, SSH, recovery and identity templates |
| Domain rules UI | `doc-15`, `doc-16` | Explain and edit why entries match origins |
| Attachments with local previews | `doc-15` | KeePass-compatible attachments previewed locally |
| Generator profiles | `doc-15`, `doc-18` | Password/passphrase/token/PIN profiles with safe copy behavior |
| Password history rollback | `doc-15`, `doc-17` | Concealed history and explicit restore |
| Expiration reminders | `doc-15` | Local reminders for tokens, cards, certs, passwords and keys |
| Local audit dashboard | `doc-17` | Weak/reused/old/expired/no-2FA findings computed locally |
| Privacy report | `doc-17` | User-readable “what left this device?” report |
| Permission dashboard | `doc-17` | Browser/system/sync/mobile permission state and repair actions |
| Clipboard hygiene | `doc-17` | Clear timers, countdowns, history exclusion where supported |
| Screen privacy mode | `doc-17`, `doc-20` | Hide/blur secrets in app switcher, screenshots and screen sharing where possible |
| Panic lock | `doc-17` | Lock vault, clear clipboard, hide UI, stop discovery and disconnect sessions |
| Diagnostics bundle | `doc-17` | Redacted local support bundle, user-reviewed before export |
| Backup timeline | `doc-17` | User-visible copy-aside backup history |
| Restore diff review | `doc-17` | Review impact before restoring a backup |
| Emergency kit generator | `doc-17` | Printable vault recovery kit without storing master password |
| Unlock/recovery drill | `doc-17` | Local check that password/keyfile/backup recovery still works |
| QR/phrase pairing ceremony | `doc-19` | Pair devices with human-verifiable phrase and QR |
| Device trust levels | `doc-19` | Per-device capability/trust policy |
| Per-device sync policy | `doc-19` | Auto-sync, deletes, passkeys and review rules per device |
| Offline beam entry | `doc-19` | Time-limited local sharing of one entry or field bundle |
| LocalSend-compatible transfer | `doc-19` | Local transfer of backups/kits/diagnostics with sensitive-file warnings |
| CLI | `doc-18` | `castellan status/get/totp/generate/lock/unlock` |
| Scoped env injection | `doc-18` | `castellan run` injects secrets only into child processes |
| Project secret profiles | `doc-18` | Commit-safe `.castellan.toml` references, never values |
| Safe `.env` generation | `doc-18` | TTL-bound `.env` writes with shred/delete support |
| direnv integration | `doc-18` | Directory/repo-pinned grants to project profiles |
| Git credential helper | `doc-18` | Host-matched local Git credentials |
| SSH agent | `doc-18` | Vault-backed signing with per-use approval |
| Commit/signing keys | `doc-18` | SSH/GPG/age/minisign signing keys guarded locally |
| SOPS/age integration | `doc-18` | Local decrypt/sign workflows with receipts |
| Docker/Kubernetes dev secrets | `doc-18` | Explicit writes/injection into local containers/clusters |
| CI secret push | `doc-18` | Explicit egress to GitHub/GitLab/etc. with warnings and receipts |
| Developer TOTP helper | `doc-18` | CLI TOTP copy/watch/stdout with explicit policy |
| Developer token manager | `doc-18` | Token templates with scopes, expiration and health metadata |
| Developer secret health | `doc-18` | Local repo/secrets audit for dev workflows |
| Pre-commit secret guard | `doc-18` | Block obvious committed secrets, including values matching the vault |
| Localhost/dev origin rules | `doc-18`, `doc-16` | Temporary development origin mapping with expiry |
| WebAuthn developer lab | `doc-18` | Inspect local passkey ceremonies and RP/origin decisions |
| Local automation socket | `doc-18` | Same-user UDS/named-pipe API only; no localhost TCP secrets API |
| Project dashboard | `doc-18` | Per-project secret profile, command and policy overview |
| Secret access receipts | `doc-13`, `doc-18` | User-visible non-secret receipts for every dev secret use |
| Team/household local mode | `doc-19` | Future shared vaults over local mesh without hosted admin cloud |
| Local-only notifications | `doc-20`, `doc-17` | OS notifications for local events, no push service dependency |

## §3 Cross-cutting requirements

- Every feature that exposes a secret, changes state, changes trust, exports,
  syncs, imports, repairs or denies access must emit a local action-log receipt
  per `doc-13`.
- UI surfaces must satisfy the component and visual/spec review loop in
  `doc-12`.
- Features that send data outside the local device/LAN must say so before the
  action, require explicit user intent, and log the egress.
- Extension features must respect the containment contract: no vault, no bulk
  secrets, no TOTP seeds, and app-side origin/RP-ID enforcement.
- Device mesh features must not introduce a cloud relay.
