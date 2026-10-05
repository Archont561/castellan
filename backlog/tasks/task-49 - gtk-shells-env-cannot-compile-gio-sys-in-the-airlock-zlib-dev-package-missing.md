---
id: TASK-49
title: 'gtk-shells env cannot compile gio-sys in the airlock: zlib dev package missing'
status: Done
assignee:
  - '@agent'
created_date: '2026-10-05 08:07'
updated_date: '2026-10-05 11:52'
labels:
  - infra
dependencies: []
references:
  - pixi.toml
  - >-
    backlog/decisions/decision-13 -
    JS-runtime-and-GTK-webkit-stack-in-separate-pixi-environments.md
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

Solved, and the defect turned out to be a class rather than one package:
`expat` sits directly behind `zlib` in the same shape. conda-forge names
the runtime `.so` package differently from the dev package that carries the
headers and the `.pc`, a `.pc`'s `Requires` chain names the *dev* name, and
pkg-config walks the whole `Requires.private` chain even for a dynamic link
— so `fontconfig.pc`'s `Requires.private: expat` broke `cairo`, `pango` and
`gdk-3.0` the moment `zlib` was supplied, and only `libexpat` was present.
Both dev packages are now named in the feature, and "transitively present"
is not the same as "resolvable by pkg-config".
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 zlib added to the gtk-shells feature and pixi.lock relocked
- [x] #2 publish-sandbox transport republished carrying zlib's .pc and headers
- [x] #3 `pixi run lint`'s @castellan/rust lane passes in a freshly restored airlock
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
zlib landed as filed and immediately surfaced the second name in the same class: fontconfig.pc carries Requires.private: expat, and cairo/pango/gdk-3.0 all resolve fontconfig while the env carried only libexpat (runtime). gdk-sys then failed exactly where gio-sys had. Added expat = ">=2.6,<3" beside zlib.

Verified every module the shells stack names now resolves: gio-2.0, cairo, pango, gdk-3.0, gtk+-3.0, webkit2gtk-4.1, libsoup-3.0, javascriptcoregtk-4.1 all pass pkg-config --exists.

AC#2/#3 proven in a real airlock, not just locally: packed (26158 blobs; envs 1151.0 MiB + tools 91.6 MiB + vendor 605.4 MiB), force-pushed sandbox/developer-linux-64 @ 62fa92d11af0, restored into an empty tree (38062 entries per env, 0 failures), and ran pixi run --frozen --offline -e shells -- cargo clippy --workspace --all-targets green. zlib.pc, expat.pc, zlib.h and expat.h are all present in the restored tree.
<!-- SECTION:NOTES:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Added zlib and expat to the gtk-shells feature and relocked, so pkg-config resolves the whole shells stack and every -sys build script in the env compiles. Republished the sandbox transport and proved the lane green in a freshly restored airlock. Baseline is now 22/22 lint and 17/17 typecheck - nothing left red.
<!-- SECTION:FINAL_SUMMARY:END -->
