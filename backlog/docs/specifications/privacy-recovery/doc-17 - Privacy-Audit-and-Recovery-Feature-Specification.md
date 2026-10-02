---
id: doc-17
title: "Privacy, Audit and Recovery Feature Specification"
type: specification
created_date: '2026-10-02 21:05'
updated_date: '2026-10-02 21:05'
tags:
  - privacy
  - audit
  - recovery
  - security
  - specification
---
# Specification — privacy, audit and recovery features

**Scope.** Local features that help users understand risk, inspect secret access,
recover from mistakes, and prove Castellan stays local-first.

## §1 Local audit dashboard

Findings computed locally:

- weak passwords;
- reused passwords;
- old passwords;
- expiring entries;
- missing 2FA where known;
- recovery-code age/used coverage;
- developer-token health where metadata exists.

Rules:

- Dashboard never shows password values.
- Breach checks, if present, are opt-in and k-anonymous; network egress is
  visible in the privacy report.
- Remediation actions open scoped entry lists or editor flows.

## §2 Privacy report — what left this device?

User job: inspect every egress and local secret exposure.

Surface:

```text
Network and secret activity
Today
  14:32 LAN sync with Pixel 9
  14:12 HIBP prefix check · 5-char prefix
  13:40 GitHub token used by git credential helper

Blocked by design
  telemetry
  cloud backup
  cloud relay
```

Requirements:

- Powered by the local action log (`doc-13`).
- Shows local process exposure and network egress separately.
- Explicitly distinguishes LAN transfer from third-party network calls.

## §3 Permission dashboard

User job: understand integrations and fix broken permissions.

Rows:

- browser extensions/profiles;
- clipboard;
- notifications;
- local network permission;
- biometrics;
- autostart;
- mobile autofill/provider capability;
- sync discovery/server state.

Requirements:

- Each row has state, consequence and repair/open-settings action when possible.
- Repair actions log outcomes.

## §4 Clipboard hygiene

Requirements:

- Configurable clear timeout.
- Clear on lock, sleep, app quit and panic lock.
- Clear before copying another secret when feasible.
- Visible countdown toast after copy.
- Use platform APIs to exclude clipboard history where supported.
- All copy actions emit receipts; receipts do not contain copied values.

## §5 Screen privacy mode

Requirements:

- Hide secrets in app switcher where platform supports it.
- Blur/conceal secret fields on window blur when enabled.
- Block screenshots on Android where supported.
- Warn when platform only supports best-effort behavior.

## §6 Panic lock

Behavior:

- hard-lock or strongest available lock;
- clear clipboard;
- hide/blur windows;
- stop sync discovery/server;
- disconnect extension sessions where safe;
- emit panic-lock receipt after safe state is reached.

Trigger surfaces:

- command palette;
- tray/menu bar;
- keyboard shortcut;
- mobile settings/action.

## §7 Diagnostics bundle

Requirements:

- User-reviewed before export.
- Redacted by default: no vault bytes, entry titles, usernames, URLs, secrets,
  full commands, browser profiles or project paths unless enabled.
- Includes versions, protocol status, extension/native-host state, sync network
  state, non-secret recent errors and permission state.
- Creating/exporting diagnostics is logged.

## §8 Backup timeline

User job: see copy-aside saves as a safety feature.

Surface:

```text
Backups
Today 14:33 · before sync with Pixel 9 · 7 changes
Yesterday 21:02 · before editing GitHub
```

Requirements:

- Shows reason, time, backup path, related action and retention status.
- Links to restore diff review.
- Backup creation/failure records are in Activity.

## §9 Restore diff review

Requirements:

- Show counts: entries added/restored/reverted/removed/conflicted.
- Show entry titles/metadata only; concealed fields stay concealed.
- Restore writes through the same copy-aside save policy.
- Restore action is logged with backup id and counts.

## §10 Emergency kit generator

Contents:

- vault path;
- KDF parameters;
- keyfile QR or keyfile recovery instructions;
- device/sync recovery notes;
- manual field where user writes master password by hand.

Rules:

- Castellan never stores or prints the master password automatically.
- Generating/exporting/printing the kit is logged.
- Kit should be printable and exportable locally.

## §11 Unlock and recovery drill

User job: periodically prove recovery still works.

Steps:

- lock vault;
- confirm master password works;
- confirm keyfile path/backup exists;
- confirm emergency kit location;
- optionally simulate backup restore preview.

Requirements:

- No secret leaves the device.
- Last successful drill date appears in recovery settings.

## §12 Local-only notifications

Notifications may report:

- vault locked;
- clipboard cleared;
- sync completed/failed;
- backup created;
- expiration reminders;
- recovery drill due.

Rules:

- No push service dependency.
- Notification content omits secret values and can omit entry titles in privacy
  mode.
