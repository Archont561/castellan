---
type: Playbook
title: "Castellan — Testing"
description: "The testing vocabulary both languages share: rstest/proptest in Rust, bun test + fast-check + createFixture in TypeScript, and which shape to reach for."
tags:
  - testing
  - quality
status: stable
generated:
  by: agent/castellan-kb
  at: "2026-10-01T19:00:00Z"
updated: "2026-10-01T22:15:00Z"
id: infrastructure/testing
category: infrastructure
refs:
  - infrastructure/monorepo
  - infrastructure/ci
  - infrastructure/codegen
---
# Testing

Tests live in a `tests/` directory next to the code they prove — Rust as
integration tests in `crates/<name>/tests/*.rs` over the public API only
(exceptions that stay `#[cfg(test)]` inline: a bin crate's unit tests —
xtask's version parsing — and the derive-generated ts-rs export checks),
TypeScript `*.test.ts` in the package's `test/` under `bun test`. Three
shapes, one vocabulary across languages: what a `#[fixture]` is to a Rust
suite, `createFixture` is to a bun suite; what proptest is to a crate,
fast-check is to a package.

| Rust | TypeScript | What it proves |
| --- | --- | --- |
| `#[test]` | `test()` from `bun:test` | one specific, known behavior |
| `#[fixture]` | `createFixture` (`@castellan/utils/fixtures`) | shared setup, torn down after |
| `#[case]` matrices | example tests over explicit lists | many discrete shapes, one function |
| proptest | fast-check (`fc.assert`/`fc.property`) | "for any X" claims, shrunk on failure |

Where the engines live: rstest + proptest are dev-only
`[workspace.dependencies]`; fast-check + lefthook are root devDependencies;
`createFixture` is `@castellan/utils` (the same package that owns the
tsconfig bases) so no suite re-implements lifecycle wiring.

## The TypeScript side (bun test)

- **`createFixture(setup, teardown?, scope)`** registers hooks at suite
  scope and returns a getter. `"test"` scope (default) builds a fresh value
  per test via beforeEach/afterEach — isolation by default, because a test
  that mutates its fixture must not leak into the next test. `"file"` scope
  builds once per file via beforeAll/afterAll — the deliberate trade for
  expensive, immutable setup (the WASM artifact fixture is the flagship:
  wasm-bindgen's glue caches assume one instance per module lifetime).
  The getter throws when read before setup ran — a fixture wired into the
  wrong scope fails loudly at the call site. Self-tested in
  `packages/utils/test/fixtures.test.ts` (12 tests: both scopes,
  isolation, async, early-read guard).
- **Properties** wrap `fc.assert(fc.property(arb, body))` in a `test()`;
  `fc.asyncProperty` when the body awaits (the client's request-id
  property fires up to 50 concurrent calls before asserting). Expensive
  properties pin `numRuns` explicitly (50–200); cheap ones take the
  default 100.
- **Seed policy**: proptest failures commit their seed under
  `proptest-regressions/`; fast-check failures print the seed — re-run
  with `{ seed }` passed to `fc.assert`. `bun test --seed` seeds the
  runner's RNG, not fast-check's generator; it does not reach it.
- **Arbitraries are typed as the types they build** — a drift in a ts-rs
  export fails compilation of the test, not a release.

## What the properties currently pin

- `packages/protocol/test/roundtrip.test.ts` — every generated wire type
  survives `JSON.parse(JSON.stringify(v))` unchanged (11 types, one test
  each so failures name their type). The TS mirror of the crates' serde
  round-trips; catches a `bigint` id or a silently-vanishing field before
  a user's fill dialog does.
- `packages/core/test/properties.test.ts` — request ids are distinct and
  dense (exactly 1..N however calls interleave — the pairing key a
  multiplexing transport relies on), error code+message pass through
  byte-for-byte, the method spreads flat onto the request for any origin,
  and id sequences are per-client (a second client starts at 1: a shared
  port cannot assume global uniqueness).
- Rust: origin-matching asymmetries, otpauth round-trips, IPC framing
  round-trips (`arbitrary_payloads_round_trip` in castellan-ipc), TOTP
  shape/`code_now` agreement, passphrase word membership.

## Choosing a shape

A new **discrete example** gets a `#[case]` (or a plain example test). A
new **"for any X" claim** gets a property, in whichever language owns the
behavior — and its mirror if the contract spans the wire. The backlog
specifies its heaviest tests as properties (task-8's KDBX round-trip
harness over a corpus of real databases, task-27's sync idempotence);
they land in these same shapes.

See the docs page (`apps/docs/src/content/docs/testing.mdx`) for the reader-
facing version with code examples.

## The browser layer (one Playwright, one chromium)

Everything above the unit/property layer is Playwright, and the whole
repo pins **one version — 1.58.2**, chosen because it is the last stable
line of `@playwright/experimental-ct-svelte` (the svelte CT wrapper
stopped at 1.58.2 while `@playwright/test` moved on; playwright-core
refuses a browser package that does not match its own version, so one
pin is the only shape that keeps e2e + CT on a single chromium
download). Bump = move every `@playwright/*` pin together.

**Browser provisioning is a devDependency, not a manual step.** The root
`@playwright/browser-chromium@1.58.2` downloads chromium (full +
headless shell + ffmpeg) into the user-level Playwright cache during
`bun install` — its `install` script only runs because the package is in
root `trustedDependencies` (bun blocks lifecycle scripts by default;
note `playwright` itself has no postinstall since 1.5x, so there is
nothing to double-download). CI gets the same browser from the same
`bun install --frozen-lockfile` — no `playwright install` step exists
anywhere.

Who uses that one browser:

- **App e2e** — `e2e` scripts in desktop/mobile/extension, run by
  `all:e2e` = `turbo run e2e --concurrency=1`. Serial on purpose: two
  SvelteKit dev servers plus an extension build plus browsers will OOM
  small runners (proven the hard way — SIGKILLed dev server mid-run).
  Desktop (5173) and mobile (5174) use `--strictPort` webServers so a
  port drift fails loudly. The shared half of every config is
  `e2ePreset()` from `@castellan/utils/playwright` (the playwright
  sibling of `libPreset`).
- **Extension e2e** — `channel: "chromium"` is load-bearing: the
  default headless is `chrome-headless-shell`, which cannot load
  extensions at all; the channel is full Chrome-for-Testing in
  new-headless mode, which can. MV3 needs `launchPersistentContext`;
  the extension ID is read from the service worker's URL after waiting
  for it.
- **ui component tests** — `packages/ui`'s `test` script is
  `playwright test` (so CT rides `all:test`/`gates` like every other
  suite). CT needs `playwright/index.html` + a real (comment-only is
  fine) `playwright/index.js` next to it — the runtime injects itself
  into that entry; its build cache is `playwright/.cache/` (gitignored
  + biome-ignored).

### When the Playwright CDN is unreachable (`bun run browsers:offline`)

A sandbox or a locked-down network that cannot reach `cdn.playwright.dev`
loses the component tests *and* every e2e suite at once — most of the gate.
`scripts/offline-browsers.ts` is the fallback, and only that: CI never
calls it, and on a healthy machine it prints "already provisioned" and
exits.

It works because two npm packages ship a **binary inside the tarball**
instead of downloading one from a postinstall, so the npm registry is the
only host involved: `@sparticuz/chromium` (a brotli-compressed chromium
built for Lambda, with its NSS/NSPR libraries beside it) and
`@ffmpeg-installer/ffmpeg` (a static ffmpeg — Playwright needs one to
encode the failure videos `e2ePreset` asks for). Both are installed into
`$XDG_CACHE_HOME/castellan-offline-browsers` — never the repo, never the
lockfile: this is a property of the host, not of the project — and shimmed
into the layout Playwright's registry expects, with the revisions read out
of the installed `playwright-core/browsers.json` so a version bump needs no
edit.

The shim at each executable path is a **launcher script**, not a symlink:
it exports `LD_LIBRARY_PATH` for the bundled libraries, and it is also what
makes Playwright's host-requirements check pass, since `ldd` on a shell
script finds nothing missing.

What it does **not** give you is the extension suite. That chromium is a
headless shell with no extensions subsystem compiled in at all — no
`extensions::` symbols, no `chrome-extension://` scheme — so
`--load-extension` is silently ignored and the MV3 service worker never
registers. Component tests and the desktop/mobile e2e suites do pass on it
(verified: 10 CT + both shell specs).

### Gotchas this layer already caught (keep the tests in the loop)

- **`mount()` resolves to the component's root element**, not a
  wrapper: `row` for EntryRow *is* the button — click/assert the locator
  directly, and expect `icon.getByRole(...)` to fail on the svg the
  locator already points at (role locators match descendants only).
- **WXT + Svelte's package exports**: WXT's vite build resolved
  svelte's *server* entry (where `mount()` is a throwing stub), so the
  popup built green and rendered nothing (`lifecycle_function_unavailable`,
  2KB popup chunk instead of ~40KB). Fixed in `wxt.config.ts` with
  `resolve.conditions: ["browser"]` — a regression here is silent to
  every compiler; only the e2e suite sees it.
- **Svelte 5 renders `false` in text position as the string "false"**
  (Svelte 4 rendered falsy as empty) — `{cond && "text"}` must be
  `{#if}`. The popup leaked "false" lines until e2e caught it.
- **Storybook's builder reads the project's own `vite.config.ts`** —
  the framework no longer injects the svelte plugin, and stories use
  the `defineMeta` API (`<Meta>` is legacy: it builds fine and indexes
  **zero** stories, which looks exactly like success). Builder =
  `@storybook/svelte-vite` + `@storybook/addon-svelte-csf`;
  `vite.config.ts` in packages/ui carries the `svelte()` plugin.
- **`@playwright/cli` (the agent CLI) is version-independent** on
  purpose: it bundles its own alpha playwright, so its browser comes
  from `bunx playwright-cli install-browser chromium` (per machine),
  and it defaults to the *system chrome channel* — pass
  `--browser chromium`. The committed suites never depend on it.
