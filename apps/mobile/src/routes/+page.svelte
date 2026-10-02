<script lang="ts">
import { CastellanClient } from "@castellan/core";
import type { EntrySummary } from "@castellan/protocol";
import { EntryRow, LockedShield } from "@castellan/ui";
import { onMount } from "svelte";
import { tauriTransport } from "$lib/transport";

const client = new CastellanClient(tauriTransport());

let entries = $state<EntrySummary[]>([]);
let passphrase = $state("");
let error = $state("");

onMount(async () => {
  try {
    await client.ping();
    entries = await client.getEntries("https://github.com");
  } catch (cause) {
    error = String(cause);
  }
});

async function roll(): Promise<void> {
  passphrase = await client.generatePassphrase(4, "-");
}
</script>

<!--
  The same shared UnoCSS looks as the desktop face, at phone metrics: no
  max width, a tighter rhythm, and an action that spans the viewport
  because a thumb is not a mouse. Everything that is a *look* rather than a
  size comes from the shortcuts, so the two faces cannot drift.
-->
<main class="flex flex-col gap-6 p-5">
  <header class="flex items-center gap-2">
    <LockedShield locked />
    <h1 class="m-0 text-[1.3rem]">Castellan</h1>
  </header>

  {#if error}
    <p class="error text-danger">{error}</p>
  {:else}
    <section>
      <h2 class="c-section-title text-[0.85rem]">Entries for github.com</h2>
      {#if entries.length === 0}
        <p class="text-muted">No entries yet — the vault core answers.</p>
      {:else}
        {#each entries as entry (entry.id)}
          <EntryRow {entry} />
        {/each}
      {/if}
    </section>

    <section>
      <h2 class="c-section-title text-[0.85rem]">Passphrase</h2>
      <code class="phrase c-field mb-[0.6rem] break-all px-[0.8rem] py-[0.6rem] text-[0.95rem]">
        {passphrase || "—"}
      </code>
      <button class="c-action w-full rounded-10px p-3" onclick={roll} type="button">
        Generate
      </button>
    </section>
  {/if}
</main>
