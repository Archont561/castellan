<script lang="ts">
import { PROTOCOL_VERSION } from "@castellan/protocol";
import { onMount } from "svelte";

let state = $state<"checking" | "up" | "down">("checking");

onMount(async () => {
  const answer = await browser.runtime.sendMessage("castellan:ping");
  state = answer?.ok ? "up" : "down";
});
</script>

<main>
  <h1>Fob</h1>
  <p class="status" data-state={state}>
    <!-- {#if}, not `state === "…" && "…"`: Svelte 5 renders a bare `false`
         in text position as the string "false", so the && form leaks
         "false" lines into the popup — caught by the e2e suite. -->
    {#if state === "checking"}
      Looking for Castellan…
    {:else if state === "up"}
      Connected to Castellan
    {:else}
      Castellan is not running
    {/if}
  </p>
  <p class="muted">protocol v{PROTOCOL_VERSION} · no cloud, no accounts</p>
</main>

<style>
  main {
    width: 240px;
    padding: 0.9rem;
    font-family: system-ui, sans-serif;
  }
  h1 {
    margin: 0 0 0.25rem;
    font-size: 1.1rem;
  }
  .status[data-state="up"] {
    color: #4caf7d;
  }
  .status[data-state="down"] {
    color: #e06c75;
  }
  .muted {
    color: #8a90a0;
    font-size: 0.75rem;
  }
</style>
