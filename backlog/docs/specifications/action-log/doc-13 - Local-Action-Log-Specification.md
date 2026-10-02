---
id: doc-13
title: "Local Action Log Specification"
type: specification
created_date: '2026-10-02 20:38'
updated_date: '2026-10-02 20:38'
tags:
  - audit
  - privacy
  - security
  - local-first
  - specification
---
# Specification — local action log

**Scope.** Every consequential action in Castellan must leave a user-visible,
local-first receipt. The log answers: *what happened, when, on which device,
through which face, and what non-secret object was involved?* It never answers
by leaking the secret itself.

The action log is a product feature, not debug telemetry. It exists so a user
can inspect what Castellan, its browser companion, its CLI, and paired devices
did with their vault.

## §1 Principles

1. **User-visible by default.** A log that only developers can read does not
   satisfy this spec. The app exposes an Activity/Audit view, plus per-entry and
   per-device filtered histories.
2. **Local first, no telemetry.** Log records stay on the user's devices unless
   the user explicitly exports or syncs them. There is no remote logging sink.
3. **No secret values.** Passwords, TOTP seeds/codes, passkey private keys,
   recovery code bodies, keyfile bytes and decrypted attachments are never log
   fields.
4. **Receipts for exposure.** Copy, reveal, fill, CLI stdout/env injection, SSH
   signing and sync export all log that a secret was exposed to a target, with
   target identity and policy outcome, but not the value.
5. **State changes are auditable.** Unlock/lock, entry create/update/delete,
   import/export, backup/restore, settings changes, device pairing, permission
   grants and sync applies all log before/after shape in metadata terms.
6. **Failures matter.** Denied requests, failed unlock attempts, blocked origin
   matches, failed syncs and repair attempts are logged with non-secret reason
   codes.
7. **Logs are inspectable and erasable with intent.** Users can search, filter,
   export a redacted diagnostics bundle and clear local history. Clearing logs
   is itself logged until the clear point, then summarized by a retained tombstone
   where possible.
8. **Logging cannot become a side channel.** Entries may be referenced by id and
   display title, but privacy-sensitive exports default to redacted titles/URLs.

## §2 What counts as an action

Castellan logs every consequential action, grouped by category:

| Category | Examples |
| --- | --- |
| Session | app start/stop, database opened, unlock success/failure, soft lock, hard lock, panic lock |
| Secret exposure | copy username/password/TOTP/recovery code, reveal secret, browser fill, extension TOTP request, CLI `--stdout`, env injection, SSH/signing-agent operation |
| Vault mutation | create/update/delete/restore entry, move to trash, permanent delete, tag/group changes, attachment add/remove, password history restore |
| Import/export | import preview, import apply, export request, emergency kit generation, diagnostics bundle creation |
| Backup/restore | copy-aside created, save started/finished/failed, backup restore preview/apply |
| Browser integration | extension connected/disconnected, origin match allowed/denied, native-host repair, passkey ceremony request/result |
| Sync/device mesh | discovery on/off, pair request, pair approve/decline, sync review, backup before sync, conflict choice, sync apply/fail/cancel |
| Developer tooling | Git credential request, SSH agent request, `castellan run`, `.env` write/shred, direnv grant, pre-commit secret block, SOPS/age decrypt/sign |
| Settings/trust | lock policy changes, clipboard timeout changes, biometric/PIN enrollment changes, device trust/policy changes, network interface/port changes |

Pure navigation such as switching tabs or opening a settings page does not need a
persistent action receipt unless it exposes secret material, changes policy, or
answers a remote/tool request. UI test/debug traces are separate and are not the
action log.

## §3 Record shape

Normative shape, naming illustrative:

```ts
type ActionLogRecord = {
  id: string;
  happenedAt: string;        // RFC 3339 with local offset preserved for display
  deviceId: string;
  deviceAlias: string;
  face: "desktop" | "mobile" | "extension" | "cli" | "sync" | "system";
  actor: "user" | "extension" | "paired_device" | "system" | "cli_process";
  action: string;            // stable snake_case event name
  outcome: "success" | "denied" | "failed" | "cancelled" | "pending";
  target: {
    kind: "entry" | "vault" | "device" | "browser" | "origin" | "project" | "setting" | "backup" | "sync";
    id?: string;
    display?: string;        // user-visible, redacted in privacy exports
    origin?: string;         // host/origin when relevant, redacted in privacy exports
  };
  context?: {
    process?: string;        // e.g. git, ssh, bun, browser profile
    command?: string;        // argv redacted by policy
    policy?: string;         // e.g. ask_each_time, auto_paired_non_conflicting
    reasonCode?: string;     // stable non-secret reason
    counts?: Record<string, number>;
    backupId?: string;
  };
  sensitive: "none" | "metadata" | "secret_exposed";
  schemaVersion: number;
};
```

Rules:

- `action` is stable enough for filters and tests.
- `display` is for humans and may change; never key behavior on it.
- `sensitive: secret_exposed` means a secret was shown, copied, filled,
  injected, signed with, or exported to another local process/device. It does
  not mean the secret is in the record.
- Commands and process names are recorded only after redaction of obvious secret
  arguments and environment values.

## §4 Storage model

The log has two tiers:

1. **Vault action log** — encrypted, detailed records tied to an unlocked vault.
   This is the primary user-visible history.
2. **Local operational ring buffer** — minimal records needed before unlock or
   after transport failure, such as app start, extension disconnected, unlock
   failure count, native-host repair failure. This buffer contains no entry
   titles, URLs, usernames, or secret-exposure records.

Syncing action logs between devices is optional and off by default. When enabled,
only redacted receipts sync unless the user explicitly opts into full metadata.
Each record keeps its originating device id so sync does not pretend to be a
central authority.

## §5 UX surfaces

### Activity view

```text
┌────────────────────────────────────────────────────────────────┐
│ Activity                                            Export…     │
├────────────────────────────────────────────────────────────────┤
│ Filters: [All actions ▾] [Secret exposure] [This device ▾]     │
│ Search: [ GitHub, Pixel, ssh, failed…                     ]    │
├────────────────────────────────────────────────────────────────┤
│ Today                                                          │
│ 14:32  Password copied       GitHub        desktop   success   │
│ 14:30  SSH key used          prod-admin    cli       success   │
│ 14:12  Sync applied          Pixel 9       sync      success   │
│ 13:55  Fill denied           evil.example  extension denied    │
│                                                                │
│ Yesterday                                                      │
│ 21:02  Backup created        before save   desktop   success   │
└────────────────────────────────────────────────────────────────┘
```

### Action detail

```text
Password copied
Today 14:32:08 +02:00

Entry: GitHub
Face: Desktop
Device: Warsaw laptop
Outcome: success
Secret value: not stored in log
Clipboard policy: clear after 30 seconds

[Show related entry] [Copy diagnostic summary]
```

### Per-entry history

```text
Entry activity — GitHub
Today 14:32  password copied on Warsaw laptop
Today 12:01  TOTP code copied from Firefox
Yesterday    URL changed on Pixel 9
```

### Developer receipts

```text
Developer access — my-app
Today 14:22  `bun dev` received 6 env vars for 18m
Today 13:40  git credential helper used GitHub token for github.com
Today 09:12  pre-commit blocked `.env.local`
```

## §6 Filtering and export

Required filters:

- action category;
- outcome;
- face;
- device;
- target kind;
- secret-exposure only;
- denied/failed only;
- time range;
- search over display strings, stable action names and reason codes.

Exports:

- **User export**: readable local JSON/CSV/Markdown, unlocked-only, includes
  metadata the user can see in-app.
- **Diagnostics export**: redacted by default — no entry titles, usernames,
  origins, commands, project paths or browser profiles unless the user enables
  each category.

Exporting the log is itself logged before the export file is written.

## §7 Retention and clearing

Defaults:

- keep vault action records for 90 days or 10,000 records, whichever is larger;
- keep secret-exposure records for at least 30 days unless the user clears them;
- keep local operational ring buffer small, e.g. 1,000 records or 14 days.

Users can change retention locally. Clear actions:

- clear local operational log;
- clear vault action log up to a date;
- clear records for a forgotten device;
- clear exported diagnostics files from Castellan's known export list.

Clearing logs must warn that it reduces auditability, and the clear event should
leave a summary marker when technically possible.

## §8 Security and privacy constraints

- Never log secret values.
- Never log full environment values, clipboard contents, TOTP codes, QR payloads,
  passkey challenge responses, decrypted files, or key material.
- Redact command arguments that look like tokens, URLs with credentials, bearer
  headers, or inline environment assignment values.
- Log origin matching results, not the secret that would have filled.
- Log passkey ceremonies with RP ID/origin/result and credential display id, not
  private keys or signed assertions.
- Log failed unlock attempts without recording password length, keyfile path if
  the user marks it private, or biometric details beyond stable reason codes.
- The action log UI must be hidden/locked with the vault; pre-unlock operational
  state only says what is safe to reveal.

## §9 Implementation expectations

- All face/client actions should go through a small action-recorder boundary so
  logging cannot be forgotten in each UI component.
- Protocol/dispatcher methods should emit receipts for authoritative decisions:
  access allowed/denied, vault mutation, sync apply, secret returned.
- UI components may optimistically show a toast, but the durable receipt belongs
  at the layer that actually performed the action.
- Tests should cover "action produced receipt" for every protocol method that
  exposes a secret or mutates state, plus redaction tests over commands/env.
