import { expect, test } from "@playwright/test";

// The phone is the primary constraint for this face, so the test runs at
// a phone viewport from the start. Everything else matches the desktop
// contract: the shell renders, and the absent Tauri backend is reported
// honestly (no `__TAURI_INTERNALS__` in a plain browser).
test.use({ viewport: { width: 390, height: 844 } });

test("renders the shell and reports the missing Tauri backend", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { name: "Castellan", level: 1 })).toBeVisible();
  await expect(page.getByRole("img", { name: "vault locked" })).toBeVisible();
  await expect(page.locator("main .error")).toBeVisible();
});
