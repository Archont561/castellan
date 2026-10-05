---
id: TASK-49
title: 'gtk-shells env cannot compile gio-sys in the airlock: zlib dev package missing'
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-05 08:07'
labels:
  - infra
dependencies: []
references:
  - pixi.toml
  - backlog/decisions/decision-13 - JS-runtime-and-GTK-webkit-stack-in-separate-pixi-environments.md
priority: high
ordinal: 15900
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The shells environment's `gio-2.0.pc` declares `Requires: zlib`, but the
gtk-shells feature pulls only `libzlib` (the runtime `.so`) — no `zlib.pc`,
no headers — so `cargo clippy -p castellan-desktop -p castellan-mobile`
dies at gio-sys's build script in every restored airlock, which means the
`@castellan/rust` lint/typecheck/build lanes that decision-13 routed into
the shells env have never actually run there.

Found while running the repo gates for task-13. The companion defect —
conda-forge's pkg-config bakes its install-time `pc_path` into the binary,
and pixi-sandbox's restore materializes the env in a staging directory
before moving it into place, so the baked path dangles on every restored
airlock — is already fixed in-repo: the gtk-shells feature now exports
`PKG_CONFIG_PATH` from its activation table, which is what surfaced this
zlib gap as the next failure.

Fix: add `zlib` (`>=1.3,<2`) to
`[feature.gtk-shells.target.linux-64.dependencies]`, let relock solve it,
republish the sandbox transport, then prove the shells lint lane green in
a freshly restored airlock. Needs the conda network for the solve, so it
is connected-side work by construction — an airlocked session must not
edit the dependency table, because a plain `pixi run` would then try to
re-solve against conda.anaconda.org and gut the restored env.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 zlib added to the gtk-shells feature and pixi.lock relocked
- [ ] #2 publish-sandbox transport republished carrying zlib's .pc and headers
- [ ] #3 `pixi run lint`'s @castellan/rust lane passes in a freshly restored airlock
<!-- AC:END -->
