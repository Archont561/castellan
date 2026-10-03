---
type: Playbook
title: "Castellan — CI"
description: "The single gate, its caches, and the checks CI cannot perform (with their owners)."
tags:
  - ci
  - quality
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T22:00:00Z"
updated: "2026-10-02T08:30:00Z"
id: infrastructure/ci
category: infrastructure
refs:
  - infrastructure/monorepo
  - infrastructure/codegen
---
# CI

One job, ordered gates, fast-fail: checkout → apt webkit/gtk (Tauri) →
`setup-pixi --locked` without global activation → cargo caches (workspace
`Cargo.lock` keyed) → `pixi run install-frozen` → codegen → WASM → rustfmt →
actionlint → repository lint → workspace lint → typecheck → docs build →
tests → e2e → Storybook → advisories. Every repository step is an explicit
`pixi run <task>`, including lefthook's commands; system setup remains outside
the project command API. The two lint steps are not redundant: `pixi run lint`
is `biome check .` across the whole repository, while `pixi run
lint-workspace` delegates to the per-package `lint` tasks — each package over
its own `src`, plus `@castellan/rust`, which is where clippy `-D warnings` and
`cargo deny check bans licenses sources` actually live.
Running only the first is what kept the Rust lints unexecuted for the
repository's whole life. `codegen:check` regenerates and then checks both
tracked diffs and untracked files, so stale or omitted generated output fails
before typecheck instead of being hidden by it.

**Every gate tool comes from `pixi.lock`** (decision-10): bun, rust with
clippy+rustfmt, the wasm32 std, wasm-pack, wasm-bindgen-cli, cargo-deny and
actionlint. The task names are the same contributor-facing API CI uses, and
`locked: true` turns a stale lockfile into a failed gate rather than a silent
re-solve (relock.yml owns fixing the drift). One environment override remains:
conda-forge's rust points cargo at conda's gcc, so CI resets
`CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=cc` — the Tauri crates link
against apt's webkit/gtk, built for the runner's glibc, not conda's older
sysroot.

## What each gate buys

- **clippy -D warnings + missing_docs**: the workspace lints; a PR cannot
  land undocumented public API.
- **cargo test**: example tests, rstest `#[fixture]`/`#[case]` matrices,
  and proptest properties (rstest + proptest are the dev-only test
  frameworks, declared in `[workspace.dependencies]`). Properties cover the
  invariants — framing round-trips, otpauth round-trips, passphrase shape —
  and are the specified shape for the harder promises ahead: the KDBX
  round-trip harness (task-8), sync idempotence (task-27). The TS mirror:
  bun tests with fast-check properties (protocol JSON round-trips, client
  id/error/wire-shape properties) and `createFixture` fixtures — same
  vocabulary, other side of the wire (see [Testing](testing.md)).
- **actionlint**: the workflows are code too; `pixi run lint-workflows`
  runs on every push, on the conda-forge binary `pixi.lock` pins (convco,
  the other toolchain-binary linter, is in the same environment but stays
  unused by CI: the commit-msg hook runs it when present, and
  `convco check` over history needs a full clone CI does not fetch). The
  gate's own first failure was this step installing actionlint into a
  directory that was not on `PATH` — hence decision-10.
- **codegen zero-diff**: proves the committed TS matches the Rust — drift
  dies at review time.
- **docs build**: `pixi run docs-build` — the Starlight site is a workspace
  member with no typecheck/test tasks of its own, so a build step is what
  proves it.
- **wasm step**: the extension face's dependency exists and compiles.
- **deny**: licenses and advisories.

## What CI cannot check (and who owns it instead)

| Gap | Owner |
| --- | --- |
| Desktop/mobile production vite builds not in CI | e2e covers the dev-server build of both faces and the extension's *built* MV3 (its e2e script chains `wxt build`), but `vite build` output of the apps is still ungated; add when a release artifact matters |
| Mirrored version pins (rust, bun) | Closed for the tool *list* — CI installs `pixi.lock` itself (decision-10) — but rust-toolchain.toml and `packageManager` still mirror two pins for the rustup + bun.sh route; the manifests name each other in comments, review checks they moved together |
| `wasm-opt` still downloaded by wasm-pack | `wasm-bindgen-cli` is pinned in the environment, binaryen is not; the release profile fetches it, so the wasm gate is the one step that still needs the network (and the one that fails in a restored airlock) |
| Tauri app crates (no webkit in CI) | release builds + the task-37 reproducible-build work |
| Mobile on-device behavior (biometrics, providers) | hardware checklists, tasks 23/30/31/32 |
| Extension in real browsers | Playwright suite runs, but store-review quirks are manual |
| HIBP / LocalSend against the live world | integration smoke tests, manual before release |
| cargo-deny databases age between runs | advisories are re-checked on every CI run by design |

Known-limit honesty is a feature: the README's validated-state section
mirrors this table so nobody trusts a gate that does not exist.
