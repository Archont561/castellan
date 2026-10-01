---
type: Reference
title: "Castellan — project overview"
description: "Mission, the scope fence, the four abstractions, and the competitive position."
tags:
  - project
  - product
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-01T22:00:00Z"
id: project/overview
category: project
refs:
  - project/architecture
  - research/naming
---
# Project overview

**Castellan** — the keeper of the castle keys — is a local-first, offline
password manager: KDBX vault the user owns, desktop + mobile apps, a browser
companion (**Fob**), passkeys as the flagship ("the soft YubiKey"), and a
LAN device mesh instead of cloud sync. Roadmap: v0.1 daily driver → v0.2
hygiene + mobile alpha → v0.3 device mesh → v0.4 passkeys → 1.0 dev tools,
recovery, hardening.

## The scope fence (is / is not)

**Is**: offline-first; KDBX-native; software-only 2FA and passkeys
(no hardware key required); more capable than KeePassDX on every platform;
developer-tier tooling (SSH agent, CLI, env injection).

**Is not**: a cloud product, a team product (households maybe, later), a
hardware authenticator, an enterprise SSO anything. No accounts exist to
create.

## The four abstractions

1. **Vault** — a KDBX file on the user's disk. The unit of ownership and
   the thing every feature ultimately reads or mutates.
2. **Entry** — one secret-bearing record; a projection (metadata + flags,
   never secret values) is what faces ever see in lists.
3. **Protocol** — the one message language (see [architecture](architecture.md));
   everything speaks it: extension, desktop, mobile, future CLI.
4. **Face** — a delivery surface (desktop app, mobile app, extension, CLI)
   rendering the same core through a different shell.

## Position

Against KeePassXC: cross-device + mobile + passkeys in one product. Against
KeePassDX: everything KeePassDX does, plus desktop parity, passkeys, audit,
and the mesh. Against Bitwarden/1Password: no accounts, no cloud, no
subscription — the file is the product.

The name and the companion's name were collision-checked (see
[naming](../research/naming.md)); re-verify before any public launch.
