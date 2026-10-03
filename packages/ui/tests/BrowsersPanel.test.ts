import type { PanelSnapshot } from "@castellan/protocol";
import { expect, test } from "@playwright/experimental-ct-svelte";
import BrowsersPanel from "@/src/components/BrowsersPanel.svelte";

const snapshot: PanelSnapshot = {
  connections: [
    {
      id: 1,
      face: "chrome",
      client_version: "1.2.0",
      protocol_version: 3,
      state: "ready",
      last_request: "get_entries",
      connected_at: 1_770_000_000
    },
    {
      id: 2,
      face: "firefox",
      client_version: null,
      protocol_version: 3,
      state: "awaiting_approval",
      last_request: null,
      connected_at: 1_770_000_030
    }
  ],
  pending: [{ key_id: "feedfacefeedface", label: "Firefox on this machine" }],
  remembered: [
    { key_id: "feedfacefeedface", label: "Chrome on this machine", added_at: 1_760_000_000 }
  ]
};

// mount() resolves to the component's root element — the panel section.
// Descendants are queried through it; the bare classes (.prompt,
// .connection, .remembered) are the stable hooks.

test("shows a prompt for each enrollment waiting on the user", async ({ mount }) => {
  const panel = await mount(BrowsersPanel, {
    props: { snapshot, onConfirm: () => {}, onDeny: () => {}, onKill: () => {} }
  });

  await expect(panel.locator(".prompt .label")).toHaveText("Firefox on this machine");
  await expect(panel.getByRole("button", { name: "Allow" })).toBeVisible();
  await expect(panel.getByRole("button", { name: "Deny" })).toBeVisible();
});

test("shows face, version, last request and a kill switch per connection", async ({ mount }) => {
  const panel = await mount(BrowsersPanel, {
    props: { snapshot, onConfirm: () => {}, onDeny: () => {}, onKill: () => {} }
  });

  const rows = panel.locator(".connection");
  await expect(rows).toHaveCount(2);

  const chrome = rows.nth(0);
  await expect(chrome.locator(".face")).toHaveText("Chrome");
  await expect(chrome.locator(".version")).toHaveText("1.2.0");
  await expect(chrome.locator(".last-request")).toHaveText("get_entries");
  await expect(chrome.locator(".state")).toHaveText("ready");
  await expect(chrome.getByRole("button", { name: "Disconnect Chrome" })).toBeVisible();

  // A connection still handshaking shows what it has and no more.
  const firefox = rows.nth(1);
  await expect(firefox.locator(".face")).toHaveText("Firefox");
  await expect(firefox.locator(".version")).toHaveText("unknown version");
  await expect(firefox.locator(".last-request")).toHaveText("—");
  await expect(firefox.locator(".state")).toHaveText("waiting");
});

test("lists the remembered keys", async ({ mount }) => {
  const panel = await mount(BrowsersPanel, {
    props: { snapshot, onConfirm: () => {}, onDeny: () => {}, onKill: () => {} }
  });

  await expect(panel.locator(".remembered .label")).toHaveText("Chrome on this machine");
  await expect(panel.locator(".remembered .since")).toHaveText("since 2025-10-09");
});

test("reports allow, deny and kill through the callbacks", async ({ mount }) => {
  const allowed: string[] = [];
  const denied: string[] = [];
  const killed: number[] = [];
  const panel = await mount(BrowsersPanel, {
    props: {
      snapshot,
      onConfirm: (keyId) => allowed.push(keyId),
      onDeny: (keyId) => denied.push(keyId),
      onKill: (connectionId) => killed.push(connectionId)
    }
  });

  await panel.getByRole("button", { name: "Allow" }).click();
  await panel.getByRole("button", { name: "Deny" }).click();
  await panel.getByRole("button", { name: "Disconnect Chrome" }).click();

  expect(allowed).toEqual(["feedfacefeedface"]);
  expect(denied).toEqual(["feedfacefeedface"]);
  expect(killed).toEqual([1]);
});

test("a quiet panel says so without lying about sections", async ({ mount }) => {
  const quiet: PanelSnapshot = { connections: [], pending: [], remembered: [] };
  const panel = await mount(BrowsersPanel, {
    props: { snapshot: quiet, onConfirm: () => {}, onDeny: () => {}, onKill: () => {} }
  });

  await expect(panel.locator(".empty")).toHaveText("No browsers connected.");
  await expect(panel.locator(".prompt")).toHaveCount(0);
  await expect(panel.locator(".remembered")).toHaveCount(0);
});
