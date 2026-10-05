import { expect, test } from "@playwright/experimental-ct-svelte";
import TotpRing from "@/src/components/TotpRing.svelte";

test("renders the supplied code", async ({ mount }) => {
  const ring = await mount(TotpRing, {
    props: { code: "123456", secondsRemaining: 30 }
  });

  await expect(ring.getByText("123456", { exact: true })).toBeVisible();
});

test("notifies its owner once when the supplied countdown expires", async ({ mount, page }) => {
  await page.clock.install({ time: new Date("2026-10-05T12:00:00Z") });
  let expirations = 0;
  await mount(TotpRing, {
    props: { code: "654321", secondsRemaining: 1, onExpired: () => expirations++ }
  });

  await page.clock.fastForward(1000);
  expect(expirations).toBe(1);
  await page.clock.fastForward(5000);
  expect(expirations).toBe(1);
});
