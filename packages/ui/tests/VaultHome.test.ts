import { expect, test } from "@playwright/experimental-ct-svelte";

import VaultHomeHarness from "@/tests/fixtures/VaultHomeHarness.svelte";

test("loads entries and generates a passphrase through the supplied face client", async ({
  mount
}) => {
  const home = await mount(VaultHomeHarness);

  await expect(home.getByText("GitHub", { exact: true })).toBeVisible();
  await expect(home.getByText("octocat", { exact: true })).toBeVisible();

  await home.getByRole("button", { name: "Generate" }).click();
  await expect(home.locator(".phrase")).toHaveText("amber-castle-river-lantern");
});

test("picking a 2FA entry shows its live code from the client, and picking again hides it", async ({
  mount
}) => {
  const home = await mount(VaultHomeHarness);

  const row = home.locator(".row", { hasText: "GitHub" });
  await row.click();
  // The code and ring come straight from the client's protocol-shaped
  // answer — the harness returns them, the UI only draws.
  await expect(home.locator(".totp-code code")).toHaveText("135791");
  await expect(home.locator(".totp-code svg circle.progress")).toBeVisible();

  await row.click();
  await expect(home.locator(".totp-code")).toHaveCount(0);
});
