import { expect, test } from "@playwright/test";

// The desktop face is a Tauri app; e2e runs its SvelteKit layer in a plain
// browser. If the native side is absent, users must see a usable shell and an
// announced failure rather than a permanent loading state.
test("shows the shell and announces an unavailable native vault", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { name: "Castellan", level: 1 })).toBeVisible();
  await expect(page.getByRole("img", { name: "vault locked" })).toBeVisible();
  await expect(page.getByRole("alert")).toBeVisible();
});

// The import surface belongs to the desktop composition. Parsing needs the
// native client, but the user can still discover every input and the empty
// payload cannot accidentally start an import.
test("offers the desktop import paths without enabling an empty preview", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { name: "Import authenticator codes" })).toBeVisible();
  await expect(page.getByLabel("Import payload")).toBeVisible();
  await expect(page.getByLabel("Import a file or QR image")).toBeVisible();
  await expect(page.getByRole("button", { name: "Scan a QR on screen" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Preview" })).toBeDisabled();
});
