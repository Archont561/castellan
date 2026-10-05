import { expect, test } from "@playwright/test";

// The desktop face is a Tauri app; e2e runs the SvelteKit layer in a plain
// browser, where `invoke` cannot exist. That is not a stubbed environment —
// it is the exact state a user's webview would be in if the Rust side
// crashed, and the contract is: the shell renders, and the missing backend
// is reported in the UI, never hidden behind a spinner.
test("renders the shell and reports the missing Tauri backend", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { name: "Castellan", level: 1 })).toBeVisible();
  await expect(page.getByRole("img", { name: "vault locked" })).toBeVisible();
  // No `__TAURI_INTERNALS__` in a plain browser: the transport reports
  // disconnection and the page says so instead of pretending.
  await expect(page.locator("main .error")).toBeVisible();
});

// The import surface (task-13) is part of the desktop shell: the paste
// box, the file/QR entry points and — desktop-only — the screen-capture
// scan all render without a backend; only a Preview round-trip needs one.
test("renders the authenticator import surface", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByRole("heading", { name: "Import authenticator codes" })).toBeVisible();
  await expect(page.getByLabel("Import payload")).toBeVisible();
  await expect(page.getByText("Import a file or QR image")).toBeVisible();
  await expect(page.getByRole("button", { name: "Scan a QR on screen" })).toBeVisible();
  // Nothing to preview yet: the affirmative action waits for input.
  await expect(page.getByRole("button", { name: "Preview" })).toBeDisabled();
});
