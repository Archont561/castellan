---
type: Reference
title: "Castellan — monorepo setup"
description: "The geoquery/pixi-sandbox pattern Castellan inherits, and every deliberate divergence."
tags:
  - monorepo
  - tooling
  - lineage
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
created: "2026-10-01T22:00:00Z"
updated: "2026-10-02T10:30:00Z"
id: infrastructure/monorepo
category: infrastructure
refs:
  - infrastructure/codegen
  - project/architecture
---
# Monorepo setup

**Inherited** from Archont561/geoquery and Archont561/pixi-sandbox, with
credit due in every derivative. The pattern: bun workspaces + turbo, the
Rust workspace as one node in the JS task graph via a `crates/package.json`
façade, biome, lefthook, cargo-deny, strict shared TypeScript bases
(`@castellan/utils/tsconfig/*`, extended through the package exports),
single-job CI, dual MIT/Apache-2.0.

## Also taken from pixi-sandbox

- **The pixi dev-tool environment** (decision-8, adopted late): `pixi.toml`
  + committed `pixi.lock` provision bun, rust (mirroring the
  rust-toolchain.toml pin), the wasm32 std, wasm-pack, convco, actionlint,
  cargo-deny, cargo-nextest and cargo-llvm-cov — tools only, split across
  three features (`rust`, `web`, `utils`) so each toolchain area is a
  reviewable diff and a lean env is one `-e` away. The task table is two
  generic routers and nothing else: `bun` (everything after it passes
  through to the bun front door — `pixi run bun run gates`) and `xtask`
  (fronts every clap subcommand in `crates/xtask`). Contributors with plain
  rustup + bun.sh installs (or CI) get the same binaries without pixi.
  Platforms are linux-64, linux-aarch64, osx-64, osx-arm64 — conda-forge
  does not publish bun for win-64.
- The **docs site**: `apps/docs/` is pixi-sandbox's Astro + Starlight app,
  adapted — without its version-substitution rig (no versioned pages yet;
  copy the machinery from the template when the first one exists) and with
  bun forced as the runtime (`bun --bun`) so Astro never lands on a PATH
  node. A bun workspace, built in CI, deployable via pixi-sandbox's
  docs.yml when the site goes public.
- **skills** (devDependency): the agent-skills CLI. `bun x skills add
  <source>` vendors into `.agents/skills/` + `skills-lock.json`; the repo
  commits the capability, no skills yet.
- **The offline airlock**: `publish-sandbox.yml` packs the locked environment
  and the 489 vendored crates onto the orphan branch
  `sandbox/developer-<platform>`; `scripts/restore.sh` (generated, regenerable)
  unpacks and verifies it with no network. Entry point is the hand-written
  `scripts/airlock.sh` around it — see the divergence row below, and
  `scripts/airlock.sh`'s own header for the measurements behind it.

## Kept from geoquery

- Bun as the only JS runtime; `packageManager` pinned; hooks and CI assume
  `bun run`.
- The Rust façade package (`@castellan/rust`): `cache: false`, one node,
  the JS graph never learns cargo.
- Workspace-level `[workspace.dependencies]` and `[workspace.lints]`;
-missing_docs + clippy::all as workspace defaults.
- Strict TS base config; biome without prettier; lefthook over husky.

## Divergences (all deliberate)

| Change | Why |
| --- | --- |
| **Virtual** Cargo root, not a package | no pixi/publish flow needs `cargo install --path .` (decision-5) |
| No pixi for packaging or the task graph | not a Python project; scripts stay behind `bun run`, CI installs bun/rust directly — but pixi returned as a tools-only dev environment (decision-8) |
| `codegen` turbo task + dependency edge | typecheck of `@castellan/protocol` *depends on* codegen — stale generated output fails the build |
| WASM layer (`castellan-wasm` + `@castellan/wasm`) | the extension face needs pure-logic WASM (geoquery has none) |
| Root `.cargo/config.toml` setting `TS_RS_EXPORT_DIR` | redirects ts-rs test-time exports into the shared generated dir — one output, two doors (`cargo test` and xtask produce identical bytes) |
| `scripts/airlock.sh` wraps the generated `scripts/restore.sh` | that tracked `.cargo/config.toml` is exactly what stops pixi-sandbox from wiring the vendor tree (it will not clobber an existing file), and the `[source]` block cannot be tracked either — replacement pointing at a directory that only exists after a restore is a hard error for CI and every networked contributor. The wrapper puts it in a project-local `CARGO_HOME` under `.pixi/` instead (the only untracked, non-global layer cargo reads from a *file* — CLI `--config key=value` and `CARGO_SOURCE_*` do not work), fetches the sandbox branch when the clone's refspec missed it, and verifies resolution before claiming success |
| Tauri app crates as workspace members | two apps in-repo, built from the same crates |
| `apps/docs/` as a workspace member | the docs site joins the turbo graph (build in CI) |
| `@castellan/utils` owns the TS bases + test fixtures | geoquery's root `tsconfig.base.json` became a package: `base`/`lib`/`app` extended through package exports (apps compose their framework-generated config with `app.json` via extends-arrays — strictness now reaches the apps, which a root file never did), plus `createFixture` for bun test suites |
| Repo-wide `@` root alias + bunup builds through one preset | every member resolves `@/` to its own root (tsconfig paths in packages, `kit.alias` in SvelteKit, generated in WXT, tsconfig paths in Astro); `libPreset` in `@castellan/utils/bunup` centralizes ESM+dts build choices — package `src/` never uses `@` because consumer bundlers would misresolve it |
| fast-check + bun-test properties | the TS packages get proptest's mirror; convco/actionlint ride the toolchain (neither ships a usable npm CLI — the npm `convco` entry is an empty squat, npm `actionlint` is a wasm library with no bin) |

## Conventions that travel with the pattern

Commits Conventional-Commits; changesets for the packages; deny.toml
excluding the unpinned/unknown; every crate documented (missing_docs is
deny). AGENTS.md carries the invariants and the command list; README.md the
architecture and quickstart.
