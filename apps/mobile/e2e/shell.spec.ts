import { expect, test } from "@playwright/test";

// The phone is the primary constraint for this face. The fixture starts at a
// representative phone viewport and verifies the composed native-unavailable
// state rather than duplicating component-level vault behavior.
test.use({ viewport: { width: 390, height: 844 } });

test("shows the mobile shell and announces an unavailable native vault", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { name: "Castellan", level: 1 })).toBeVisible();
  await expect(page.getByRole("img", { name: "vault locked" })).toBeVisible();
  await expect(page.getByRole("alert")).toBeVisible();
});
