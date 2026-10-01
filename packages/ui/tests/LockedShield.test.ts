import { expect, test } from "@playwright/experimental-ct-svelte";
import LockedShield from "../src/components/LockedShield.svelte";

// mount() resolves to the component's root element — here the svg itself,
// so the state is asserted on the locator directly (role locators only
// match descendants, they cannot match the root they are called on).

test("draws the locked state", async ({ mount }) => {
  const icon = await mount(LockedShield, { props: { locked: true } });

  await expect(icon).toBeVisible();
  await expect(icon).toHaveAttribute("aria-label", "vault locked");
  await expect(icon).toHaveAttribute("role", "img");
});

test("draws the unlocked state", async ({ mount }) => {
  const icon = await mount(LockedShield, { props: { locked: false } });

  await expect(icon).toBeVisible();
  await expect(icon).toHaveAttribute("aria-label", "vault unlocked");
});
