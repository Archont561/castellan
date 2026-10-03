---
id: decision-11
title: "Pixi is the sole command entry point"
date: '2026-10-03 21:30'
status: accepted
supersedes: decision-8 (the "scripts stay behind bun run" command boundary), decision-10 (activated CI commands)
refs:
  - infrastructure/ci
---
## Context

Decision-10 made CI install the locked pixi environment, but activation still
left every command PATH-sensitive in form. Lefthook was worse: it ran bare
`bun`, `cargo`, and whichever `convco` happened to be present, with a grep
fallback when none was. A hook launched outside `pixi shell` could therefore
use different binaries—and even a different commit-message implementation—from
CI while both configurations claimed to share one toolchain.

The split command surface also made onboarding ambiguous: documentation called
pixi optional, then taught direct bun and cargo commands. The lockfile was the
tool authority without pixi being the command authority.

## Decision

**Pixi is the sole entry point for repository commands.** High-frequency
operations are named tasks in `pixi.toml` (`install`, `gates`, `codegen-check`,
`lint`, `typecheck`, `test`, `e2e`, and the development servers). Generic
`bun`, `bunx`, and `cargo` passthrough tasks cover package-scoped and one-off
work without duplicating every underlying option.

Lefthook bodies invoke `pixi run`, including commit-message validation through
the locked convco; the PATH-dependent fallback is removed. The guarded bun
`prepare` lifecycle re-enters `pixi run hooks-install`. CI does not globally
activate the environment and expresses each repository operation as a named
`pixi run` task. Runner and platform setup (`apt`, caches, shell processing)
is not a repository operation and remains native workflow code.

Pixi still manages tools only. Bun continues to own JS dependencies and
scripts, Cargo continues to own Rust dependencies and builds, and Turbo
continues to own the cross-workspace graph; pixi tasks delegate to those
owners rather than replacing them.

## Consequences

- The documented first prerequisite is pixi. Direct bun/rustup installations
  remain useful for ecosystem tooling but are not a supported command path for
  this repository.
- A bare repository tool in lefthook or CI is now reviewable drift, rather than
  an accepted consequence of environment activation.
- Task names are a small stable command API. Add a named task for a recurring
  root operation; use the generic passthrough for uncommon flags and
  package-local work.
- Installing JS dependencies may launch a nested `pixi run hooks-install`
  during bun's prepare lifecycle. This is intentional: hook installation must
  obey the same boundary as the command that triggered it.
