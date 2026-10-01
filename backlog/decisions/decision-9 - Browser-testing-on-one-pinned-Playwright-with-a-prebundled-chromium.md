---
id: decision-9
title: "Browser testing on one pinned Playwright with a prebundled chromium devDependency"
date: '2026-10-01 20:45'
status: accepted
refs:
  - infrastructure/testing
  - infrastructure/ci
---
## Context

The repo needed three browser-testing consumers at once: app e2e (desktop,
mobile), extension e2e (a built MV3 loaded into a real browser), and
component tests for the ui package — plus an agent-facing CLI for
exploration with on-disk snapshots. Playwright's own distribution made two
problems visible:

1. `playwright install` is a manual, machine-local step — exactly the
   "works on my machine" drift the repo refuses everywhere else (pinned
   toolchains, lockfiles as anchors).
2. `@playwright/experimental-ct-svelte` (component testing for Svelte)
   stopped at **1.58.2** while `@playwright/test` moved on. Playwright-core
   refuses a browser package that does not match its own version, so an
   unpinned setup means either two chromium downloads or a launch failure.

## Decision

**Every `@playwright/*` package in the repo is pinned to 1.58.2 and moves
together or not at all.** The chromium binary comes from the root
`@playwright/browser-chromium@1.58.2` devDependency, whose install script
bun runs because the package is in `trustedDependencies` — one browser for
all three suites, downloaded by `bun install`, on contributors' machines
and in CI identically. No `playwright install` step exists anywhere.

The agent CLI (`@playwright/cli`) is deliberately outside this pin: it
bundles its own alpha Playwright, gets its browser from
`playwright-cli install-browser chromium` per machine, and nothing
committed depends on it.

E2e runs serially (`turbo run e2e --concurrency=1`): two dev servers plus
an extension build plus browsers is a RAM budget, not a race — parallel
runs OOM-killed a dev server on the first try.

## Consequences

- A Playwright bump touches every pin at once (grep `1.58.2`); CT-svelte
  lag bounds how far that can go until it ships a newer stable.
- The extension e2e depends on `channel: "chromium"` (full Chrome-for-
  Testing in new headless — the default headless shell cannot load
  extensions at all), and its `e2e` script chains `wxt build`, so the
  tested artifact is the shipped one.
- The e2e suite is the only gate that can catch rendering-level
  regressions the compilers cannot see — it found two shipped bugs on its
  first run (Svelte *server* runtime bundled into the popup; `false`
  stringified into text), see `infrastructure/testing.md`.
