import { expect, test } from "@playwright/experimental-ct-svelte";
import TotpCodeHarness from "@/tests/fixtures/TotpCodeHarness.svelte";

test("refreshes the code when its displayed countdown expires", async ({ mount, page }) => {
  await page.clock.install({ time: new Date("2026-10-05T12:00:00Z") });
  const totp = await mount(TotpCodeHarness);

  await expect(totp.getByText("111111", { exact: true })).toBeVisible();
  await page.clock.fastForward(1000);
  await expect(totp.getByText("222222", { exact: true })).toBeVisible();
});

test("announces that a code is loading until the face client responds", async ({ mount }) => {
  const totp = await mount(TotpCodeHarness, { props: { scenario: "loading" } });

  await expect(totp.getByRole("status", { name: "Loading authenticator code" })).toBeVisible();
  await totp.getByRole("button", { name: "Finish code loading" }).click();
  await expect(totp.getByText("123456", { exact: true })).toBeVisible();
});

test("announces a failed code refresh", async ({ mount }) => {
  const totp = await mount(TotpCodeHarness, { props: { scenario: "rejected" } });

  await expect(totp.getByRole("alert")).toContainText("the vault is locked");
});
