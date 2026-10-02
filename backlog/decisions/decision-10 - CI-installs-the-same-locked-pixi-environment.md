---
id: decision-10
title: "CI installs the same locked pixi environment contributors do"
date: '2026-10-02 01:45'
status: accepted
supersedes: decision-8 (the "CI keeps its direct installs" consequence)
refs:
  - infrastructure/ci
---
## Context

Decision-8 accepted a known duplication: pixi provisions the dev tools locally
while CI installs the same binaries its own way (rustup, `setup-bun`,
`install-action`, and actionlint's download script). The drift risk was
written down in `.knowledge/infrastructure/ci.md` and left to review.

Review did not catch it. The first CI run of the repository failed at **Lint
workflows** with exit 127 — `actionlint: command not found`. The download
script it used (`scripts/download-actionlint.bash`) puts the binary in the
*current directory* and exports its path as a step output; it never touches
`PATH`. The step installed actionlint perfectly and the next step could not
see it, so the gate died with the eight checks after it unreported. The
binary pixi already pins — conda-forge `actionlint` 1.7.12, in the lockfile,
on PATH in any `pixi run` — was sitting right there unused.

The failure is the generic shape of the accepted risk, not an accident: two
lists of tools, installed two ways, and only one of them is locked.

`taiki-e/install-action` is not the repair. actionlint is absent from its
manifest, so it falls through to cargo-binstall, which cannot install a Go
program — the same hole other repositories have already fallen into.

## Decision

**CI installs the dev-tool environment from `pixi.lock` and runs the gates
inside it.** `prefix-dev/setup-pixi` (SHA-pinned, pixi v0.81.0 — the version
relock.yml already pins) with `locked: true` and `activate-environment: true`
replaces `setup-rust-toolchain`, `setup-bun`, both `install-action` steps and
the actionlint download. The steps after it are unchanged bare commands —
`bun run lint:workflows`, `cargo fmt --all --check` — now resolved against the
same binaries a contributor gets.

Three things follow from "the lock is the list":

- **`locked: true`, never a silent re-solve.** A stale `pixi.lock` fails the
  gate; fixing drift is relock.yml's lane, which commits the refreshed lock
  and dispatches CI itself.
- **wasm-bindgen is pinned too.** wasm-pack downloads a matching
  `wasm-bindgen` mid-build, which was the last unpinned network install
  inside a gate. conda-forge's `wasm-bindgen-cli` is pinned exactly
  (`==0.2.129`) because wasm-pack reuses a binary on PATH *only* on an exact
  version match, and otherwise silently goes back to downloading.
- **The link step keeps using the system toolchain.** conda-forge's rust
  activation points cargo at conda's gcc, whose sysroot is older than the
  runner's; the Tauri crates link against apt's webkit/gtk, built for the
  runner's glibc. CI pins
  `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=cc` after activation, so
  rustc and std stay the locked conda ones and only the final link is the
  platform's.

What decision-8 drew stays drawn: scripts stay behind `bun run`, pixi manages
**tools only**, rust-toolchain.toml and `packageManager` still pin the rustup
and bun.sh route for contributors who skip pixi, and `pixi.toml` still mirrors
those pins.

## Consequences

- A tool is added in one place. The `.knowledge/infrastructure/ci.md` gap row
  "tool list duplicated between pixi.toml and ci.yml" is closed; the thing
  review has to check now is that the mirrored *version* pins (rust, bun)
  still agree, which the manifests comment in place.
- Gate tools are pinned as strictly as the code's dependencies: a version
  moves in a reviewable `pixi.toml` + `pixi.lock` diff, never by a release
  being published between two runs. `bun run lint:workflows` means the same
  thing on both sides of the wire.
- CI gains pixi's install on the critical path, cached by `setup-pixi` on a
  `pixi.lock` hash — paid back by wasm-pack no longer fetching or compiling
  `wasm-bindgen-cli` on every run.
- The four conda platforms do not include win-64 (decision-8): a Windows
  runner, if one is ever added, needs the bun.sh + rustup route, not this
  environment.
- The airlock gets closer to self-sufficient: `restore.sh` now materialises a
  `wasm-bindgen` too, so `bun run wasm` stops being the one gate that needs
  the network. `wasm-opt` (binaryen) is still downloaded by wasm-pack for
  release profiles and is the remaining offline gap.
