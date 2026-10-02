---
id: doc-19
title: "Device Mesh and Local Sharing Feature Specification"
type: specification
created_date: '2026-10-02 21:05'
updated_date: '2026-10-02 21:05'
tags:
  - sync
  - device-mesh
  - local-first
  - sharing
  - specification
---
# Specification — device mesh and local sharing features

**Scope.** Product behavior for LAN-only sync, QR/phrase pairing, trust levels,
per-device policies, offline beam/share flows and local file transfer. Protocol
details live in the existing device mesh specification (`doc-6`); UI component
contracts live in `doc-12`.

## §1 Nearby device sync

Requirements:

- Discover nearby devices on LAN without accounts or cloud relay.
- Sync only with paired devices.
- Show local device identity and human-verifiable fingerprint phrase.
- Show pending local/remote change counts before applying.
- Create copy-aside backup before any write.
- Conflicts require review.

## §2 Quick receive / auto sync

Policy choices:

- off;
- ask every time;
- auto-accept non-conflicting changes from paired devices.

Rules:

- No auto-accept from everyone for vault data.
- Deletes, destructive changes, passkey changes, first sync and conflicts require
  review regardless of auto setting.
- Policy changes are logged.

## §3 Pairing ceremony with QR and phrase

Flow:

1. Device A shows QR and phrase.
2. Device B scans QR and displays the same phrase.
3. Both devices require explicit approval.
4. Approved devices store pinned fingerprints and aliases.

Phrase example:

```text
castle-river-copper-lantern
```

Decline stores no trust; repeated declines may offer block.

## §4 Device trust levels

Potential capabilities:

- view metadata;
- fill/copy secrets;
- sync vault changes;
- approve passkey use;
- receive beam entries;
- approve new devices/family admin in future.

Rules:

- Trust is per device and user-visible.
- Capability increases require explicit approval and receipt.
- Removing trust stops sync and invalidates auto-accept policy.

## §5 Per-device sync policy

Per-device controls:

- auto-sync on LAN;
- auto-accept non-conflicting changes;
- deletes: ask/allow/deny;
- passkey changes: ask/allow/deny;
- recovery-code changes: ask/allow/deny;
- max auto-apply age;
- require backup before sync.

## §6 Offline beam entry

User job: share one entry or limited field bundle to a paired nearby device.

Requirements:

- Sender chooses fields: username, password, TOTP code, URL, notes, attachment.
- Defaults are conservative: no notes, no TOTP seed, no passkey private key.
- Expiry: time-limited and/or first-use.
- Receiver sees source device, entry title and included field types before
  accepting.
- Beam creates receipts on both devices where possible.

## §7 LocalSend-compatible file transfer for sensitive artifacts

Use cases:

- emergency kit PDF;
- backup copy;
- keyfile transfer;
- diagnostics bundle;
- import/export files.

Rules:

- Sensitive-file warnings before send.
- Paired devices preferred; unpaired transfer requires explicit one-time
  approval and phrase/QR confirmation.
- File transfer does not become vault sync and does not bypass backup policy.

## §8 Sync history and action log

- Sync-specific history shows peer, time, counts, result and backup path.
- General Activity view shows sync next to browser, CLI and vault events.
- Logs never include secret values or raw deltas.

## §9 Team/household local mode — future scope

Concept:

- shared vaults or entry groups over the same local mesh;
- no hosted admin console;
- local invitations and trust changes;
- per-device/person roles.

This is future scope and should receive its own decision record before
implementation because it changes Castellan from single-owner to shared-secret
product behavior.
