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
updated: "2026-10-04T15:26:34Z"
id: infrastructure/monorepo
category: infrastructure
refs:
  - infrastructure/codegen
  - project/architecture
---
# Monorepo setup

**Inherited** from Archont561/geoquery, Archont561/pixi-sandbox and
Archont561/pathway, with credit due in every derivative. The pattern: bun
workspaces + turbo, the Rust crates as per-package turbo nodes
(`@castellan/rust-*`, pathway's shape) beside the `crates/package.json`
façade for workspace-wide Cargo invocations, biome, lefthook, cargo-deny,
strict shared TypeScript bases
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
  commits the capability and its pinned skills. UI work has Anthropic's
  `frontend-design` and `webapp-testing` skills beside the existing backlog,
  session, refactor, TDD, Playwright and writing-for-agents skills.
- **The offline airlock**: `publish-sandbox.yml` packs the locked environment
  and the 489 vendored crates onto the orphan branch
  `sandbox/developer-<platform>`; `scripts/restore.sh` (generated, regenerable)
  unpacks and verifies it with no network. On 0.4.0 it stops one step short of
  a usable cargo here — see the divergence row below.

## Kept from geoquery

- Bun as the only JS runtime; `packageManager` pinned; hooks and CI assume
  `bun run`.
- The Rust façade package (`@castellan/rust`) for the invocations that must
  stay one cargo call (fmt, deny, coverage, codegen, the Tauri app crates'
  clippy — they need the gtk stack, so they cannot ride the per-crate
  packages); per-crate `nextest`/`clippy` live on `@castellan/rust-*`
  (pathway's per-package structure, adopted 2026-10-04) with precise
  per-crate turbo inputs — cached, because they are pure Rust and
  environment-independent, while the façade's tasks stay `cache: false`.
- Workspace-level `[workspace.dependencies]` and `[workspace.lints]`;
-missing_docs + clippy::all as workspace defaults.
- Strict TS base config; biome without prettier; lefthook over husky.

## Divergences (all deliberate)

| Change | Why |
| --- | --- |
| **Virtual** Cargo root, not a package | no pixi/publish flow needs `cargo install --path .` (decision-5) |
| No pixi for packaging or the task graph | not a Python project; scripts stay behind `bun run`, CI installs bun/rust directly — but pixi returned as a tools-only dev environment (decision-8) |
| `codegen` turbo task + dependency edge + CI zero-diff gate | consumers regenerate before typecheck; `codegen:check` rejects changed or untracked committed output |
| WASM layer (`castellan-wasm` + `@castellan/wasm`) | the extension face needs pure-logic WASM (geoquery has none) |
| Root `.cargo/config.toml` setting `TS_RS_EXPORT_DIR` | redirects ts-rs test-time exports into the shared generated dir — one output, two doors (`cargo test` and xtask produce identical bytes) |
| **No** local wrapper around `scripts/restore.sh` | that tracked `.cargo/config.toml` is exactly what stops pixi-sandbox 0.4.0 from wiring the vendor tree (it will not clobber an existing file), and the `[source]` block cannot be tracked either — replacement pointing at a directory that only exists after a restore is a hard error for CI and every networked contributor. A working wrapper was written, measured and then reverted (`32b2b51`): the fix belongs in `restore`, and a local copy would be one more thing to re-sync with each pixi-sandbox release *and* would mask the upstream fix when it lands. Filed as [pixi-sandbox#55](https://github.com/Archont561/pixi-sandbox/issues/55) with the full measurement set; adoption here is a `PIXI_SANDBOX_VERSION` bump, and AGENTS.md carries the interim manual commands |
| Tauri app crates as workspace members | two apps in-repo, built from the same crates |
| `apps/docs/` as a workspace member | the docs site joins the turbo graph (build in CI) |
| `@castellan/utils` owns the TS bases + test fixtures | geoquery's root `tsconfig.base.json` became a package: `base`/`lib`/`app` extended through package exports (apps compose their framework-generated config with `app.json` via extends-arrays — strictness now reaches the apps, which a root file never did), plus `createFixture` for bun test suites |
| Repo-wide `@` root alias + bunup builds through one preset | every member resolves `@/` to its own root (tsconfig paths in packages, `kit.alias` in SvelteKit, generated in WXT, tsconfig paths in Astro); `libPreset` in `@castellan/utils/bunup` centralizes ESM+dts build choices — package `src/` never uses `@` because consumer bundlers would misresolve it. The alias is **enforced**, not suggested: biome's `style/noRestrictedImports` forbids any `../`-climbing import, with `packages/*/src/**` exempted as the place where `@` is the wrong answer. A directory the alias does not reach is a directory whose bundler has not been taught it — `packages/ui`'s component tests needed `resolve.alias` in `ctViteConfig`, because tsconfig `paths` is a typechecker fact and vite never reads it |
| fast-check + bun-test properties | the TS packages get proptest's mirror; convco/actionlint ride the toolchain (neither ships a usable npm CLI — the npm `convco` entry is an empty squat, npm `actionlint` is a wasm library with no bin) |

## Conventions that travel with the pattern

Commits Conventional-Commits; changesets for the packages; deny.toml
excluding the unpinned/unknown; every crate documented (missing_docs is
deny). AGENTS.md carries the invariants and the command list; README.md the
architecture and quickstart.
