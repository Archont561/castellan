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

// mount() resolves to the component's root element — for EntryRow that is
// the row button itself, so the locator is asserted and clicked directly;
// only the parts inside it (username, badges) are queried as descendants.

test("shows the title, username and capability badges", async ({ mount }) => {
  const row = await mount(EntryRow, { props: { entry } });

  await expect(row).toContainText("GitHub");
  await expect(row.locator(".username")).toHaveText("ada@castellan.dev");
  await expect(row.getByTitle("TOTP")).toHaveText("2FA");
  await expect(row.getByTitle("Passkey")).toBeVisible();
});

test("omits what the entry does not have", async ({ mount }) => {
  const bare: EntrySummary = {
    ...entry,
    username: null,
    has_totp: false,
    has_passkey: false
  };
  const row = await mount(EntryRow, { props: { entry: bare } });

  await expect(row).toHaveText("GitHub");
  await expect(row.locator(".username")).toHaveCount(0);
  await expect(row.locator(".badge")).toHaveCount(0);
});

test("reports picks through the callback", async ({ mount }) => {
  const picks: EntrySummary[] = [];
  const row = await mount(EntryRow, {
    props: { entry, onPick: (picked) => picks.push(picked) }
  });

  await row.click();

  expect(picks.map((picked) => picked.id)).toEqual([entry.id]);
});
