---
name: ui-testing
description: Plan, implement, review, or repair maintainable UI tests for Svelte, Storybook, Playwright, and TypeScript applications. Use whenever a request changes or adds visible UI, a component, form, route, authentication/permission behavior, an API-driven screen, responsive layout, browser interaction, or frontend bug—even if the user asks only for implementation. Model user behavior, states, transitions, and external dependencies; choose the narrowest useful test level; work test-first with the tdd skill; use deterministic mocks, accessibility, and selective visual/E2E coverage; then refactor tests and implementation after green.
compatibility: Works best with Svelte, Storybook, Playwright, TypeScript, MSW, and a project test runner. Inspect the repository and adapt to its installed versions, configured test tools, browser matrix, and scripts rather than assuming React APIs or adding duplicate tooling.
---

# UI testing

A UI test is evidence for a **user-visible contract**: what someone can see,
do, and recover from. Build the smallest set of independent tests that makes a
plausible regression observable. High confidence comes from complementary
layers—not from repeating one assertion in every layer.

## Supporting skills and boundaries

Use these skills as part of this workflow rather than duplicating their full
procedures here:

- **`tdd`** — invoke before implementation for the red → green behavior loop,
  including its seam agreement. This skill supplies the UI test model and test
  level; `tdd` supplies the detailed test-first discipline.
- **`refactor`** — invoke only after relevant tests are green. Use it to make
  production code and tests clearer without changing their contract.
- **`playwright-visual-regression`** — invoke after a visual contract is
  selected. It owns deterministic screenshot capture, baseline approval, and
  visual-diff diagnosis.
- **`skill-creator`** — use when changing or evaluating this skill itself:
  create realistic eval prompts, validate the frontmatter, package it, and
  review whether its instructions improve agents' outputs.

`playwright-cli` and `webapp-testing` are useful for exploratory inspection and
failure debugging. They do not replace durable component, page, or E2E tests.

## 1. Inspect before choosing tests

Read the repository and target area before writing a spec. Locate the component
or route, its public inputs/events, existing stories, component tests, page/E2E
specs, API calls, fixtures, mock handlers, Playwright and Storybook config,
package scripts, accessibility/visual tooling, and browser-support policy.

For Svelte, prefer the existing Svelte component-test and Svelte CSF patterns.
Treat a story as a typed, reusable scenario—not as marketing-only preview
content. Reuse local fixtures, test helpers, and project commands. Do not add a
second runner, snapshot product, or mocking library to solve a problem an
installed tool already handles.

Finish inspection by writing a compact **test model** in the plan or work log:

```text
Seam: <component, page, or journey exposed to a user>
User goal: <what success means>
State model: <initial, loading, success, empty, error, disabled, ...>
Transitions: <action → observable result; async outcomes>
Dependencies: <local, API, external system; control method>
Risks: <a11y, responsive, race, permissions, visual composition>
Selected evidence: <unit / component / story play / a11y / visual / page / E2E / contract>
Deliberately excluded: <redundant layer and why>
```

A state deserves its own evidence only when it changes what a person sees, can
do, receives after an action, hears through assistive technology, or how the
layout/API behaves. Do not mechanically enumerate prop permutations.

## 2. Select the narrowest honest level

| Level | Use for | Do not use when |
| --- | --- | --- |
| **Unit** | Pure parsing, formatting, transformations, calculations, and complex deterministic business rules | The question is whether a person can use the rendered UI |
| **Component / Storybook** | A component state, its public props/events, keyboard interaction, validation, loading/empty/error, component a11y, or local visual contract | Routing, multiple real surfaces, or app/session composition is essential evidence |
| **Page / integration** | Components composed with route state, page-level APIs, forms, navigation, permissions, and auth state | The behavior is already fully proved by one isolated component or needs a cross-page browser journey |
| **E2E** | A small set of critical journeys, deep links, browser behavior, lifecycle/session behavior, and cross-page business workflows | It merely repeats a component's disabled button or validation assertion |
| **Contract / real integration** | Important external API/schema assumptions and a small real smoke surface | Deterministic feedback on every UI change is the goal |

A typical strategy is unit tests for logic, component/story scenarios for most
UI behavior, page tests for composition, and a small E2E set for the business
paths that matter most. Keep the unique assertion at its lowest reliable
layer; higher layers should prove composition, not duplicate it.

Read [component testing](references/component-testing.md) for state/lifecycle
selection, [Storybook](references/storybook.md) for executable scenarios, and
[Playwright](references/playwright.md) for page/browser testing.

## 3. Drive a UI TDD slice

For each visible behavior, invoke `tdd` and work one tracer bullet at a time:

1. Name the user-visible requirement and agreed seam. Model the relevant states
   and transitions, for example `idle → submit → loading → success` and
   `idle → submit → loading → error → retry → loading`.
2. Add or update the smallest story/specification that can reach the starting
   state. Write an interaction assertion with user-oriented locators. Run it
   before implementation and confirm it is red for the expected missing or
   incorrect behavior—not because setup is broken.
3. Implement only enough Svelte/TypeScript behavior to turn that assertion
   green. Test public props, events, rendered semantics, and user input; do not
   reach into component state or framework internals.
4. Run the focused test, then the nearest relevant group. Add secondary
   evidence only when the test model shows a distinct risk: a11y, visual,
   responsive, page integration, a race, or an E2E journey.
5. When green, invoke `refactor`. Remove duplication, improve names and
   readability, simplify mocks/selectors, and extract helpers only for stable
   domain actions. Re-run the same tests to prove behavior is unchanged.

Stories and `play` functions are ideal for isolated transition behavior; page
and E2E tests should reuse the same state vocabulary where they cover
composition. See [Storybook](references/storybook.md).

## 4. Make tests user-oriented and isolated

Use locators in this order: accessible role plus name; associated label;
visible text where it is stable and meaningful; semantic HTML; then a stable,
purposeful test ID. A test ID is appropriate when no accessible/semantic hook
can unambiguously identify a repeated or non-textual control. It is not a
substitute for a missing accessible name.

Write behavior names: “user cannot submit invalid details,” “expired session
redirects to sign in,” or “selected country remains after reload.” A helper
should speak the domain (`datePicker.selectDate("2026-10-15")`), not hide an
arbitrary chain of DOM operations.

Each test owns its setup and cleanup. Seed state through fixtures, APIs,
controlled storage, MSW, or a database fixture; never require an earlier test
to create it. Fix clocks, random values, locale, timezone, viewport, network
responses, and logged-in identity whenever they affect the result. Wait for a
state the UI owns, not arbitrary timeouts.

## 5. Add targeted secondary evidence

- **Accessibility is part of the primary behavior.** Check semantics,
  accessible names, labels, keyboard operation, focus movement/restoration,
  Escape behavior, disabled/error/loading semantics, and relevant ARIA. Run
  configured Storybook/axe checks and add manual keyboard, screen-reader, or
  device review for important flows that automation cannot judge. See
  [accessibility](references/accessibility.md).
- **Visual regression answers “does it look right?”** Add it for shared or
  layout-heavy components, overlays, tables, responsive layouts, and important
  loading/empty/error compositions. It complements behavioral assertions; it
  does not replace them. Use deterministic stories/fixtures and the
  `playwright-visual-regression` skill. See
  [visual regression](references/visual-regression.md).
- **Mock API behavior intentionally.** Use MSW in stories/component/page tests
  for the few meaningful API outcomes. Use a sanitized Playwright HAR/cassette
  at application level when replaying an external exchange is valuable. Keep a
  small contract or real-service smoke surface for external compatibility. See
  [mocking](references/mocking.md).
- **Test async hazards when they exist.** Exercise slow, failed, cancelled,
  repeated, or out-of-order operations when the UI can observe them. The
  `pol` request completing after the later `poland` request must not replace
  the newer result with stale data.
- **Test responsive and role boundaries where they change behavior.** Choose
  meaningful mobile/tablet/desktop breakpoints and the distinct auth/permission
  boundaries; do not replicate the full suite for each viewport or role.

## 6. Compose pages and journeys deliberately

At page level, verify the route composes already-tested components into the
correct state: direct navigation, load/success/empty/error, permission/auth
redirects, navigation/back/forward/reload/deep links, and form outcomes where
app wiring matters. For forms, select applicable coverage from empty, valid,
invalid, required/format errors, server errors, loading, success, disabled and
double submit, dirty/unsaved state, and keyboard submission.

Add E2E only when the whole journey has distinct value: authentication
lifecycle, cross-page navigation, deep link, browser feature, or a critical
business workflow. Inspect the supported-browser matrix first. It is often
right to run component behavior in Chromium, visual tests in a canonical
Chromium plus required mobile/WebKit targets, and only critical E2E journeys on
the supported browser matrix. The project requirement—not habit—decides.

Read [E2E](references/e2e.md) before adding or broadening journey coverage.

## 7. Run, diagnose, and maintain

Run the focused spec first; then the changed component/story/a11y/visual suite,
the relevant page test, and the E2E journey if the model selected one. Run the
entire repository only when the project policy or risk warrants it. In CI,
compare normal visual baselines, retain reports/traces/diffs on failure, and
never update snapshots automatically.

Classify every failure before changing it:

```text
render | behavior | accessibility | visual | network/mock | timing/race
environment | test assumption | product regression
```

Inspect the state and evidence. Correct an intentional product change through
reviewed test/fixture/baseline updates; repair a product regression; make a
flaky test deterministic; or fix an invalid assumption. Do not weaken an
assertion, raise a visual threshold, change a mock, or refresh a snapshot until
the cause is understood.

After green, use `refactor` to remove brittle selectors, duplicated fixtures or
handlers, overlong play functions, giant page objects, and generic helpers that
hide product intent. Keep test abstractions at domain boundaries.

## Avoid false confidence

- Keep evidence at public seams: user semantics and outcomes, not CSS classes,
  private component state/functions, framework mechanics, fragile DOM paths, or
  unnecessary test IDs.
- Keep the test pyramid intentional: component/page tests carry most UI detail;
  E2E proves a few composed journeys; visual snapshots complement behavior and
  accessibility rather than replacing either; unhappy states matter as much as
  happy paths.
- Keep the environment controlled: normal CI uses fixtures, MSW, or sanitized
  replay—not production APIs or a live third party. Cassettes contain neither
  credentials nor personal/production data and never become the only contract
  evidence.
- Keep failures informative: understand a diff or assertion before changing a
  snapshot, threshold, mock, timeout, or expectation. A passing update command
  is not approval.
- Keep tests independent: no execution-order state, shared mutable fixtures,
  arbitrary waits, or generic page objects/helpers that conceal domain intent.
  Do not add the same assertion at each level merely to increase the count.
- Keep the TDD loop honest: write the behavior specification before the change
  where TDD is appropriate, prove it turns red for the intended reason, then
  make the smallest green change and refactor from the protected state.

## Definition of done

Before handoff, verify the applicable items:

- [ ] User goal, states, transitions, and risks are named.
- [ ] The narrowest useful test level is selected; redundant coverage is
      explicitly avoided.
- [ ] A focused test/specification was red for the expected reason before the
      minimal implementation turned it green.
- [ ] Tests use user-facing locators, independently owned setup, deterministic
      data/time/network, and behavior-oriented names.
- [ ] Relevant keyboard, focus, semantic, error/loading, and responsive
      behavior has coverage.
- [ ] MSW/HAR/contract/real-service choices fit the dependency and contain no
      secrets or production personal data.
- [ ] Visual baselines, if valuable, were reviewed intentionally; no snapshot
      was blindly approved.
- [ ] Page/E2E coverage proves composition or a critical journey without
      repeating component behavior.
- [ ] Refactoring preserved green tests, and the selected test suites pass.

## Handoff report

State the user behavior and test levels covered; stories/specs/fixtures or
handlers/cassettes changed; a11y, visual, responsive, page, and E2E evidence;
commands run and results; snapshots approved; and deliberate gaps or remaining
external-contract/manual-testing risk.

## Castellan integration

For this repository, read root and target `AGENTS.md` first. `packages/ui`
uses Svelte CSF stories beside components and Playwright Svelte component tests;
its stories should expose meaningful presentational states and fixtures should
script face-client behavior. The repository pins Playwright and Chromium
through the workspace; use its package scripts and browser provisioning rather
than running `playwright install`. Follow the existing offline-browser fallback
only when necessary. Current Storybook/a11y/MSW/HAR tooling may be absent from
some packages: inspect first, then add only the minimal project-approved
integration required by the test model.
