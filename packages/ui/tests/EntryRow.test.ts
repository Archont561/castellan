import type { EntrySummary } from "@castellan/protocol";
import { expect, test } from "@playwright/experimental-ct-svelte";
import EntryRow from "@/src/components/EntryRow.svelte";

const entry: EntrySummary = {
  id: "3f9d2c88-9a41-4b1d-9f6a-6c4f5b2a7e10",
  title: "GitHub",
  username: "ada@castellan.dev",
  url: "https://github.com",
  has_totp: true,
  has_passkey: true
};

test("shows an entry's identity and capabilities", async ({ mount }) => {
  const row = await mount(EntryRow, { props: { entry } });

  await expect(row).toHaveRole("button");
  await expect(row).toHaveAccessibleName(/GitHub/);
  await expect(row).toContainText("ada@castellan.dev");
  await expect(row.getByText("2FA", { exact: true })).toBeVisible();
  await expect(row.getByRole("img", { name: "Passkey" })).toBeVisible();
});

test("does not invent absent account details or capabilities", async ({ mount }) => {
  const bare: EntrySummary = {
    ...entry,
    username: null,
    has_totp: false,
    has_passkey: false
  };
  const row = await mount(EntryRow, { props: { entry: bare } });

  await expect(row).toHaveAccessibleName("GitHub");
  await expect(row.getByText("2FA", { exact: true })).toHaveCount(0);
  await expect(row.getByRole("img", { name: "Passkey" })).toHaveCount(0);
});

test("keyboard activation selects the entry through its public callback", async ({ mount }) => {
  const picked: EntrySummary[] = [];
  const row = await mount(EntryRow, {
    props: { entry, onPick: (selected) => picked.push(selected) }
  });

  await row.focus();
  await row.press("Enter");

  expect(picked).toEqual([entry]);
});
