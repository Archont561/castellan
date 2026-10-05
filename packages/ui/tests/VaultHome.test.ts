import type { EntrySummary } from "@castellan/protocol";
import { expect, test } from "@playwright/experimental-ct-svelte";
import VaultHome from "@/src/components/VaultHome.svelte";

const entry: EntrySummary = {
  id: "entry-1",
  title: "GitHub",
  username: "octocat",
  url: "https://github.com",
  has_totp: true,
  has_passkey: false
};

const layoutProps = {
  mainClass: "p-5",
  titleClass: "text-[1.3rem]",
  sectionTitleClass: "text-[0.85rem]",
  phraseClass: "px-3",
  actionClass: "p-3",
  emptyMessage: "No entries"
};

test("shows a loading state before the face client returns entries", async ({ mount }) => {
  let completeEntries: ((entries: EntrySummary[]) => void) | undefined;
  const home = await mount(VaultHome, {
    props: {
      ...layoutProps,
      client: {
        ping: async () => {},
        getEntries: () =>
          new Promise<EntrySummary[]>((resolve) => {
            completeEntries = resolve;
          }),
        getTotp: async () => ({ code: "135791", secondsRemaining: 30 }),
        generatePassphrase: async () => "amber-castle-river-lantern"
      }
    }
  });

  await expect(home.getByRole("status", { name: "Loading vault" })).toBeVisible();
  if (!completeEntries) throw new Error("entry loading did not start");
  completeEntries([entry]);
  await expect(home.getByRole("button", { name: /GitHub/ })).toBeVisible();
});

test("shows loaded entries, can reveal a selected TOTP code, and generates a passphrase", async ({
  mount
}) => {
  const home = await mount(VaultHome, {
    props: {
      ...layoutProps,
      client: {
        ping: async () => {},
        getEntries: async () => [entry],
        getTotp: async () => ({ code: "135791", secondsRemaining: 30 }),
        generatePassphrase: async () => "amber-castle-river-lantern"
      }
    }
  });

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
  const home = await mount(VaultHome, {
    props: {
      ...layoutProps,
      client: {
        ping: async () => {},
        getEntries: async () => [],
        getTotp: async () => ({ code: "135791", secondsRemaining: 30 }),
        generatePassphrase: async () => "amber-castle-river-lantern"
      }
    }
  });

  await expect(home.getByText("No entries", { exact: true })).toBeVisible();
});

test("announces that the vault client is unavailable", async ({ mount }) => {
  const home = await mount(VaultHome, {
    props: {
      ...layoutProps,
      client: {
        ping: async () => {
          throw new Error("the native vault is unavailable");
        },
        getEntries: async () => [entry],
        getTotp: async () => ({ code: "135791", secondsRemaining: 30 }),
        generatePassphrase: async () => "amber-castle-river-lantern"
      }
    }
  });

  await expect(home.getByRole("alert")).toContainText("the native vault is unavailable");
});
