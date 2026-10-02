/**
 * The component looks, asserted where they actually happen.
 *
 * `uno.ts` is a table of class names; a class name proves nothing. UnoCSS
 * treats an unmatched utility inside a shortcut as a *warning* and emits the
 * rest of the shortcut anyway — that is how `transition-[stroke-dashoffset]`
 * silently became "transition everything, for one second" during the
 * migration. So these tests mount the real components and read computed
 * styles back out of the browser: if a shortcut stops resolving, or the
 * foundation in `@castellan/utils/uno` stops being layered under it, the
 * declarations change and these fail.
 *
 * They are deliberately few. Each one pins a property that a face would
 * visibly regress on, not the whole appearance — pinning everything would
 * make every design tweak a test edit.
 */
import type { EntrySummary } from "@castellan/protocol";
import { expect, test } from "@playwright/experimental-ct-svelte";
import EntryRow from "../src/components/EntryRow.svelte";
import TotpRing from "../src/components/TotpRing.svelte";

const entry: EntrySummary = {
  id: "3f9d2c88-9a41-4b1d-9f6a-6c4f5b2a7e10",
  title: "GitHub",
  username: "ada@castellan.dev",
  url: "https://github.com",
  has_totp: true,
  has_passkey: true
};

/** One computed property of the first element matching `selector`. */
const styleOf = (locator: import("@playwright/test").Locator, property: string) =>
  locator.evaluate((element, name) => getComputedStyle(element).getPropertyValue(name), property);

test("c-entry-row lays the row out and keeps the button from looking like one", async ({
  mount
}) => {
  const row = await mount(EntryRow, { props: { entry } });

  // The grid is the contract: the title column grows, the badges do not.
  const columns = await styleOf(row, "grid-template-columns");
  expect(columns.split(" ")).toHaveLength(3);
  expect(await styleOf(row, "display")).toBe("grid");
  // The button reset — without it the row falls back to the UA's 13px Arial.
  expect(await styleOf(row, "font-family")).not.toContain("Arial");
  expect(await styleOf(row, "background-color")).toBe("rgba(0, 0, 0, 0)");
  expect(await styleOf(row, "border-top-width")).toBe("0px");
  expect(await styleOf(row, "cursor")).toBe("pointer");
});

test("c-badge is sized relative to the row it lands in", async ({ mount }) => {
  const row = await mount(EntryRow, { props: { entry } });
  const badge = row.getByTitle("TOTP");

  // 0.7em of the row's own size, not an absolute px value.
  const [rowSize, badgeSize] = await Promise.all([
    styleOf(row, "font-size"),
    styleOf(badge, "font-size")
  ]);
  expect(Number.parseFloat(badgeSize)).toBeCloseTo(Number.parseFloat(rowSize) * 0.7, 1);
  // A hairline mixed from the current text colour, not a hard-coded grey:
  // the browser resolves the oklab mix down to 30% alpha of whatever the
  // badge inherits.
  expect(await styleOf(badge, "border-top-color")).toMatch(/^oklab\(.*\/ 0\.3\)$/);
});

test("c-ring-progress transitions the dash offset and nothing else", async ({ mount }) => {
  const ring = await mount(TotpRing, { props: { code: "123456", secondsRemaining: 30 } });
  const progress = ring.locator(".progress");

  // The regression this test exists for.
  expect(await styleOf(progress, "transition-property")).toBe("stroke-dashoffset");
  expect(await styleOf(progress, "transition-duration")).toBe("1s");
  expect(await styleOf(progress, "stroke-width")).toBe("2.5px");
  // The track behind it is the same width, tinted off the current colour.
  expect(await styleOf(ring.locator(".track"), "stroke-width")).toBe("2.5px");
});
