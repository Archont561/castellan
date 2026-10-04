---
id: decision-13
title: "JavaScript runtime and the GTK/webkit stack live in separate pixi environments"
date: '2026-10-04 13:05'
status: proposed
---
## Context

`cargo check/clippy/test --workspace --all-targets` compiles the Tauri
shells, and on Linux wry links against gtk3 + webkit2gtk-4.1. CI apt-installs
those; offline airlocks have no system packages and cannot run full-workspace
Rust gates at all. Pulling the stack from conda-forge into the pixi
environment that already carries the toolchain was the obvious fix — and the
solver proved it impossible in one environment, twice, on two independent
axes:

- every conda-forge `bun` build (1.3.7–1.3.11, its newest at the time) links
  `icu 75.x`, while the only `webkit2gtk4.1` builds compatible with
  nodejs/bun's `libbrotlicommon 1.2.x` demand `icu 78.3`;
- the icu-free webkit2gtk4.1 builds pin `woff2 ==1.0.2 h54a6638_1`, which
  demands `libbrotlicommon 1.1.x` — impossible beside nodejs, which requires
  1.2.x in every build our range allows.

No combination of pins closes a single environment; the relock bot's own
run log (PR #9) is the forensic record. Webkit2gtk4.1's alma9 rebuilds also
raised a second, smaller requirement: a declared solve-time glibc floor,
which is why the manifest now carries `[system-requirements] libc = 2.34`.

## Decision

Two environments in `pixi.toml`:

- **default** = rust + web + utils — the every-day command API, unchanged
  from before this episode, bun back on its exact pin.
- **shells** = rust + utils + gtk-shells — the Rust toolchain plus the
  conda-forge gtk3 / webkit2gtk4.1 / dbus / librsvg stack, and **no
  JavaScript runtime**, so the icu/brotli conflict never materializes.

Rust lanes reach the right environment without callers deciding: the
`@castellan/rust` turbo facade (crates/package.json) invokes cargo through
**scripts/cargo-env.sh**, which execs `pixi run -e shells -- cargo …` when
the shells env is materialized locally and plain `cargo` (system gtk, as
before) otherwise, overridable with `CASTELLAN_CARGO_ENV`. Cargo-shaped pixi
tasks ride `[feature.rust.tasks]`, so both environments expose them.
The sandbox transport publishes both environments to airlocks
(pixi-sandbox.toml); CI restricts its pixi install to `default` because its
webkit/gtk still comes from apt.

## Consequences

- Full-workspace Rust gates run in offline airlocks for the first time, via
  the same `bun run gates` command — the facade routes the Rust node into
  `shells` transparently.
- CI behavior is byte-identical to before (apt webkit + system-cc
  re-export), deliberately; migrating CI to the conda stack is a separate
  decision.
- Developer machines that run a full `pixi install` stop needing system gtk
  for Rust lanes; nothing changes for those who stay on system packages.
- If conda-forge's bun is ever rebuilt on the current global icu pin, the
  conflict dissolves and the environments *could* re-merge — the split is
  traceable to the upstream pin state and reversible when it moves.
- One caveat kept in view: `webkit2gtk4.1` is a young conda-forge package
  (migrated from cf-post-staging), pinned to the staged 2.48 line; if it
  bit-rots, the apt path in CI remains the fallback.
