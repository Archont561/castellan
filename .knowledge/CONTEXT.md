---
type: Context Briefing
title: "Castellan — knowledge bundle context"
description: "The five facts an agent needs before touching this repository."
tags:
  - meta
  - orientation
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-02T14:53:38Z"
id: context
category: meta
refs:
  - project/overview
  - infrastructure/monorepo
  - infrastructure/testing
---
# Context briefing

1. **Castellan is a local-first, offline password manager** with a browser
   companion (**Fob**), desktop and mobile apps, and a vault that is a KDBX
   file the user owns. No accounts, no cloud, no telemetry. The roadmap runs
   v0.1 (daily driver) → v0.4 (soft security key) → 1.0 (dev tools,
   recovery, hardening).

2. **One RPC contract, one dispatcher, derived TypeScript.** Every operation
   is a `rpc_contract!` entry in `crates/protocol` with request/result fields
   and face membership. Xtask commits TS wire/WASM projection types, scoped
   app clients, and browser-family native-host manifests; every native
   transport terminates in `crates/dispatch`, with desktop/mobile sharing
   `@castellan/tauri`. Hand-written wire types, app client methods and
   app-level operation matches are banned (decision-1 in `backlog/decisions/`).

3. **The monorepo pattern is inherited** from Archont561/geoquery and
   Archont561/pixi-sandbox — bun + turbo, a Rust-workspace façade package,
   biome, lefthook, single-job CI — adjusted: virtual Cargo root, a codegen
   task edge, a WASM layer geoquery does not have, and pixi returning later
   as a tools-only dev environment (decision-8).

4. **Security invariants are not preferences**: no secrets in list answers;
   origin/RP-ID enforcement in the app process, never only in the extension;
   copy-aside before every KDBX save; no localhost TCP for browser
   integration; the sync mesh never gains a cloud relay.

5. **Where things live**: `backlog/` (backlog.md format — tasks, decisions,
   milestones, docs) is delivery state; `.knowledge/` (this bundle, OKF
   v0.2) is durable knowledge — architecture, contracts, rationale,
   research. Task checklists belong in backlog, never here.

## Session scratchpad

### 2026-10-04 — airlock skill, nextest lanes, pathway turbo structure

PR #11 merged at `ac6d8e6e`: the session skill now teaches the offline
airlock (`scripts/restore.sh`) with the stale-transport trap and the
direct-binary fallback; the Rust test lanes run on cargo-nextest (+ a
doc-test pass); every crate is its own turbo package (`@castellan/rust-*`,
pathway's shape) with cached per-crate inputs, while the `@castellan/rust`
façade keeps the one-invocation lanes `cache: false` (they compile the
Tauri pair against the host gtk stack). Post-merge: `ci` green on main,
baseline 23/23 turbo tasks; `publish sandbox` failed as it has on every
push since PR #9 — see below.

Recorded here because no code carries them yet:

- **Upstream pixi-sandbox issue, drafted not filed**: pack's `check_rel_path`
  (`crates/pixi-sandbox-core/src/manifest.rs:298` on 0.5.2 and on main at
  `8e070d4`) rejects any path containing `:`; perl man pages from the shells
  env's gtk stack (`man/man3/App::Cpan.3`) kill the per-file oracle with
  "files manifest entry must be relative". Requested fix: reject only a
  leading Windows drive prefix; regression test asserting
  `man/man3/App::Cpan.3` passes and `C:/x`, `C:\x`, `C:x` fail.
- **Local `ci:` fix pending**: `publish-sandbox.yml`'s captured pipeline runs
  in a `{ set -e; … }` brace group; the first failure exits the whole step
  shell, so the tail/commit-comment failure channel never runs — that is why
  the publish failures above leave no comment and no log tail. One-line
  shape fix: run the pipeline in a `( … )` subshell.
- **pixi-sandbox 0.5.3+** (post-`8e070d4`, unreleased) adds diagnostics and
  log-file controls to pack/doctor/publish (issues #93/#94). When a release
  ships: bump `PIXI_SANDBOX_VERSION`, re-run the sandbox upgrade, and delete
  the hand-maintained failure channel rather than extending it.
- **Transport stays stale** (PR #8-era snapshot) until the upstream fix ships
  and the publisher repacks. Until then `pixi run` guts the restored env on
  first call; the session skill's direct-binary fallback is the working
  environment.

Next session should start with: `./scripts/restore.sh`, then
`gh run list --workflow=publish-sandbox.yml --limit 3` — if green, the
airlock is current and `pixi run` works again; if red, fall back to
`.pixi/envs/default` binaries per the session skill. Decide whether to file
the upstream issue and land the brace-group fix; task-13 (TOTP in the UI)
and task-42 (OTP crate extensions) are the highest-value fully-provable
backlog tasks.
