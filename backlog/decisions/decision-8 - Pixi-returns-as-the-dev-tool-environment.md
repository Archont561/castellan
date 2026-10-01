---
id: decision-8
title: "Pixi returns as the dev-tool environment — tools only, not packaging"
date: '2026-10-01 21:15'
status: accepted
supersedes: decision-5 (the "Drop: pixi entirely" half)
---
## Context

Decision-5 dropped pixi because the repo had nothing but rust+bun to manage, and
geoquery's pixi.toml existed for things this repo does not do (conda packaging, a Python
SDK, an offline sandbox transport). Since then the toolchain grew: convco and actionlint
have no usable npm distribution (npm `convco` is an empty squat, npm `actionlint` a wasm
library with no bin), cargo-deny/nextest/llvm-cov are release binaries, and the wasm face
needs wasm-pack plus the wasm32 std — five install instructions per contributor per OS,
each one a way for "works on my machine" to drift from CI.

## Decision

**A pixi.toml at the repo root provisions the dev tools — nothing else.** One
`pixi install` yields bun, rust (with cargo/clippy/rustfmt), the wasm32 std, wasm-pack,
convco, actionlint, cargo-deny, cargo-nextest and cargo-llvm-cov, pinned and locked
(`pixi.lock`, committed like bun.lock and Cargo.lock).

The boundary stays where decision-5 drew the rest of it:

- **Scripts stay behind `bun run`** — pixi tasks are thin wrappers; hooks, CI and
  AGENTS.md keep assuming `bun run`. No geoquery-style pixi-run indirection.
- **The rust version stays pinned by rust-toolchain.toml** (rustup users and CI read
  it); pixi.toml *mirrors* the pin and must move with it — conda's cargo is not a
  rustup proxy, so the pin file does not re-pin anything inside `pixi run`.
- **bun's pin stays `packageManager` in package.json**, mirrored the same way.
- **No conda packaging, no Python, no pixi-managed project dependencies** — the env is
  a tool belt, not a runtime.

## Consequences

- Platforms are linux-64, linux-aarch64, osx-64, osx-arm64 — conda-forge does not
  publish bun for win-64. If Windows dev arrives: add the platform plus
  `[target.win-64.dependencies]` tables for the packages that skip it; Windows bun
  comes from bun.sh.
- CI keeps its direct installs (rustup + setup-bun + install-action + the actionlint
  download script) — same binaries, different provisioning. The drift risk between the
  two lists is accepted for now and owned in `.knowledge/infrastructure/ci.md`.
- The two version mirrors (rust, bun) are a duplication decision-5 existed to avoid;
  it is bought back deliberately in exchange for one-command onboarding, and the
  manifest comments name the files that must move together.
