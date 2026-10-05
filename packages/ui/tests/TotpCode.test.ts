import { expect, test } from "@playwright/experimental-ct-svelte";
import TotpCode from "@/src/components/TotpCode.svelte";

test("refreshes the code when its displayed countdown expires", async ({ mount, page }) => {
  await page.clock.install({ time: new Date("2026-10-05T12:00:00Z") });
  const answers = [
    { code: "111111", secondsRemaining: 1 },
    { code: "222222", secondsRemaining: 30 }
  ];
  let calls = 0;
  const totp = await mount(TotpCode, {
    props: {
      getCode: async () =>
        answers[Math.min(calls++, answers.length - 1)] as (typeof answers)[number]
    }
  });

  await expect(totp.getByText("111111", { exact: true })).toBeVisible();
  await page.clock.fastForward(1000);
  await expect(totp.getByText("222222", { exact: true })).toBeVisible();
});

test("announces that a code is loading until the face client responds", async ({ mount }) => {
  let complete: ((answer: { code: string; secondsRemaining: number }) => void) | undefined;
  const totp = await mount(TotpCode, {
    props: {
      getCode: () =>
        new Promise((resolve) => {
          complete = resolve;
        })
    }
  });

  await expect(totp.getByRole("status", { name: "Loading authenticator code" })).toBeVisible();
  if (!complete) throw new Error("code loading did not start");
  complete({ code: "123456", secondsRemaining: 30 });
  await expect(totp.getByText("123456", { exact: true })).toBeVisible();
});

test("announces a failed code refresh", async ({ mount }) => {
  const totp = await mount(TotpCode, {
    props: {
      getCode: async () => {
        throw new Error("the vault is locked");
      }
    }
  });

  await expect(totp.getByRole("alert")).toContainText("the vault is locked");
});
