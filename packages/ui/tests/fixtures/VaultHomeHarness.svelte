<script lang="ts">
import type { EntrySummary } from "@castellan/protocol";

import VaultHome from "@/src/components/VaultHome.svelte";

type Scenario = "loaded" | "loading" | "empty" | "unavailable";

interface Props {
  scenario?: Scenario;
}

let { scenario = "loaded" }: Props = $props();

const entry: EntrySummary = {
  id: "entry-1",
  title: "GitHub",
  username: "octocat",
  url: "https://github.com",
  has_totp: true,
  has_passkey: false
};

let finishEntryLoading = $state<(() => void) | undefined>();

const client = {
  async ping(): Promise<void> {
    if (scenario === "unavailable") throw new Error("the native vault is unavailable");
  },
  async getEntries(): Promise<EntrySummary[]> {
    if (scenario === "loading") {
      return new Promise((resolve) => {
        finishEntryLoading = () => resolve([entry]);
      });
    }
    return scenario === "empty" ? [] : [entry];
  },
  async getTotp(entryId: string): Promise<{ code: string; secondsRemaining: number }> {
    if (entryId !== entry.id) throw new Error(`no such entry: ${entryId}`);
    return { code: "135791", secondsRemaining: 30 };
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

{#if finishEntryLoading}
  <button onclick={finishEntryLoading} type="button">Finish entry loading</button>
{/if}
