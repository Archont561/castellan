# `@castellan/ui`

Shared, prop-driven Svelte components for the desktop and mobile faces. The
package renders user-visible state and emits intent through callbacks; it does
not construct transports, call generated clients, access Tauri APIs, or hold
secrets. Each face owns client construction, platform input, and density
metrics.

## Components and stories

| Area | Purpose |
| --- | --- |
| `src/components/` | Reusable shared Svelte surfaces such as `VaultHome`, `OtpImport`, and `BrowsersPanel` |
| `src/**/*.stories.svelte` | Svelte CSF stories beside their components; each is a deterministic UI scenario |
| `tests/` | Playwright Svelte component tests and narrowly scoped visual-style contracts |
| `uno.ts` | Shared component look shortcuts; one-off face metrics remain in consuming markup |

Interactive components need meaningful default, loading/empty/error, disabled,
focused, and mobile-density stories where those states differ. Tests assert
user-facing roles/names, keyboard behavior, public callbacks, and deterministic
state transitions. The shared [UI-testing skill](../../.agents/skills/ui-testing/SKILL.md)
and [`AGENTS.md`](AGENTS.md) describe the test model.

## Commands

```console
$ pixi run dev-storybook
$ pixi run storybook-build
$ pixi run bun run --cwd packages/ui typecheck
$ pixi run bun run --cwd packages/ui test
```

The apps compile this package from source. Keep package-local imports relative;
the consuming app owns the `@` alias. Playwright and Chromium are pinned at the
workspace level—do not run `playwright install` by hand.
