---
id: doc-15
title: "Daily Convenience Feature Specification"
type: specification
created_date: '2026-10-02 21:05'
updated_date: '2026-10-02 21:05'
tags:
  - ux
  - convenience
  - vault
  - specification
---
# Specification — daily convenience features

**Scope.** Features that make Castellan fast for everyday use while preserving
local-first and secret-minimizing behavior. Browser-specific and mobile-native
surfaces are specified in `doc-16` and `doc-20`; this document focuses on the
shared app experience.

## §1 Global quick search / command palette

User job: act on entries without opening the full vault hierarchy.

Required capabilities:

- Open with platform shortcuts (`Cmd/Ctrl+K` in-app; optional global hotkey from
  the tray task).
- Search entries by metadata only: title, URL/host, username, tags, groups and
  type. Recovery code bodies and secret values are excluded.
- Run safe commands: lock vault, generate password/passphrase, open settings,
  open connected devices, open audit, start sync.
- Entry actions: copy username, copy password, copy TOTP, open URL, edit entry.
- Locked state shows unlock/open-vault commands, not entry secrets.

Security and logging:

- Copy/reveal/fill-like commands emit action-log receipts.
- Password copy and TOTP copy do not reveal values in UI or logs.
- Global OS hotkey requires a visible setting and can be disabled.

## §2 Menu bar / system tray mini mode

User job: reach common local actions from OS chrome.

Surface:

```text
Castellan
  Unlocked · soft-lock in 12m
  Search vault…
  Copy TOTP…
  Generate password…
  Lock now
  Connected browsers
    Firefox · active
    Chrome · active
```

Requirements:

- Shows vault state: no database, locked, unlocked, soft-locked, disconnected
  from backend, sync/discovery active.
- Offers lock now and panic lock where supported.
- Opens app to exact settings/activity/sync pages.
- Does not show secret values in the OS menu.

## §3 Smart entry templates

User job: create structured entries quickly.

Templates:

- login;
- secure note;
- recovery codes;
- SSH key;
- software license;
- Wi-Fi password;
- database credential;
- API token;
- credit card;
- identity;
- passkey.

Rules:

- Templates create typed fields, not unstructured note blobs.
- Foreign KeePass clients must see compatible custom fields, not broken data.
- Template choice is reversible until first save.
- New templates define search metadata and secret fields explicitly.

## §4 Domain rules UI

User job: understand and control why an entry matches an origin.

Surface:

```text
Matches
  github.com                 exact host
  gist.github.com            single subdomain
  localhost:5173             dev rule · expires in 8h

Never match
  evil-github.example
```

Requirements:

- Explains the protocol origin matching rule in human language.
- Shows page origin, entry URL/host and match reason.
- Supports exact host, subdomain, custom dev rule and never-match overrides.
- Dangerous broad/dev rules require expiry and are visible in Activity.

## §5 Attachments with local previews

User job: keep local supporting files with entries.

Requirements:

- Store KeePass-compatible attachments.
- Preview common safe types locally: text, images, PDFs where platform supports
  it. Unknown files show metadata only.
- Warn before adding large attachments because they affect vault size, backups
  and sync.
- Opening/exporting an attachment emits an action-log receipt.

## §6 Generator profiles

User job: generate appropriate secrets without reconfiguring every time.

Profiles:

- human passphrase;
- website password;
- PIN;
- Wi-Fi password;
- API token;
- base64/base64url/hex bytes;
- recovery-code-like chunks.

Requirements:

- Profiles have names, parameters and examples.
- Generated output is not saved until the user explicitly applies or copies it.
- Copy emits a receipt; generator settings changes emit setting receipts.
- Entropy/strength text is near the generated value.

## §7 Password history and rollback

User job: recover from a bad password change.

Requirements:

- Show password history concealed by default.
- Copy/reveal old password requires explicit action.
- Restore previous password creates a new current value rather than deleting
  history silently.
- History rollback records entry, time, face and outcome in Activity.

## §8 Expiration reminders

User job: rotate entries before they fail.

Entry types:

- passwords;
- API tokens;
- SSH/signing keys;
- certificates;
- credit cards;
- recovery codes;
- licenses.

Requirements:

- Local notifications only; no push service.
- Remind, snooze, mark renewed and open editor actions.
- Expiration findings appear in the audit dashboard.
- Reminder settings are per-entry and globally configurable.

## §9 Entry health badges

User job: see risk hints without opening every entry.

Examples:

```text
GitHub          2FA 🔑 healthy
Old forum       reused
Bank            expiring soon
```

Rules:

- Badges are metadata and do not expose secret values.
- Severity is text/icon plus color, not color-only.
- Clicking a badge opens the relevant audit finding or entry detail.

## §10 Local-only onboarding

User job: understand Castellan's promise before creating/importing a vault.

Copy contract:

```text
Your vault is a file you own.
No account. No cloud. No telemetry.
Sync is direct between your devices when you choose it.
```

Required first-run choices:

- create new vault;
- open KeePass database;
- import from another manager;
- learn how local sync works.

## §11 Import preview

User job: import without losing data silently.

Requirements:

- Preview counts by type, duplicates, unsupported fields and warnings.
- User can review duplicates before import.
- Unsupported fields are preserved when possible or explicitly reported.
- Import apply creates a backup and an action-log receipt.
