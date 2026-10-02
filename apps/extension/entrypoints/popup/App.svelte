<script lang="ts">
import { PROTOCOL_VERSION } from "@castellan/protocol";
import { onMount } from "svelte";
import { pingBackground } from "@/src/messages";

let state = $state<"checking" | "up" | "down">("checking");

onMount(async () => {
  const answer = await pingBackground(browser.runtime);
  state = answer.ok ? "up" : "down";
});
</script>

<!--
  The popup uses the same shared UnoCSS foundation as the apps, with
  `shell: false` (uno.config.ts): tokens and tints, but no `html`/`body`
  painting — the browser owns this surface and sizes it from the content.
  No `@castellan/ui` preset either: the popup renders none of the shared
  components, so it ships none of their looks. The status colour is a data-attribute variant rather than a
  CSS rule on `[data-state]`, so the state → colour mapping is visible in
  the markup that sets the state. `status`/`muted` stay as e2e hooks.
-->
<main class="w-240px p-[0.9rem] font-sans">
  <h1 class="mb-1 mt-0 text-[1.1rem]">Fob</h1>
  <p class="status data-[state=up]:text-ok data-[state=down]:text-danger" data-state={state}>
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
  <p class="muted text-[0.75rem] text-muted">protocol v{PROTOCOL_VERSION} · no cloud, no accounts</p>
</main>
