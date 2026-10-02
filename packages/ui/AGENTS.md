# AGENTS.md — working in `@castellan/ui`

This file applies to `packages/ui/**`. The repository-level `AGENTS.md` still
applies; this file narrows the rules for Castellan's shared Svelte UI package.

## Purpose of this package

`@castellan/ui` is the shared Svelte component library for the desktop and
mobile faces. It owns reusable view components, Castellan-styled wrappers
around headless primitives, the UI package's Storybook stories, and component
interaction tests.

It must **not** own transports, generated clients, vault access, Tauri APIs,
WXT/browser APIs, or any secret-bearing business logic. Faces pass data and
callbacks in; components render state and emit intent.

Canonical design spec:

```txt
backlog/docs/specifications/ui-system/doc-12 - Castellan-UI-System-Specification.md
```


## Relevant project skills

Use `.agents/skills/frontend-design` before inventing or revising visual
direction: palette, typography, hierarchy, motion, and the "does this look like
a generic AI UI?" self-critique. Use `.agents/skills/webapp-testing` or
`.agents/skills/playwright-cli` for the render-and-inspect loop; the former is
better when a short native Playwright script is clearer, the latter is better
for interactive snapshots.

## Dependency boundary

The shared app UI may use the dependency scaffold from TASK-47:

- `bits-ui` for unstyled accessible primitives.
- `@floating-ui/dom` for positioning.
- `@harshmandan/svaul` for Svelte 5 mobile sheets/drawers.
- `lucide-svelte` for icons.
- `@internationalized/date` as the Bits UI date peer.

Desktop and mobile apps should consume Castellan wrappers from this package
instead of depending on those primitive libraries directly. The extension is a
separate surface: injected/page UI should stay small, Shadow-DOM-contained,
and use `@floating-ui/dom` directly from `apps/extension`.

## Component organization

Use this split when adding files:

```txt
packages/ui/src/primitives/   thin Castellan wrappers around headless pieces
packages/ui/src/components/   product components made from primitives
packages/ui/src/layouts/      shell/pane/list-detail layout components
packages/ui/tests/            Playwright component tests
packages/ui/src/**/*.stories.svelte
```

Current components live under `src/components/`; do not move them just to make
the tree match until the related component migration task is doing that work.

## Styling rules

- Styling is UnoCSS. Shared component looks belong in `packages/ui/uno.ts` as
  `c-*` shortcuts.
- One-off metrics stay in markup because desktop and mobile legitimately use
  different density, width, and touch-target decisions.
- Never define the same CSS property both in a shortcut and as a utility on the
  same element. Source order would decide the winner.
- Keep the bare semantic/test class first: `row c-entry-row ...`,
  `username ...`, `badge ...`, `error ...`. That class carries no styling and
  protects tests from design churn.
- Do not put shortcut tables under `src/**`; consumers scan this package's
  source, so a shortcut table under `src` would make every utility it mentions
  ship everywhere.

## Accessibility and behavior baseline

Every interactive component needs the keyboard and screen-reader contract in
the UI system spec, not only a pointer happy path.

Required by default:

- visible focus state;
- correct native element or ARIA role;
- Enter/Space behavior for buttons and rows;
- Escape closes transient surfaces;
- focus returns to the opener after dialogs, menus, popovers, and sheets;
- 44 px minimum mobile touch target, 32 px minimum compact desktop target;
- secret reveal/copy actions are explicit and field-local;
- destructive actions require either an undoable trash step or confirmation.

## Data and security rules

- List components render metadata only. Do not add password, TOTP seed, passkey
  private key, recovery code, or vault-file bytes to list props.
- Secret fields render one secret at a time and expose explicit callbacks for
  copy/reveal. Clipboard timers and lock policy come from the face/core layer;
  this package only renders state and invokes callbacks.
- Origin/RP-ID decisions are not UI decisions. The app process enforces them;
  UI components display the result.

## Tests and stories

A new reusable component should normally land with:

1. a Storybook story for default, empty, loading/error, disabled, focused, and
   mobile-density states where relevant;
2. a Playwright component test for keyboard behavior and visible state;
3. a semantic class or accessible name that the app e2e suites can query.

For wrappers around headless primitives, test Castellan's contract, not the
third-party library internals: focus restoration, events/callbacks, labels,
open/close behavior, and dangerous-action gating.


## Agent visual/spec review loop — offline browser

For any UI change that is visible, run a tight inspect → compare → fix loop
against the canonical UI spec. Do not rely on code review alone; many failures
in this package are spacing, focus, overflow, and state-representation bugs.

Preferred loop:

1. **Name the contract first.** Identify the exact spec section and component
   states you are changing in
   `backlog/docs/specifications/ui-system/doc-12 - Castellan-UI-System-Specification.md`.
2. **Render the smallest useful surface.** Prefer a Storybook story for a
   component; use the desktop/mobile app only when the behavior depends on the
   app shell, routing, or face-specific metrics.
3. **Open it with the offline browser.** If the local Playwright browser cache
   is missing, run `bun run browsers:offline` once. Then open the local URL
   with `bunx playwright-cli open --browser chromium <url>`; the tool writes
   snapshots under `.playwright-cli/` for inspection.
4. **Compare to the spec, not to memory.** Check layout shape, density,
   responsive behavior, empty/loading/error/locked states, focus order, Escape
   handling, action placement, and the secret-display rules.
5. **Exercise input, not just pixels.** Tab through controls, press
   Enter/Space/Escape, use arrow keys in lists/menus, try touch-sized mobile
   viewports, and verify focus returns after dialogs, menus, popovers and
   sheets close.
6. **Classify divergence.** If implementation violates the spec, fix the code
   or tests. If the spec is wrong or incomplete, update the spec in the same
   change and say why. Never silently normalize a mismatch as "close enough".
7. **Clean artifacts.** Delete ad-hoc screenshots, downloaded reference images,
   and temporary traces unless a task explicitly asks for committed visual
   baselines.

Useful commands from the repository root:

```console
$ bun run browsers:offline                         # provision offline Chromium when needed
$ bun run dev:storybook                            # inspect package stories first
$ bunx playwright-cli open --browser chromium <url>
$ bun run --cwd packages/ui test                   # component behavior tests
```

When using Arena live previews instead of the agent's offline browser, make the
server bind `0.0.0.0` and keep browser-facing URLs relative. The offline browser
loop may use local URLs inside the sandbox; the user's browser cannot.

## Imports

Package source must use relative imports for package-local files. Do not use
`@/...` inside `packages/ui/src/**`; the consuming app owns that alias at build
time, so it would resolve to the app, not this package.
