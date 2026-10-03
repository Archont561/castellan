<script module lang="ts">
import type { PanelSnapshot } from "@castellan/protocol";
import { defineMeta } from "@storybook/addon-svelte-csf";
import BrowsersPanel from "./BrowsersPanel.svelte";

const busy: PanelSnapshot = {
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
      client_version: "1.2.0",
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

const quiet: PanelSnapshot = { connections: [], pending: [], remembered: [] };

const noops = {
  onConfirm: (_keyId: string) => {},
  onDeny: (_keyId: string) => {},
  onKill: (_connectionId: number) => {}
};

const { Story } = defineMeta({
  title: "Components/BrowsersPanel",
  component: BrowsersPanel
});
</script>

<!--
  The panel's interesting states: everything at once (a prompt to answer,
  two connections — one ready, one still waiting — and a remembered key),
  and the quiet day (nothing connected, nothing remembered, no prompts).
-->
<Story name="Busy" args={{ snapshot: busy, ...noops }} />
<Story name="Quiet" args={{ snapshot: quiet, ...noops }} />
