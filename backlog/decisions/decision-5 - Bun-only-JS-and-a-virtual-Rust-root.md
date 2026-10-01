---
id: decision-5
title: "Bun as the only JS runtime, and a virtual Rust root — geoquery's setup, adjusted"
date: '2026-10-01 20:25'
status: accepted
---
## Context

The monorepo setup was copied from Archont561/geoquery and pixi-sandbox. Two of geoquery's
constraints do not apply here: its Cargo root is also a package because `pixi publish`
needs `cargo install --path .`, and its whole environment runs through pixi for conda
packaging plus an offline sandbox transport. Castellan has no conda package, no Python SDK,
no airgap requirement.

## Decision

**Keep:** bun as the only JS runtime (`packageManager` pinned, turbo + biome as workspace
devDependencies, hooks and CI assume `bun run`), the `crates/package.json` turbo façade
(`@castellan/rust` — one Rust node in the JS task graph, `cache: false`), workspace-level
`[workspace.dependencies]`/`[workspace.lints]`, the strict `tsconfig.base.json` (since
moved into the `@castellan/utils` package), lefthook,
single-job CI, dual MIT/Apache-2.0.

**Drop:** pixi entirely. The toolchain is pinned by `rust-toolchain.toml` + `packageManager`
instead. The Rust root is a **virtual** manifest (geoquery's is a package; ours has no
root-level artifact to install).

**Add:** the `codegen` turbo task with the dependency edge from `@castellan/protocol`'s
typecheck, and the whole WASM face (geoquery has none).

## Consequences

- One less lockfile and one less task runner than geoquery; contributors need bun + rust
  and nothing else.
- If a conda/pixi need ever appears (packaging for offline environments would be the
  likeliest), this decision is the one to reopen, and geoquery's pixi.toml is the template.
