---
id: doc-16
title: "Browser Extension Convenience Feature Specification"
type: specification
created_date: '2026-10-02 21:05'
updated_date: '2026-10-02 21:05'
tags:
  - extension
  - autofill
  - passkeys
  - privacy
  - specification
---
# Specification — browser extension convenience features

**Scope.** Fob, the Castellan browser companion. It should feel convenient but
remain contained: no vault, no bulk secrets, no TOTP seeds, no app-side security
decisions delegated to the extension.

## §1 Current-site smart panel

User job: fill or copy for the current site without browsing the whole vault.

Surface:

```text
Fob                 Connected
Current site
github.com

Matching entries
GitHub
ada@castellan.dev
[Fill] [TOTP] [More]

Vault
Unlocked
[Open Castellan] [Lock vault]
```

Requirements:

- Current origin is visible before any fill/copy action.
- Matching entries come from the app after origin validation or from extension
  pre-filtering that is revalidated by the app.
- Locked/disconnected/no-database states are explicit.
- Popup is not a full vault manager.

## §2 Repair connection flow

User job: fix Fob without hand-editing native messaging manifests.

Diagnoses:

- Castellan app not running;
- native host manifest missing;
- native host path stale;
- browser permission missing;
- protocol version mismatch;
- app too old;
- extension too old.

Actions:

- open Castellan;
- repair browser connection;
- show diagnostics;
- copy redacted support summary.

All repair attempts and outcomes are logged locally.

## §3 Site-specific fill policy

User job: tune behavior per origin.

Policy fields:

- fill username automatically: never/ask/yes;
- fill password: never/ask/yes;
- submit after fill: never/ask/yes, default never;
- preferred entry;
- subdomain behavior;
- dev-origin expiry;
- never-match domains.

Rules:

- Global automatic password fill defaults off.
- Submit-after-fill requires per-site explicit opt-in.
- Policy decisions are visible in "Why is this shown here?".

## §4 Save/update prompt with diff review

User job: save new or changed credentials without blind writes.

Surface:

```text
Update GitHub?

Username
  unchanged

Password
  changed now

URL
  github.com

[Update entry] [Create new] [Ignore]
```

Requirements:

- Shows metadata diff, never old/new password text.
- App performs final save and backup.
- Extension sends one candidate change at a time.
- Ignore can be one-time or remembered for the origin/session.

## §5 Multi-step login helper

User job: handle username-first and password-second sites.

Requirements:

- Track tab-scoped, short-lived state only.
- Clear state on navigation, timeout, tab close, lock or origin change.
- Never store password in extension between steps; request it from app only when
  needed and user-approved.
- Log each fill step as a separate receipt.

## §6 Recovery-code capture

User job: save recovery codes when a site displays them.

Requirements:

- Detection suggests capture but requires user review.
- Preview count and issuer/origin; code bodies are visible only in the reviewed
  capture surface, not extension logs.
- Save as structured recovery-code entry with per-code used state.
- Recovery code bodies are excluded from search indexing.

## §7 "Why is this shown here?" explanation

User job: trust and debug autofill suggestions.

Surface:

```text
Showing GitHub because:
  page origin: https://github.com
  entry URL: https://github.com
  match: exact host
```

Requirements:

- Available from popup and injected overlay.
- Includes app-side validation result.
- For denied suggestions, shows non-secret reason code.
- Explanation views emit no secret exposure receipt unless they reveal secret
  data, which they must not.

## §8 Localhost and development-origin rules

User job: use Castellan on local development apps safely.

Requirements:

- Dev rules are explicit, scoped and expiring.
- Rule text says which local origin can request which real/staging entry.
- Broad localhost rules warn and default to short TTL.
- Activity logs creation, use and expiry of dev rules.

## §9 Passkey ceremony visibility

User job: understand passkey interception and app-side enforcement.

Requirements:

- Extension shows RP ID/origin and credential display name before user action
  where the browser flow allows UI.
- App validates RP ID/origin before private key use.
- Logs ceremony request, allowed/denied result and RP/origin metadata; never
  private keys or signed assertions.
