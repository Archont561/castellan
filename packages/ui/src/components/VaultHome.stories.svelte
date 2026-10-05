<script module lang="ts">
import type { EntrySummary } from "@castellan/protocol";
import { defineMeta } from "@storybook/addon-svelte-csf";
import VaultHome from "./VaultHome.svelte";

const entry: EntrySummary = {
  id: "entry-1",
  title: "GitHub",
  username: "octocat",
  url: "https://github.com",
  has_totp: true,
  has_passkey: false
};

const layout = {
  mainClass: "mx-auto flex max-w-640px flex-col gap-8 p-6",
  titleClass: "m-0 text-[1.4rem]",
  sectionTitleClass: "text-[1rem]",
  phraseClass: "mb-3 px-4 py-3 text-[1.05rem]",
  actionClass: "rounded-8px px-4 py-2",
  emptyMessage: "No entries yet — the vault core answers, and it is honestly empty."
};

const readyClient = {
  ping: async () => {},
  getEntries: async () => [entry],
  getTotp: async () => ({ code: "482913", secondsRemaining: 30 }),
  generatePassphrase: async () => "amber-castle-river-lantern"
};

const emptyClient = { ...readyClient, getEntries: async () => [] };
const unavailableClient = {
  ...readyClient,
  ping: async () => {
    throw new Error("The native vault is unavailable.");
  }
};

const { Story } = defineMeta({
  title: "Components/VaultHome",
  component: VaultHome
});
</script>

<!-- The shared home surface in its three distinct app states: loaded,
     successful-but-empty, and unable to reach the face client. -->
<Story name="Loaded" args={{ ...layout, client: readyClient, explainPassphrases: true }} />
<Story name="Empty" args={{ ...layout, client: emptyClient }} />
<Story name="Native client unavailable" args={{ ...layout, client: unavailableClient }} />
