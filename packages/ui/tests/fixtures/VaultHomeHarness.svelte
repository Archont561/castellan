<script lang="ts">
import type { EntrySummary } from "@castellan/protocol";

import VaultHome from "@/src/components/VaultHome.svelte";

const entry: EntrySummary = {
  id: "entry-1",
  title: "GitHub",
  username: "octocat",
  url: "https://github.com",
  has_totp: true,
  has_passkey: false
};

const client = {
  async ping(): Promise<void> {},
  async getEntries(): Promise<EntrySummary[]> {
    return [entry];
  },
  async generatePassphrase(): Promise<string> {
    return "amber-castle-river-lantern";
  }
};
</script>

<VaultHome
  {client}
  mainClass="p-5"
  titleClass="text-[1.3rem]"
  sectionTitleClass="text-[0.85rem]"
  phraseClass="px-3"
  actionClass="p-3"
  emptyMessage="No entries"
/>
