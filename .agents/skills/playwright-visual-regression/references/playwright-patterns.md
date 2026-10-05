# Playwright visual-test patterns

Use this reference when a visual test needs a fixture, a configuration decision,
or a failure investigation. The main skill is the policy; these are adaptable
patterns rather than paste-everywhere boilerplate.

## Stable configuration

Keep project-wide defaults only when every visual test benefits from them.
Explicit per-test settings are often clearer for breakpoint-specific coverage.
Adapt the local test-runner API and existing configuration rather than replacing
it wholesale.

```ts
import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  snapshotPathTemplate: "{testDir}/{testFilePath}-snapshots/{arg}{ext}",
  expect: {
    toHaveScreenshot: {
      animations: "disabled",
      caret: "hide",
      scale: "css",
      maxDiffPixels: 0
    }
  },
  use: {
    locale: "en-US",
    timezoneId: "UTC",
    colorScheme: "light",
    reducedMotion: "reduce",
    screenshot: "only-on-failure",
    trace: "on-first-retry"
  },
  projects: [
    {
      name: "chromium-desktop",
      use: { ...devices["Desktop Chrome"], viewport: { width: 1440, height: 900 } }
    },
    {
      name: "chromium-mobile",
      use: { ...devices["iPhone 13"], viewport: { width: 390, height: 844 } }
    }
  ]
});
```

`expect.toHaveScreenshot` defaults are appropriate only in a controlled
canonical environment. If a project includes nonvisual tests with different
needs, apply these options to the visual assertions instead. Do not copy a
`devices` preset without understanding its viewport, scale factor, touch, and
user-agent implications.

## Readiness helper

A readiness helper should wait for signals owned by the application. It must
not hide a failure forever or assume all third-party requests finish.

```ts
import { expect, type Locator, type Page } from "@playwright/test";

export async function waitForVisualReady(page: Page, region: Locator) {
  await expect(region).toBeVisible();
  await page.waitForFunction(() =>
    document.fonts.status === "loaded" &&
    [...document.images].every((image) => image.complete && image.naturalWidth > 0)
  );
}
```

For a loading skeleton, wait for both the intended content and the absence of
the skeleton. For a chart, fixture the data and wait for the chart’s own
“rendered” marker; a screenshot cannot prove the chart is ready merely because
its canvas exists. For an animation that is part of the intended product state,
put it in a deterministic paused state rather than relying on a timing delay.

## Dynamic regions

Prefer fixture data:

```ts
await page.route("**/api/dashboard", (route) => route.fulfill({
  contentType: "application/json",
  body: JSON.stringify({ generatedAt: "2025-01-01T00:00:00Z", total: 42 })
}));
```

When the live value is explicitly irrelevant to the contract and cannot be
fixed, use a tight mask and leave a reason next to it:

```ts
const generatedAt = page.getByTestId("generated-at");
await expect(page.getByTestId("dashboard")).toHaveScreenshot("dashboard.png", {
  animations: "disabled",
  caret: "hide",
  mask: [generatedAt] // live timestamp is covered by a separate text-format test
});
```

Never mask the region whose placement, color, content, or visibility is the
thing under review. A mask’s opaque color itself becomes pixels in the baseline,
so ensure the locator is present and stable.

## Running, updating, and reviewing

Use the project’s package command. Typical Playwright Test commands are:

```sh
# Compare one named visual test; no baseline is changed.
bunx playwright test tests/invoice.spec.ts --project=chromium-mobile -g "overdue"

# Deliberately update only that reviewed contract in the canonical environment.
bunx playwright test tests/invoice.spec.ts --project=chromium-mobile -g "overdue" --update-snapshots

# Open Playwright’s expected/actual/diff review UI after a failure.
bunx playwright show-report
```

In a noninteractive/CI shell, use the project’s reporter/artifact conventions.
Do not append `--update-snapshots` to the normal test script. After an update,
run the first command again, inspect the PNG diff in version control, and verify
that no `test-results/`, report, actual, diff, trace, or video artifact is
staged.

## Failure triage

| Diff shape | Likely cause | First action |
| --- | --- | --- |
| Whole page is subtly noisy | OS/browser/font/version drift | Compare versions, canonical OS, font availability, and device scale before changing tolerance. |
| One timestamp/avatar/remote image | Live data or resource timing | Fixture it; use a narrow documented mask only if it is irrelevant. |
| Skeleton/spinner or blank region | Screenshot raced the state | Add a state-specific readiness signal; do not add a sleep. |
| Repeated one-pixel edge changes | Scale/antialiasing/font rendering | Pin capture environment; use a narrow justified tolerance only if still repeatable. |
| Large localized composition change | Product or CSS regression | Inspect test state and intended design; fix code or explicitly review/update baseline. |
| All browser projects fail after a test edit | Wrong route/fixture/selector | Assert the semantic state before the screenshot and repair the harness. |

## Review checklist

Before accepting baseline files, verify all of the following:

- The test reaches the intended route/component state with fixed data.
- The screenshot is taken after explicit readiness conditions.
- Every changed pixel matches an approved product/design change.
- The change is limited to expected browser projects and viewport states.
- No sensitive data appears in a committed image.
- Masks and nonzero tolerances remain narrow, justified, and test-local.
- Normal (non-update) comparison passes after the update.
