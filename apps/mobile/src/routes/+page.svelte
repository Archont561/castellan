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

<main>
  <header>
    <LockedShield locked />
    <h1>Castellan</h1>
  </header>

  {#if error}
    <p class="error">{error}</p>
  {:else}
    <section>
      <h2>Entries for github.com</h2>
      {#if entries.length === 0}
        <p class="muted">No entries yet — the vault core answers.</p>
      {:else}
        {#each entries as entry (entry.id)}
          <EntryRow {entry} />
        {/each}
      {/if}
    </section>

    <section>
      <h2>Passphrase</h2>
      <code class="phrase">{passphrase || "—"}</code>
      <button onclick={roll} type="button">Generate</button>
    </section>
  {/if}
</main>

<style>
  main {
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  h1 {
    font-size: 1.3rem;
    margin: 0;
  }
  h2 {
    font-size: 0.85rem;
    color: var(--muted);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .muted {
    color: var(--muted);
  }
  .phrase {
    display: block;
    padding: 0.6rem 0.8rem;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    font-size: 0.95rem;
    margin-bottom: 0.6rem;
    word-break: break-all;
  }
  button {
    width: 100%;
    padding: 0.75rem;
    border-radius: 10px;
    border: 1px solid var(--border);
    background: var(--accent);
    color: #14161a;
    font-weight: 600;
  }
  .error {
    color: #e06c75;
  }
</style>
