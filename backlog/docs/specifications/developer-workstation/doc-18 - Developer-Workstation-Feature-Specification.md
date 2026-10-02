---
id: doc-18
title: "Developer Workstation Feature Specification"
type: specification
created_date: '2026-10-02 21:05'
updated_date: '2026-10-02 21:05'
tags:
  - developer
  - cli
  - secrets
  - local-first
  - specification
---
# Specification — developer workstation features

**Scope.** Castellan as a local-first developer secret workstation: CLI,
project secret profiles, process-scoped environment injection, Git/SSH helpers,
signing, SOPS/age, containers, CI secret pushes and developer audit receipts.

## §1 CLI foundation

Commands:

```console
castellan status
castellan unlock
castellan lock
castellan get <query> --username|--password --clip
castellan totp <query> --clip|--watch
castellan generate password|passphrase|token
```

Rules:

- Secrets do not print to stdout by default.
- `--stdout` requires explicit flag and, outside non-interactive trusted mode,
  a warning.
- `--clip` uses clipboard hygiene policy.
- Every secret exposure emits an action-log receipt.

## §2 Scoped env injection — `castellan run`

User job: run commands with secrets without writing `.env` files.

Example:

```console
castellan run my-app -- bun dev
```

Requirements:

- Injects secrets into child process only.
- Logs secret names/profile/command metadata, never values.
- Requires approval for new project/profile/command combinations.
- Clears grant when command exits unless a time-limited policy says otherwise.

## §3 Project secret profiles

Commit-safe file:

```toml
# .castellan.toml
[project]
name = "my-app"

[env]
DATABASE_URL = "entry:postgres-dev/url"
STRIPE_SECRET_KEY = "entry:stripe-dev/token"
```

Rules:

- Config files contain references only, never secret values.
- Grants pin directory path, git remote/root and profile id.
- Profile changes are logged.

## §4 Safe `.env` generation

Requirements:

- `castellan env write <profile> --to .env.local --ttl 1h`.
- Warn if target is not gitignored.
- Generated file contains header with creation time, expiry and delete command.
- `castellan env shred` deletes/overwrites where platform permits.
- Writing and shredding are logged.

## §5 direnv integration

Requirements:

- `.envrc` can request `use castellan <profile>`.
- First use prompts with directory, git remote, profile and env var names.
- Grants can be once, time-limited or repo-pinned.
- Values are never printed in approval UI.

## §6 Git credential helper

Requirements:

- Host/origin-matched credential retrieval.
- Per-host allow once/always/deny policy.
- Supports GitHub, GitLab, Codeberg and self-hosted Git over HTTPS.
- Logs host, process and entry reference, not token.

## §7 SSH agent and signing operations

Requirements:

- Vault-backed SSH private keys via `SSH_AUTH_SOCK`.
- Per-use or time-limited approval by host/process/key.
- Optional biometric/PIN gate.
- Support commit/tag signing with SSH keys where feasible.
- Logs signing request metadata, never private keys or signatures if sensitive.

## §8 GPG, age, minisign and SOPS

Requirements:

- Store signing/decryption identities in vault-compatible fields.
- Provide helper commands for age/SOPS workflows.
- Require per-repo trust policy before decryption/signing automation.
- Log decrypt/sign action, file path redacted by privacy setting when needed.

## §9 Docker Compose and Kubernetes local secrets

Requirements:

- Process injection preferred over writing secrets into container/orchestrator.
- Explicit review before writing Docker/Kubernetes secrets.
- Show context/cluster/namespace and secret names.
- Log egress into container/orchestrator local state.

## §10 CI/CD secret push

Use case: explicit user-initiated upload to GitHub Actions, GitLab CI or similar.

Rules:

- Strong warning: after upload, secrets are no longer local-only.
- Show repo/project, provider, secret names and scopes.
- Require confirmation per push batch.
- Log third-party egress.

## §11 Developer TOTP helper

Requirements:

- `castellan totp <entry> --clip`, `--watch`, `--stdout` with explicit policy.
- TOTP seed never leaves vault/app process.
- CLI output can show code only by explicit user command.
- Each code copy/stdout emits receipt.

## §12 Developer token manager

Entry templates:

- GitHub PAT;
- GitLab token;
- npm token;
- PyPI token;
- Cargo registry token;
- Docker registry token;
- AWS/Cloudflare/API tokens.

Metadata:

- scopes;
- created/expires;
- last rotated;
- allowed projects;
- service host;
- notes.

Warnings are local unless user explicitly checks remote APIs.

## §13 Developer secret health and local scans

Requirements:

- `castellan scan .` detects committed `.env`, private keys, known token shapes,
  shell history risks and values matching stored Castellan secrets.
- Results are local and redacted.
- Findings link to remediation: move to vault, gitignore, rotate, install hook.

## §14 Pre-commit secret guard

Requirements:

- Optional hook install.
- Blocks obvious secrets before commit.
- If a staged value matches a vault secret, report entry name only if unlocked;
  otherwise report generic secret match.
- Does not print the secret.

## §15 Localhost and dev-origin rules

Requirements:

- Temporary mapping from local dev origin to real/staging entry.
- Expiry required, default short.
- Visible in domain rules and Activity.
- Never global by default.

## §16 WebAuthn/passkey developer lab

Requirements:

- Debug view for RP ID, origin, challenge metadata, resident key/user
  verification flags, result and denial reason.
- Supports localhost/staging development rules with expiry.
- Never exposes private keys or signed assertions.

## §17 Local automation socket — no localhost TCP secrets API

Rules:

- No unauthenticated localhost HTTP API for secrets.
- Automation uses UDS/named pipe with same-user/peer-credential checks.
- Grants are explicit, path/repo/process scoped and short-lived by default.
- Every request is logged.

## §18 Project dashboard

Surface:

```text
my-app
  7 secrets
  2 expired
  direnv allowed
  last used 14m ago

Commands
  bun dev
  bun test
  docker compose up
```

Requirements:

- Shows profiles, grants, recent receipts, stale tokens and policy warnings.
- Does not show secret values.

## §19 Developer receipts

All developer features are backed by the action-log spec (`doc-13`). Receipts
name project/profile/command/host/process and outcome, never secret values.
