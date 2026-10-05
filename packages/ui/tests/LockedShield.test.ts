import { expect, test } from "@playwright/experimental-ct-svelte";
import LockedShield from "@/src/components/LockedShield.svelte";

test("announces that the vault is locked", async ({ mount }) => {
  const icon = await mount(LockedShield, { props: { locked: true } });

  await expect(icon).toBeVisible();
  await expect(icon).toHaveRole("img");
  await expect(icon).toHaveAccessibleName("vault locked");
});

test("announces that the vault is unlocked", async ({ mount }) => {
  const icon = await mount(LockedShield, { props: { locked: false } });

  await expect(icon).toBeVisible();
  await expect(icon).toHaveAccessibleName("vault unlocked");
});
