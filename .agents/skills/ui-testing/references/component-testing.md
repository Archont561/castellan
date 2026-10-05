# Component testing and Svelte lifecycle

Use a component test or Storybook scenario when the component can expose the
user-visible contract without app routing or a real session. In Svelte, test the
component's props, snippets/slots, dispatched callbacks, rendered semantics,
and keyboard behavior through the DOM; treat `$state`, local variables,
reactivity wiring, and child implementation as private.

## Select states and transitions

Start from an observable state model, not a prop matrix. Include a state when it
changes rendered information, available action, interaction outcome,
accessibility semantics, layout, or dependency behavior. For a non-trivial
component, consider only the applicable parts of this lifecycle:

```text
mount → initial state → interaction → transition → async operation
      → loading → success / empty / error → updated UI → unmount
```

Also assess prop changes/re-rendering, disabled/read-only modes, validation,
retry/cancellation, stale responses, focus/keyboard behavior, and responsive
layout. An unmount test is valuable when an outstanding request, subscription,
timer, or callback could update a no-longer-present component; it is needless
for a pure presentational component.

Express transitions as user action and observation:

```text
closed --Activate filters--> open --Choose “Open”--> selected --Clear--> empty
idle --Submit--> loading --Server rejects--> error --Retry--> loading
```

One interaction test may prove several steps in a meaningful transition. Do not
split it into one test per DOM mutation unless separate failure diagnosis needs
that granularity.

## Svelte-oriented component tests

Use the repository's configured component-testing harness (in Castellan,
Playwright's Svelte component testing with `mount`). Mount a real component or a
small Svelte harness when it needs a public collaborator. A harness is valuable
when it supplies controlled callbacks, protocol-shaped client responses, or
observes a public event; it is not an excuse to inspect private state.

```ts
import { expect, test } from "@playwright/experimental-ct-svelte";
import PaymentFormHarness from "./fixtures/PaymentFormHarness.svelte";

test("user sees the server validation message and can correct the amount", async ({ mount }) => {
  const form = await mount(PaymentFormHarness, {
    props: { submitResult: { kind: "validation-error", message: "Amount is too high" } }
  });

  await form.getByLabel("Amount").fill("10000");
  await form.getByRole("button", { name: "Pay" }).click();

  await expect(form.getByRole("alert")).toHaveText("Amount is too high");
  await expect(form.getByRole("button", { name: "Pay" })).toBeEnabled();
});
```

Adapt the mount/props syntax to the installed version. The important evidence is
the label, button, and alert someone can use—not whether a callback was called
from a particular internal function.

## Forms and async behavior

Choose form evidence from the applicable states: empty, valid, invalid,
required/format errors, server validation, loading, success/failure, disabled
submit, double submit, dirty/unsaved changes, and keyboard submission. Cover
client and server validation when both affect the experience.

For async components, make timing controllable. Hold request A, start request
B, resolve B, then resolve A and assert B's result remains visible. Exercise
cancellation or unmount only where the component owns cancellable work. Prefer a
deferred test response, fake clock, or controllable promise to an arbitrary
sleep.

## Keep the component boundary honest

Move pure rules (formatting, parsing, date/math, complex transformations) to a
unit test. Move route/auth/multi-component behavior to a page test. Keep the
component test at its public UI seam. A component's story and interaction test
should make the same vocabulary of states discoverable to designers, reviewers,
and agents.
