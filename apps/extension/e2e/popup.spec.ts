import { type BrowserContext, test as base, chromium, expect, type Page } from "@playwright/test";

// The built extension as WXT emits it for chromium (the e2e task depends
// on `build`, so it exists by the time the specs run). Resolved from this
// file's location so the suite works regardless of the runner's cwd.
const extensionDir = new URL("../.output/chrome-mv3", import.meta.url).pathname;

// MV3 extensions live in a persistent context: the service worker (the
// background) registers per profile, and the extension ID is only knowable
// from the worker's URL — so the fixture waits for it before handing over
// a page pointed at the popup.
const test = base.extend<{ ctx: BrowserContext; page: Page }>({
  // biome-ignore lint/correctness/noEmptyPattern: playwright's fixture signature must destructure the fixtures object, and this one uses none.
  ctx: async ({}, use) => {
    const ctx = await chromium.launchPersistentContext("", {
      channel: "chromium",
      args: [`--disable-extensions-except=${extensionDir}`, `--load-extension=${extensionDir}`]
    });
    await use(ctx);
    await ctx.close();
  },
  page: async ({ ctx }, use) => {
    let [worker] = ctx.serviceWorkers();
    if (!worker) worker = await ctx.waitForEvent("serviceworker");
    const extensionId = new URL(worker.url()).host;
    const page = await ctx.newPage();
    await page.goto(`chrome-extension://${extensionId}/popup.html`);
    await use(page);
  }
});

test("the background service worker registers", async ({ ctx }) => {
  let [worker] = ctx.serviceWorkers();
  if (!worker) worker = await ctx.waitForEvent("serviceworker");

  // The manifest loaded and WXT's background entry compiled — every other
  // guarantee (native messaging, message routing) hangs off this one.
  expect(worker.url()).toContain("background");
});

test("popup reports the absent native host honestly", async ({ page }) => {
  await expect(page.getByRole("heading", { name: "Fob" })).toBeVisible();

  // No Castellan app is running in the test environment: connectNative
  // fails, the background answers { ok: false }, and the popup says so
  // instead of spinning on "Looking for Castellan…".
  await expect(page.locator(".status")).toHaveText("Castellan is not running");
  await expect(page.locator(".muted")).toContainText(/protocol v\d+/);
});
