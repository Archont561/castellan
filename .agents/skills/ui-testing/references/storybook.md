# Storybook scenarios and interaction tests

Treat a Storybook story as an executable UI scenario: a named, typed starting
state that can be rendered, interacted with, scanned for accessibility issues,
and captured visually. Keep Svelte stories beside the component using the
repository's configured Svelte CSF convention; do not use React-only patterns.

## Choose meaningful stories

Name the state, not the implementation. A DatePicker might have `Default`,
`Selected`, `Disabled`, `Loading`, `Error`, `Empty`, `Invalid`, `LongContent`,
and `Mobile`, but include only states that alter user-visible behavior, layout,
a11y, or API outcome. Stories should make review and testing easier, not become
a combinatorial prop catalogue.

Use fixed props, dates, random seeds, locale, user identity, and MSW handlers.
A story must not depend on production APIs, current time, or data left by a
previous story. A story for an async component should make its terminal and
loading states intentional instead of waiting for a real request.

## Svelte CSF pattern

The exact API follows the installed Storybook version. In a Svelte CSF setup,
the usual shape is a module script with `defineMeta`, followed by typed `Story`
instances. Keep fixtures in the story or shared test fixture only when they
represent a reusable scenario.

```svelte
<script module lang="ts">
  import { defineMeta } from "@storybook/addon-svelte-csf";
  import DatePicker from "./DatePicker.svelte";

  const { Story } = defineMeta({ title: "Components/DatePicker", component: DatePicker });
  const selectedDate = new Date("2026-10-15T12:00:00Z");
</script>

<Story name="Selected" args={{ value: selectedDate, disabled: false }} />
<Story name="Disabled" args={{ value: selectedDate, disabled: true }} />
```

Do not treat the exact snippet as a substitute for local configuration. Check
whether the project uses Svelte CSF, CSF object stories, portable stories, or a
Storybook test runner, then follow it consistently.

## Test transitions with play functions

Use a story `play` function for isolated interactions whose start state is the
story. Locate controls as a person would, simulate keyboard/pointer input, and
assert the resulting visible/semantic state. A play function should read like a
short user journey, not a long UI automation script.

```ts
// Pseudocode: import helpers from the Storybook test package configured here.
export const SelectAndClear = {
  args: { value: null },
  play: async ({ canvas, userEvent, expect }) => {
    await userEvent.click(canvas.getByRole("button", { name: "Choose date" }));
    await userEvent.click(canvas.getByRole("gridcell", { name: "15 October 2026" }));
    await expect(canvas.getByRole("button", { name: "Clear date" })).toBeVisible();
    await userEvent.click(canvas.getByRole("button", { name: "Clear date" }));
    await expect(canvas.getByText("No date selected")).toBeVisible();
  }
};
```

Use component-test specs when the project's runner gives better debugging,
fixtures, or browser coverage. Do not duplicate the same transition in both a
play function and component test unless each produces distinct value (for
example, portable story reuse versus a browser-specific focus contract).

## Storybook as a test hub

When configured, run its interaction, accessibility, and visual tools against
these same stories. A Story establishes initial state; interaction tests answer
“does it behave?”; visual tests answer “does it look right?”; a11y scanning
answers “are detectable semantic violations present?” None replaces the others.
