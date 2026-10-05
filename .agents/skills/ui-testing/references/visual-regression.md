# Visual-regression selection

Visual regression is a targeted visual contract. Invoke the repository's
`playwright-visual-regression` skill for capture configuration, baseline
lifecycle, diff review, and CI protocol; this reference decides when it belongs
in a UI testing model.

## Good candidates

Add a visual assertion when an unintended presentation change would matter and
is awkward to capture semantically: shared/layout components, dialogs, menus,
dropdowns, tables, complex form states, responsive compositions, and important
loading/empty/error states. Use a bounded component/region by default; use a
page capture only when page composition itself is the contract.

A button's label changing, a dialog opening, and a submitted form's outcome
still need semantic/interaction assertions. A visual test answers “does it look
right?”; interaction test answers “does it behave correctly?” The two should
share deterministic state but make different claims.

## Prerequisites

Use a stable Storybook/component state or deterministic page fixture. Fix data,
clock/timezone/locale, random seed, identity, API responses, viewport, color
scheme, fonts/environment where possible, and animation/caret behavior. Wait
for the actual content rather than a timeout. Mask only a documented,
irrelevant dynamic subregion; fixture it instead whenever possible.

Name snapshot state and viewport semantically. Capture the breakpoint where a
layout changes, not every possible width. Follow the project browser matrix;
canonical visual baselines are tied to their browser/OS/font environment.

## Review discipline

Run comparison mode first. Review expected, actual, and diff images before an
intentional targeted snapshot update; rerun normal comparison after the update.
CI should fail and retain artifacts on a mismatch, not automatically refresh
baselines. Do not use a global tolerance, masks, or snapshot updates to mute an
unknown change.
