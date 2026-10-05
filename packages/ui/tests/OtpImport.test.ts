import { expect, test } from "@playwright/experimental-ct-svelte";
import OtpImportHarness from "@/tests/fixtures/OtpImportHarness.svelte";

// The review contract: every account the payload carried is listed —
// refused ones named with their reason — and import sends exactly what
// stayed checked. The harness scripts the protocol pair and records
// the selection into the DOM.
test("previews a batch, reviews it, and imports only what stayed checked", async ({ mount }) => {
  const panel = await mount(OtpImportHarness);

  await panel.getByLabel("Import payload").fill("otpauth://totp/whatever");
  await panel.getByRole("button", { name: "Preview" }).click();

  // All three accounts are visible, including the refused one.
  await expect(panel.locator(".candidate")).toHaveCount(3);
  await expect(panel.locator(".problem")).toContainText("HOTP");

  // Uncheck one importable account; the button's count follows.
  await expect(panel.getByRole("button", { name: "Import 2 accounts" })).toBeVisible();
  await panel.getByLabel("Import Example").uncheck();
  await panel.getByRole("button", { name: "Import 1 account" }).click();

  await expect(panel.locator(".summary")).toHaveText("Imported 1 account.");
  // Exactly the checked account crossed the callback, titled by issuer
  // with the account as username.
  await expect(panel.locator(".imported-record li")).toHaveCount(1);
  await expect(panel.locator(".imported-record li")).toHaveText("GitHub:octocat");
});

test("an image file goes through the face's QR decoder into preview", async ({ mount }) => {
  const panel = await mount(OtpImportHarness);

  await panel.locator("input[type=file]").setInputFiles({
    name: "qr.png",
    mimeType: "image/png",
    buffer: Buffer.from("not a real png; the harness decoder is scripted")
  });

  // The decoded text landed in the payload box and previewed.
  await expect(panel.getByLabel("Import payload")).toHaveValue("decoded-from-qr");
  await expect(panel.locator(".candidate")).toHaveCount(3);
});

test("a preview failure is shown, not swallowed", async ({ mount }) => {
  const panel = await mount(OtpImportHarness);

  await panel.getByLabel("Import payload").fill("garbage");
  await panel.getByRole("button", { name: "Preview" }).click();

  await expect(panel.locator(".import-error")).toContainText("not a recognized import");
  await expect(panel.locator(".candidate")).toHaveCount(0);
});
