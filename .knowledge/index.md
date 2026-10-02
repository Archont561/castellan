---
okf_version: "0.2"
---
# Castellan knowledge base

Durable knowledge for the Castellan password manager: architecture,
contracts, rationale, and research. Delivery state lives in `backlog/`
(tasks, milestones, acceptance criteria); this bundle is the *why* and the
*how it must stay*. Human- and agent-readable, git-shippable, OKF v0.2.

# Project

* [Project overview](project/overview.md) - what Castellan is, what it refuses to be, the four abstractions
* [Architecture](project/architecture.md) - the monorepo layout, one-protocol principle, and the codegen flow
* [Data model](project/data-model.md) - protocol types, the KDBX mapping, and entry projections
* [Standards and formats](project/standards.md) - every external format and RFC Castellan conforms to

# Faces

* [Desktop face](faces/desktop.md) - Tauri + SvelteKit static, the single rpc command, platform shells
* [Mobile face](faces/mobile.md) - Tauri mobile, credential-provider work, biometric gating
* [Extension face — Fob](faces/extension.md) - WXT MV3/Firefox, interception duties, what it must never hold

# Security

* [Threat model](security/threat-model.md) - assets, adversaries, trust boundaries, residual risks stated honestly
* [Biometric unlock](security/biometric-unlock.md) - the K_hw-wraps-K_master architecture and invalidation policy
* [Passkeys](security/passkeys.md) - the soft authenticator, enforcement-in-app, two-credential guidance
* [Recovery](security/recovery.md) - recovery codes, emergency kit, Shamir splitting, no-account-recovery stance

# Features

* [Vault](features/vault.md) - KDBX policy, copy-aside saves, unlock methods
* [Authenticator and OTP](features/otp.md) - otpauth grammar, migration imports, drift handling
* [Device mesh](features/device-mesh.md) - LocalSend interop, pairing, LAN delta sync
* [Audit](features/audit.md) - weak/reused/old findings, HIBP k-anonymity, offline 2FA directory

# Infrastructure

* [Monorepo setup](infrastructure/monorepo.md) - the geoquery/pixi-sandbox lineage and Castellan's divergences
* [Code generation](infrastructure/codegen.md) - ts-rs + xtask pipeline, committed generated output
* [Native channel](infrastructure/native-channel.md) - framing, the byte-pump host, manifests, association
* [CI](infrastructure/ci.md) - the single gate, caches, what CI cannot check
* [Styling](infrastructure/styling.md) - the shared UnoCSS config, tokens, shortcuts, extraction

# Research

* [Passkey providers](research/passkey-providers.md) - how third parties become credential providers per platform
* [LocalSend protocol](research/localsend-protocol.md) - v2.2 wire details for interop
* [keepass-rs internals](research/keepass-rs.md) - 0.15 API facts, write-safety findings
* [Naming](research/naming.md) - the Castellan/Fob collision table and re-verification checklist
