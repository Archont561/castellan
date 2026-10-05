# Playwright page and browser testing

Use Playwright page/integration tests for behavior that needs rendered route
composition, page API wiring, navigation, permissions, or an application shell.
Use E2E for a small set of cross-page, browser, session, or business journeys;
see [E2E](e2e.md) for scope.

## Navigation and readiness

Navigate directly to the route whenever the scenario can be seeded there, then
assert the rendered state. Test meaningful page lifecycle states: direct load,
loading, success, empty, error, permission denied, auth redirect, deep link,
back/forward/reload, and relevant form outcomes. Each test sets up its own
identity/data/storage and leaves no dependency on another test's order.

Wait on an observable state owned by the UI—a heading, loaded list, alert,
network fixture completion, or disappearance of a skeleton. `networkidle` is
not a general readiness rule, and `waitForTimeout` is not a synchronization
strategy. Use Playwright's auto-waiting plus explicit semantic assertions.

```ts
import { expect, test } from "@playwright/test";

test("expired session redirects a deep link to sign in", async ({ page }) => {
  await page.goto("/settings/security");
  await expect(page).toHaveURL(/\/sign-in\?returnTo=/);
  await expect(page.getByRole("heading", { name: "Sign in" })).toBeVisible();
});
```

## Locators and assertions

Prefer `getByRole(name)`, `getByLabel`, and stable visible text. Select semantic
regions, dialogs, tables, and forms before reaching for a test ID. A test ID is
legitimate when a repeated non-textual element has no stable user-visible name;
keep it stable and name it after its domain role. Do not encode markup shape in
selectors such as `.panel > div:nth-child(2)`.

Assert both the semantic state and the interaction result where they differ.
For a dialog, that may mean role/name, focus entering it, keyboard Escape
closing it, and focus returning to the opener. Do not use a screenshot as the
only evidence that a control works.

## Browser and responsive selection

Read the browser-support requirements and existing Playwright projects first.
Run a test in Chromium by default only when that is the project's fast feedback
policy; add WebKit, Firefox, device emulation, or real mobile targets when the
product support policy or changed behavior requires it. Choose representative
mobile/tablet/desktop widths at actual layout changes and test overflow,
wrapping, hidden controls, mobile navigation, dialog/table behavior, touch
sizes, and horizontal scrolling where applicable.

Keep each project/viewport's intended snapshot or behavior expectation clear.
Cross-browser screenshots need separate canonical baselines; browser differences
are not solved by one broad tolerance.

## Failure evidence

Use the repository's trace, screenshot, video, report, and `playwright-cli`
conventions to investigate failures. Capture test artifacts in CI for review,
not in version control. Diagnose render, behavior, a11y, visual, mock/network,
timing/race, environment, test-assumption, and product failures before changing
a spec or fixture.
