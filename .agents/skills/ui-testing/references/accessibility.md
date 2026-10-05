# Accessibility evidence

Accessibility is a behavior requirement from the first UI test model, not a
post-implementation audit. Choose evidence that corresponds to the interaction
being changed.

## Automated checks

Use the configured Storybook accessibility addon/axe integration for stories
and Playwright-compatible axe tooling for page states when available. Run scans
on meaningful normal and error/dialog states, and treat results as actionable
failures unless a reviewed documented exception applies.

```ts
// Use only when the project has configured @axe-core/playwright.
import AxeBuilder from "@axe-core/playwright";

const results = await new AxeBuilder({ page }).analyze();
expect(results.violations).toEqual([]);
```

Automated scanners catch many detectable semantic and contrast problems, but
cannot decide whether wording is clear, focus order is useful, an announcement
is timely, a gesture has an equivalent, or a complex custom widget is
understandable. Important flows still need manual keyboard, browser/device, and
screen-reader assessment appropriate to the product's accessibility commitment.

## Interaction contract

For an interactive component/page, select the applicable checks:

- native element or correct role, accessible name, and associated label;
- keyboard access (Tab, Shift+Tab, Enter, Space, arrow keys where expected);
- visible focus and predictable focus movement/restoration;
- dialog/menu/popover focus containment where appropriate and Escape closure;
- disabled semantics that match actual operability;
- validation and server errors exposed with useful semantics and focus policy;
- loading/status announcements when waiting changes the user's next action;
- expanded/selected/invalid/busy ARIA state only where the native element does
  not already convey it;
- automated color/contrast checks plus manual verification of important visual
  states.

Query tests through these semantics. If a button has no usable name, fix the
product instead of installing a test-only selector.

## Practical test shape

Start with a semantic existence/assertion, then exercise a keyboard transition
and assert its observable result. For a dialog, test the opener, focus entering
its labelled dialog, Escape or cancel closing it, and focus returning to the
opener. Avoid asserting raw ARIA attributes when the equivalent user-visible
behavior provides stronger evidence; assert attributes when they are the
contract, such as `aria-expanded` on a custom disclosure.
