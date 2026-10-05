import { expect, test } from "@playwright/experimental-ct-svelte";
import OtpImportHarness from "@/tests/fixtures/OtpImportHarness.svelte";

test("lets a user review a batch and import only the accepted accounts", async ({ mount }) => {
  const panel = await mount(OtpImportHarness);

  await panel.getByLabel("Import payload").fill("otpauth://totp/anything");
  await panel.getByRole("button", { name: "Preview", exact: true }).click();

  const review = panel.getByRole("list", { name: "Accounts to import" });
  await expect(review.getByRole("listitem")).toHaveCount(3);
  await expect(review).toContainText("HOTP accounts need counter support");
  await panel.getByRole("checkbox", { name: "Import Example" }).uncheck();
  await panel.getByRole("button", { name: "Import 1 account" }).click();

  await expect(panel.getByText("Imported 1 account.", { exact: true })).toBeVisible();
  await expect(panel.getByRole("list", { name: "Imported accounts" })).toHaveText("GitHub:octocat");
  await expect(panel.getByLabel("Import requests")).toHaveText("1");
});

test("previews a decoded QR image through the face decoder", async ({ mount }) => {
  const panel = await mount(OtpImportHarness);

  await panel.getByLabel("Import a file or QR image").setInputFiles({
    name: "qr.png",
    mimeType: "image/png",
    buffer: Buffer.from("synthetic QR fixture")
  });

  await expect(panel.getByLabel("Import payload")).toHaveValue("decoded-from-qr");
  await expect(panel.getByRole("list", { name: "Accounts to import" })).toBeVisible();
});

test("announces an invalid import payload", async ({ mount }) => {
  const panel = await mount(OtpImportHarness);

  await panel.getByLabel("Import payload").fill("garbage");
  await panel.getByRole("button", { name: "Preview", exact: true }).click();

  await expect(panel.getByRole("alert")).toContainText("not a recognized import");
  await expect(panel.getByRole("list", { name: "Accounts to import" })).toHaveCount(0);
});

test("keeps the most recent preview when requests resolve out of order", async ({ mount }) => {
  const panel = await mount(OtpImportHarness, { props: { scenario: "race" } });

  await panel.getByLabel("Import payload").fill("first");
  await panel.getByRole("button", { name: "Preview", exact: true }).click();
  await panel.getByLabel("Import payload").fill("second");
  await panel.getByRole("button", { name: "Preview", exact: true }).click();

  const review = panel.getByRole("list", { name: "Accounts to import" });
  await expect(review).toContainText("Current");
  await panel.getByRole("button", { name: "Release stale preview" }).click();
  await expect(review).toContainText("Current");
  await expect(panel.getByText("Stale", { exact: true })).toHaveCount(0);
});

test("prevents a second import while the first is still pending", async ({ mount }) => {
  const panel = await mount(OtpImportHarness, { props: { scenario: "pending-import" } });

  await panel.getByLabel("Import payload").fill("one account");
  await panel.getByRole("button", { name: "Preview", exact: true }).click();
  await panel.getByRole("button", { name: "Import 1 account" }).click();

  await expect(panel.getByRole("region", { name: "Import authenticator codes" })).toHaveAttribute(
    "aria-busy",
    "true"
  );
  await expect(panel.getByRole("button", { name: "Importing…" })).toBeDisabled();
  await panel.getByRole("button", { name: "Finish import" }).click();
  await expect(panel.getByText("Imported 1 account.", { exact: true })).toBeVisible();
  await expect(panel.getByLabel("Import requests")).toHaveText("1");
});
