---
id: TASK-38
title: "E2E depth: vault flows, autofill, and visual regression baselines"
status: To Do
assignee:
  - '@agent'
created_date: '2026-10-01 20:45'
updated_date: '2026-10-01 20:45'
labels:
  - testing
  - quality
dependencies: [TASK-8, TASK-11, TASK-13]
references:
  - backlog/decisions/decision-9
priority: medium
ordinal: 3800
---

The browser-testing infrastructure is in place (decision-9): one pinned
Playwright, prebundled chromium, per-app e2e, ui component tests,
Storybook. What exists today are honest smoke tests — shells render,
absent backends are reported, the extension registers its service worker.
Depth lands with the features:

- **Vault flows (desktop + mobile)**: unlock, entry list from a real
  (test-fixture) vault, entry detail, TOTP countdown against a
  deterministic clock. Needs a transport stub the e2e layer can mount —
  the Transport seam exists for exactly this (`disconnected` is one
  implementation; a scripted fixture transport is the test-side twin).
- **Autofill (extension)**: load the built extension against a fixture
  page, drive the fill prompt, assert the page received credentials via
  the content-script bridge — the passkey/autofill path end to end.
- **Visual regression baselines**: `toHaveScreenshot()` for the ui
  components (Storybook stories make every state reachable) and the app
  shells. Decisions needed: per-OS baselines vs single-platform CI, and
  where baseline PNGs live in the repo.
- **Determinism**: TOTP and lock timeouts need clock control
  (`page.clock`) before countdown assertions stop being flake bait.

Acceptance: e2e suites grow with each feature task above rather than in a
separate hardening pass; every new ui component ships stories + component
tests as part of its task.
