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
  ],
  manifest_problems: [
    {
      browser: "edge",
      kind: "stale_path",
      detail: "points at /old/place/castellan, the app now lives at /opt/castellan"
    }
  ]
};

const noops = { onConfirm: () => {}, onDeny: () => {}, onKill: () => {} };

test("shows each browser enrollment waiting for a decision", async ({ mount }) => {
  const panel = await mount(BrowsersPanel, { props: { snapshot, ...noops } });
  const approvals = panel.getByRole("region", { name: "Waiting for approval" });

  await expect(approvals.getByRole("listitem")).toHaveText("Firefox on this machineAllowDeny");
  await expect(approvals.getByRole("button", { name: "Allow" })).toBeVisible();
  await expect(approvals.getByRole("button", { name: "Deny" })).toBeVisible();
});

test("reports connected browser state without hiding unavailable metadata", async ({ mount }) => {
  const panel = await mount(BrowsersPanel, { props: { snapshot, ...noops } });
  const connections = panel.getByRole("list", { name: "Active connections" });

  await expect(connections.getByRole("listitem").filter({ hasText: "Chrome" })).toContainText(
    "Chrome1.2.0readyget_entries"
  );
  await expect(connections.getByRole("button", { name: "Disconnect Chrome" })).toBeVisible();
  await expect(connections.getByRole("listitem").filter({ hasText: "Firefox" })).toContainText(
    "Firefoxunknown versionwaiting—"
  );
});

test("shows remembered browsers and repairable manifest problems", async ({ mount }) => {
  const panel = await mount(BrowsersPanel, { props: { snapshot, ...noops } });

  await expect(panel.getByRole("list", { name: "Remembered browsers" })).toContainText(
    "Chrome on this machinesince 2025-10-09"
  );
  await expect(panel.getByRole("list", { name: "Manifest problems" })).toContainText(
    "EdgeManifest points at an old app path"
  );
});

test("keyboard decisions and disconnect actions reach the face callbacks", async ({ mount }) => {
  const allowed: string[] = [];
  const denied: string[] = [];
  const disconnected: number[] = [];
  const panel = await mount(BrowsersPanel, {
    props: {
      snapshot,
      onConfirm: (keyId) => allowed.push(keyId),
      onDeny: (keyId) => denied.push(keyId),
      onKill: (connectionId) => disconnected.push(connectionId)
    }
  });

  await panel.getByRole("button", { name: "Allow" }).press("Enter");
  await panel.getByRole("button", { name: "Deny" }).press("Space");
  await panel.getByRole("button", { name: "Disconnect Chrome" }).click();

  expect(allowed).toEqual(["feedfacefeedface"]);
  expect(denied).toEqual(["feedfacefeedface"]);
  expect(disconnected).toEqual([1]);
});

test("names the quiet connected-browser state", async ({ mount }) => {
  const panel = await mount(BrowsersPanel, {
    props: {
      snapshot: { connections: [], pending: [], remembered: [], manifest_problems: [] },
      ...noops
    }
  });

  await expect(panel.getByText("No browsers connected.")).toBeVisible();
  await expect(panel.getByRole("region", { name: "Waiting for approval" })).toHaveCount(0);
  await expect(panel.getByRole("list", { name: "Remembered browsers" })).toHaveCount(0);
});
