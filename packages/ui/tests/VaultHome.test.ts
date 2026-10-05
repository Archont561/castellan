import { expect, test } from "@playwright/experimental-ct-svelte";
import VaultHomeHarness from "@/tests/fixtures/VaultHomeHarness.svelte";

test("shows a loading state before the face client returns entries", async ({ mount }) => {
  const home = await mount(VaultHomeHarness, { props: { scenario: "loading" } });

  await expect(home.getByRole("status", { name: "Loading vault" })).toBeVisible();
  await home.getByRole("button", { name: "Finish entry loading" }).click();
  await expect(home.getByRole("button", { name: /GitHub/ })).toBeVisible();
});

test("shows loaded entries, can reveal a selected TOTP code, and generates a passphrase", async ({
  mount
}) => {
  const home = await mount(VaultHomeHarness);

  const github = home.getByRole("button", { name: /GitHub/ });
  await expect(github).toBeVisible();
  await github.click();
  await expect(home.getByText("135791", { exact: true })).toBeVisible();
  await github.click();
  await expect(home.getByText("135791", { exact: true })).toHaveCount(0);

  await home.getByRole("button", { name: "Generate" }).press("Enter");
  await expect(home.getByText("amber-castle-river-lantern", { exact: true })).toBeVisible();
});

test("names an empty vault after a successful response", async ({ mount }) => {
  const home = await mount(VaultHomeHarness, { props: { scenario: "empty" } });

  await expect(home.getByText("No entries", { exact: true })).toBeVisible();
});

test("announces that the vault client is unavailable", async ({ mount }) => {
  const home = await mount(VaultHomeHarness, { props: { scenario: "unavailable" } });

  await expect(home.getByRole("alert")).toContainText("the native vault is unavailable");
});
