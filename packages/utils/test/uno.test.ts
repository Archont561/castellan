/**
 * The shared UnoCSS config, proven without a browser.
 *
 * Two things here are load-bearing and silently breakable. The first is the
 * extraction pipeline: @castellan/ui is consumed as source, so if the
 * include pattern stops matching its files, every shared component still
 * compiles, still renders, still passes its component tests inside the
 * library's own harness — and arrives in the apps with class attributes and
 * no CSS. The second is that an unmatched utility inside a shortcut is a
 * warning UnoCSS prints and carries on from (it cost this suite's author one
 * silently-dropped `transition-property`), so the component looks are
 * asserted by the declarations they produce, not by the class names.
 */
import { describe, expect, test } from "bun:test";
import { createGenerator } from "unocss";

import { unoPreset } from "@/src/uno";

/** The declarations UnoCSS generates for a space-separated class list. */
async function declarationsFor(classes: string, shell = true): Promise<string> {
  const uno = await createGenerator(unoPreset({ shell }));
  const { css } = await uno.generate(classes, { preflights: false });
  return css;
}

describe("the extraction pipeline", () => {
  const { include = [], exclude = [] } = (unoPreset().content?.pipeline ?? {}) as {
    include?: RegExp[];
    exclude?: RegExp[];
  };

  const included = (id: string): boolean =>
    include.some((pattern) => pattern.test(id)) && !exclude.some((pattern) => pattern.test(id));

  test("scans the shared component package, resolved either way", () => {
    // vite hands us the real path by default…
    expect(included("/repo/packages/ui/src/components/EntryRow.svelte")).toBe(true);
    // …and the symlinked one under `preserveSymlinks`.
    expect(
      included("/repo/apps/desktop/node_modules/@castellan/ui/src/components/EntryRow.svelte")
    ).toBe(true);
  });

  test("scans the consuming app's own source", () => {
    expect(included("/repo/apps/desktop/src/routes/+page.svelte")).toBe(true);
  });

  test("and nothing else in node_modules", () => {
    expect(included("/repo/node_modules/svelte/src/index.js")).toBe(false);
    expect(included("/repo/node_modules/some-ui-kit/dist/Button.svelte")).toBe(false);
  });
});

describe("the component looks", () => {
  test("c-entry-row is the clickable row, hover tint included", async () => {
    const css = await declarationsFor("c-entry-row");

    expect(css).toContain("grid-template-columns:1fr auto auto");
    // The button reset: a row that still inherits the app's typography.
    expect(css).toContain("font:inherit");
    expect(css).toContain("color-mix(in oklab, currentColor 8%, transparent)");
  });

  test("c-ring-progress transitions the dash offset and nothing else", async () => {
    const css = await declarationsFor("c-ring-progress");

    // The regression this test exists for: without an explicit
    // transition-property the ring animates `all` at one second.
    expect(css).toContain("transition-property:stroke-dashoffset");
    expect(css).toContain("transition-duration:1000ms");
    expect(css).toContain("stroke-width:2.5");
  });

  test("the tints are oklab mixes of the current text colour", async () => {
    const css = await declarationsFor("text-tint-60 border-tint-30 stroke-tint-20 bg-tint-8");

    for (const percent of [60, 30, 20, 8]) {
      expect(css).toContain(`color-mix(in oklab, currentColor ${percent}%, transparent)`);
    }
  });

  test("tokens resolve through CSS variables, so a face can re-theme a subtree", async () => {
    const css = await declarationsFor("c-action text-muted");

    expect(css).toContain("background-color:var(--accent)");
    expect(css).toContain("color:var(--muted)");
  });
});

describe("the page shell", () => {
  test("paints html and body for a face that owns its window", async () => {
    const uno = await createGenerator(unoPreset());
    const { css } = await uno.generate("");

    expect(css).toContain("--accent: #d4a24e");
    expect(css).toContain("background: var(--bg)");
  });

  test("but only the tokens where somebody else owns the surface", async () => {
    const uno = await createGenerator(unoPreset({ shell: false }));
    const { css } = await uno.generate("");

    // The popup and Storybook still need the variables the utilities
    // resolve against…
    expect(css).toContain("--accent: #d4a24e");
    // …and must not paint the page they were injected into.
    expect(css).not.toContain("background: var(--bg)");
  });
});
