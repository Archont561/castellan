import { expect, test } from "@playwright/experimental-ct-svelte";
import TotpRing from "../src/components/TotpRing.svelte";

// The ring ticks on its own between protocol answers, but the contract
// worth pinning is what it draws *for a given answer*: the code, and a
// progress arc sized by the remaining seconds. Timing-dependent behavior
// (the 1s tick) is deliberately not asserted — that way lies flakes.
test("shows the code and the countdown arc", async ({ mount }) => {
  const ring = await mount(TotpRing, {
    props: { code: "123456", secondsRemaining: 30 }
  });

  await expect(ring.locator("code")).toHaveText("123456");
  await expect(ring.locator("svg circle.progress")).toBeVisible();
});

test("handles a code that just rotated in", async ({ mount }) => {
  const ring = await mount(TotpRing, {
    props: { code: "654321", secondsRemaining: 1 }
  });

  await expect(ring.locator("code")).toHaveText("654321");
});
