---
id: TASK-33
title: "SSH agent (KeeAgent-compatible)"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:50'
updated_date: '2026-10-01 20:50'
labels:
  - devtools
  - security
dependencies: [TASK-8]
references:
  - crates
  - apps/desktop/src-tauri
priority: medium
ordinal: 3300
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The developer tier begins: an SSH agent speaking the agent protocol over the usual socket, serving keys from vault entries, KeeAgent-compatible field naming so existing databases just work, per-use confirmation like ssh-askpass.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 ssh -T git@github.com authenticates with a vault-stored Ed25519 key
- [ ] #2 KeeAgent-style entry fields recognized (OpenSSH + PuTTY userkeys read)
- [ ] #3 Signing requests prompt (optional, default on for new keys) with the requesting process name where the OS provides it
- [ ] #4 Agent socket lifecycle on all three platforms, incl. launchd/systemd user-session notes
<!-- AC:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Agent protocol implementation in a new crate (framing is simple and well-documented); key material decrypted per signature, never cached unlocked.
<!-- SECTION:PLAN:END -->
