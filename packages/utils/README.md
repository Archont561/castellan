# `@castellan/utils`

The small reuse package for repository-wide TypeScript infrastructure. It keeps
shared tooling policy in one place so every package and face uses the same
strictness, test-fixture lifecycle, build preset, UnoCSS foundation, and
Playwright defaults.

## Exports and subpath APIs

| Path | Purpose |
| --- | --- |
| `@castellan/utils` | `createFixture` and its fixture lifecycle types |
| `@castellan/utils/tsconfig/*` | Strict TypeScript base configurations |
| `@castellan/utils/bunup` | Shared library build preset |
| `@castellan/utils/playwright` | Shared Playwright E2E preset |
| `@castellan/utils/uno` | Shared tokens, extraction pipeline, and optional page shell |

A fixture factory is shareable, but a `createFixture()` call must be made in the
test file that owns its hooks. Do not instantiate fixtures at module scope in a
shared support module: test-runner hooks attach to the file that evaluates it.

## Commands

```console
$ pixi run bun run --cwd packages/utils test
$ pixi run bun run --cwd packages/utils typecheck
$ pixi run bun run --cwd packages/utils lint
```
