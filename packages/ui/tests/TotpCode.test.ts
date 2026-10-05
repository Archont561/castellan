import { expect, test } from "@playwright/experimental-ct-svelte";
import TotpCodeFailingHarness from "@/tests/fixtures/TotpCodeFailingHarness.svelte";
import TotpCodeHarness from "@/tests/fixtures/TotpCodeHarness.svelte";

// The task-13 contract: the code and its remaining seconds come from
// the protocol answer, the component only draws and — when the ring
// runs out — asks again. The harness scripts a 1-second first answer,
// so one real tick proves the whole refetch loop without flaking.
test("shows the fetched code, then refetches when it expires", async ({ mount }) => {
  const totp = await mount(TotpCodeHarness);

  await expect(totp.locator("code")).toHaveText("111111");
  // After the 1s window closes the component must come back with the
  // scripted second answer — no local period guessing could produce it.
  await expect(totp.locator("code")).toHaveText("222222", { timeout: 5000 });
});

test("a failed fetch surfaces as text, not a blank", async ({ mount }) => {
  const totp = await mount(TotpCodeFailingHarness);

  // The error paragraph is the component's entire render here, so the
  // mount locator itself is the element under assertion.
  await expect(totp).toContainText("the vault is locked");
  await expect(totp).toHaveClass(/totp-error/);
});
