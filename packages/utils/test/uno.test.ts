/**
 * The shared UnoCSS foundation, proven without a browser.
 *
 * Two things here are load-bearing and silently breakable. The first is the
 * extraction pipeline: @castellan/ui is consumed as source, so if the
 * include pattern stops matching its files, every shared component still
 * compiles, still renders, still passes its component tests inside the
 * library's own harness — and arrives in the apps with class attributes and
 * no CSS. The second is the seam this file opens for `@castellan/ui/uno`:
 * the component shortcuts are layered on from there, and they expand into
 * utilities only this foundation defines, so "the extra preset is applied"
 * and "the foundation is still under it" are both asserted below.
 *
 * The looks themselves are not asserted here — they are not defined here.
 * `packages/ui/tests/styling.test.ts` pins them in a real browser, which is
 * the only place an unmatched utility inside a shortcut (a warning UnoCSS
 * carries on from) actually shows up.
 */
import { describe, expect, test } from "bun:test";
import type { Preset } from "unocss";
import { createGenerator } from "unocss";

import { unoPreset } from "@/src/uno";

/** The declarations UnoCSS generates for a space-separated class list. */
async function declarationsFor(
  classes: string,
  shell = true,
  presets: Preset[] = []
): Promise<string> {
  const uno = await createGenerator(unoPreset({ shell, presets }));
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

describe("the foundation", () => {
  test("the tints are oklab mixes of the current text colour", async () => {
    const css = await declarationsFor("text-tint-60 border-tint-30 stroke-tint-20 bg-tint-8");

    for (const percent of [60, 30, 20, 8]) {
      expect(css).toContain(`color-mix(in oklab, currentColor ${percent}%, transparent)`);
    }
  });

  test("tokens resolve through CSS variables, so a face can re-theme a subtree", async () => {
    const css = await declarationsFor("bg-accent text-muted");

    expect(css).toContain("background-color:var(--accent)");
    expect(css).toContain("color:var(--muted)");
  });

  test("carries no component shortcuts of its own", async () => {
    // They live in `@castellan/ui/uno`, with the components they describe,
    // because the extension popup depends on this package and not on the
    // component library. A shortcut that reappears here is a design system
    // the popup pays for and cannot use.
    expect(await declarationsFor("c-entry-row c-action c-ring-progress")).toBe("");
  });

  test("layers a consumer's presets on top, with the foundation under them", async () => {
    // How `presetUi()` arrives. The shortcut is resolved by the extra
    // preset; the tint it expands to is resolved by the foundation — which
    // is the half of the contract that would break silently.
    const css = await declarationsFor("c-probe", true, [
      { name: "probe", shortcuts: { "c-probe": "bg-tint-8 text-accent" } }
    ]);

    expect(css).toContain("color-mix(in oklab, currentColor 8%, transparent)");
    expect(css).toContain("color:var(--accent)");
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
