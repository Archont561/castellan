---
name: playwright-visual-regression
description: Create, extend, review, or debug Playwright visual-regression tests and screenshot baselines as part of a frontend visual-TDD cycle. Use this whenever a user asks about visual testing, screenshot/snapshot assertions, pixel diffs, baseline approval, UI-regression coverage, or a frontend change can affect layout, typography, color, responsive behavior, overlays, loading/error states, or other rendered UI—even if they ask only to implement the UI change. Use the existing project test conventions; make rendering deterministic, exercise the intended user-visible state, review diffs, and never silently approve a changed baseline.
compatibility: Requires a project with Playwright Test or Playwright component testing. A stable browser/runtime and a way to serve the application are needed to create or compare baselines.
---

# Playwright visual regression

Visual regression is a **visual contract**, not an image-generation step. A
snapshot says that one meaningful user-visible state should keep its approved
appearance in a particular rendering environment. Use it alongside semantic,
behavior, and accessibility checks—not in their place.

This skill owns durable visual assertions and their baseline lifecycle.
`playwright-cli` / `webapp-testing` own one-off browser exploration; the `tdd`
skill owns the wider red → green discipline. Read
[references/playwright-patterns.md](references/playwright-patterns.md) when
writing a new harness or troubleshooting a flaky comparison. The originating
video research and captured source material are preserved in
[references/video-source.md](references/video-source.md); it is provenance,
not an API reference.

## Begin with the right visual contract

1. Read the repository instructions, existing Playwright configuration,
   package scripts, tests, and snapshot locations. Reuse its runner, fixtures,
   browser version, and commands. Do **not** add a second visual-test library
   or run `playwright install` if the repository provisions browsers another
   way.
2. State the user-facing seam and the exact state to cover before editing:
   **surface + state + viewport(s) + visual risk**. For example, “the
   authenticated billing card in its error state at 390 px and 1440 px; the
   risk is a mobile overflow from the new action.” If the product spec, design,
   or intended appearance is unclear, ask rather than turning the current
   rendering into an accidental specification.
3. Choose the narrowest honest surface:
   - **Component snapshot** for an isolated component/state. Prefer a story or
     component-test mount that makes the state explicit.
   - **Page/flow snapshot** when routing, real composition, shell layout,
     authentication, or an interaction is the thing under test.
   - Snapshot an **element** by default. Use a page screenshot for a deliberately
     bounded shell/layout contract. Use `fullPage` only when the entire,
     deterministic document is itself the contract; it otherwise makes diffs
     noisy and hides the changed area.
4. Cover the small set of states with meaningful visual risk: normal plus any
   changed loading, empty, error, disabled, selected/focus, open overlay, or
   narrow breakpoint state. Do not collect screenshots merely to raise a count.

## Run a visual TDD slice

Work one visible behavior at a time. Keep the test and the implementation in
the same change.

1. **Make the state controllable.** Seed fixed data and authenticated state;
   mock or fixture network responses; freeze time; fix locale, timezone, color
   scheme, viewport, and reduced-motion preference. Use stable image/font
   assets. Drive to the state through public UI/API seams rather than modifying
   component internals.
2. **Write the focused assertion first.** Name it after the user-visible state
   and give the snapshot a semantic, stable name such as
   `invoice-card--overdue--mobile.png`. Run it *without* updating snapshots.
   A missing expected image is useful initial red evidence that the scenario
   runs; a changed approved image is the real regression red signal. Never use
   an update command to make an unexplained failure disappear.
3. **Implement only the intended UI slice.** Wait for the visual readiness
   conditions you own: the identifying control/region is visible, the intended
   data is rendered, fonts are ready, and critical images have settled. Avoid
   a fixed sleep as a substitute for readiness.
4. **Inspect before approving.** On failure, open the Playwright report and
   inspect expected, actual, and diff (especially the slider view). First rule
   out wrong data, an unfinished state, unmasked nondeterminism, or an
   environment mismatch. Then fix an unintended change or explicitly review an
   intentional design change.
5. **Create or update a baseline only after that review.** Use the project’s
   targeted update command/test filter. Inspect the resulting image and the Git
   diff; commit the approved baseline with the code and test that justify it.
   Rerun the same test in normal comparison mode to prove it is green.
6. **Refactor only after green.** Preserve the semantic scenario and baseline
   name unless the user-visible contract changed. Run the relevant existing
   test command, then the required broader suite/CI lane.

A visual test is valuable only if a plausible unwanted layout or styling edit
would make it fail. Do not approve an image whose state you cannot explain.

## Make the screenshot deterministic

Start strict in a canonical environment. Fix causes of pixel noise instead of
hiding them with a broad tolerance.

- Fix `viewport`, `deviceScaleFactor` where appropriate, browser project,
  locale, timezone, color scheme, reduced motion, and mock data. Do not let
  live APIs, randomized copy, current dates, ads, A/B experiments, or a real
  clock decide pixels.
- Wait on concrete readiness signals. Useful signals are an expected heading,
  a loaded/skeleton-free region, `document.fonts.ready`, and critical images
  with `complete && naturalWidth > 0`. `networkidle` is not a universal
  readiness condition—persistent connections can make it misleading.
- Disable animations and hide the caret for the capture. Control CSS
  transitions, animated media, canvases, charts, and blinking cursors at the
  fixture level when they affect the assertion.
- Mask only a known, irrelevant dynamic subregion, with the reason documented
  next to the mask. A mask is **not** a way to silence a component, layout,
  price, status, or privacy regression. Prefer a deterministic fixture to a
  mask.
- Keep sensitive customer data, secrets, access tokens, and private images out
  of both the test fixture and committed PNGs.

Use the smallest sensible tolerance. Pixel-perfect comparison (`maxDiffPixels:
0`) is a good default in one pinned browser/OS environment. Increase
`maxDiffPixels`, `maxDiffPixelRatio`, or `threshold` only after a reviewed,
repeatable rendering difference remains; scope the exception to the one test
and explain the source. Do not use a global tolerance to quiet failures.

## Write legible Playwright assertions

Keep setup explicit and snapshots close to the behavior they describe. Adapt
imports and fixture names to the local runner.

```ts
import { expect, test } from "@playwright/test";

test.describe("invoice card / overdue", () => {
  test.use({
    viewport: { width: 390, height: 844 },
    deviceScaleFactor: 1,
    colorScheme: "light",
    locale: "en-US",
    timezoneId: "UTC",
    reducedMotion: "reduce"
  });

  test("keeps the overdue state usable on mobile", async ({ page }) => {
    await page.route("**/api/invoices/inv-42", (route) =>
      route.fulfill({ path: "tests/fixtures/invoice-overdue.json" })
    );
    await page.goto("/invoices/inv-42");

    const card = page.getByTestId("invoice-card");
    await expect(card).toHaveAttribute("data-state", "overdue");
    await expect(card.getByRole("button", { name: "Pay invoice" })).toBeVisible();
    await page.waitForFunction(() =>
      document.fonts.status === "loaded" &&
      [...document.images].every((image) => image.complete && image.naturalWidth > 0)
    );

    await expect(card).toHaveScreenshot("invoice-card--overdue--mobile.png", {
      animations: "disabled",
      caret: "hide",
      scale: "css",
      maxDiffPixels: 0
    });
  });
});
```

Use semantic locators and a semantic assertion before the screenshot, as in the
example. The screenshot then verifies presentation while the first assertion
helps distinguish a wrong state from a visual defect. Do not assert only a PNG
for functionality that needs an accessible/behavioral check.

Use `page.toHaveScreenshot()` for a bounded page composition and
`locator.toHaveScreenshot()` for a component/region. Keep names independent of
test wording likely to churn; let Playwright keep browser/project identifiers
in its normal snapshot layout unless the repository deliberately standardizes a
custom `snapshotPathTemplate`.

## Responsive and browser coverage

Define named projects or scoped `test.describe` blocks for the product’s
supported viewport classes. Include the breakpoint that changed and a nearby
wide viewport when the same component reflows differently. A minimal pattern:

```ts
for (const screen of [
  { name: "mobile", viewport: { width: 390, height: 844 } },
  { name: "desktop", viewport: { width: 1440, height: 900 } }
]) {
  test.describe(screen.name, () => {
    test.use({ viewport: screen.viewport });
    test("navigation is visually stable", async ({ page }) => {
      // drive the same deterministic state
      await expect(page.getByTestId("app-shell")).toHaveScreenshot(
        `app-shell--${screen.name}.png`,
        { animations: "disabled", caret: "hide" }
      );
    });
  });
}
```

Do not multiply every screenshot across every browser by habit. Use one pinned
engine for fast visual feedback unless browser-specific rendering is a product
requirement. If Firefox/WebKit are included, maintain their separate approved
baselines and review them independently; browser engines and operating systems
render fonts, native controls, shadows, and antialiasing differently.

## Baseline, report, and CI protocol

- Establish a **canonical capture environment**—normally the same OS, pinned
  Playwright version, browser build, fonts, locale, and color settings used by
  CI. Generate and compare a baseline there. Do not create a Linux baseline on
  macOS and then tune tolerances until it happens to pass.
- Version approved baseline PNGs with the tests unless the repository explicitly
  uses an approved external visual-testing service. Keep generated reports,
  actuals, diffs, videos, traces, and temporary exploration screenshots out of
  Git according to its ignore rules.
- Normal local and CI runs compare only. CI must fail on a mismatch and retain
  Playwright report/test-result artifacts so reviewers can see expected,
  actual, and diff images. CI must **not** automatically run
  `--update-snapshots` or commit baseline changes.
- Restrict update mode to a deliberate developer/maintainer action in the
  canonical environment. Target the test/project rather than refreshing the
  whole suite. Never approve a broad batch because it makes CI green.
- On a mismatch, classify it as **intentional design change**, **unintended
  regression**, **test/data/readiness defect**, or **environment drift**.
  Correct the cause, then compare again. An intentional design change still
  needs a human review of every changed baseline.

## Finish with an auditable handoff

Report: (1) visual seam and states covered; (2) test files/projects/viewports;
(3) command(s) run and result; (4) whether baselines were created/changed and
where; (5) diff-review outcome; and (6) remaining risk or intentionally
uncovered states. Mention any masked region or nonzero tolerance and why.

## Castellan-specific integration

When applying this skill in Castellan, preserve the repository’s testing
contract:

- Read `AGENTS.md` plus the target package’s `AGENTS.md` first. `packages/ui`
  uses Playwright component testing with Svelte; use its stories to make UI
  states reachable and run its existing package test command.
- All repository Playwright packages and Chromium are pinned together. The root
  `@playwright/browser-chromium` dependency provisions the browser during the
  workspace install; do not run `playwright install` by hand. If the browser
  cache is unavailable, follow the documented offline-browser fallback.
- The current visual-regression planning task calls out an explicit decision
  about canonical OS and baseline location. Surface and record that decision
  before introducing a large baseline set; do not silently choose a divergent
  convention.
- Use the repository’s agent-browser inspection loop for exploratory rendering,
  but turn an approved durable visual contract into a real Playwright Test
  assertion rather than committing ad-hoc CLI screenshots.
