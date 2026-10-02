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
