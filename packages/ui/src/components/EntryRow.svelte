<!--
  One row of a vault entry list: title, username, capability dots.
  Presentational only — data comes in as props, every action is an event
  callback, and the components stay usable in the desktop grid and the mobile
  list alike.
-->
<script lang="ts">
import type { EntrySummary } from "@castellan/protocol";

interface Props {
  entry: EntrySummary;
  onPick?: (entry: EntrySummary) => void;
}

let { entry, onPick }: Props = $props();
</script>

<button class="row" onclick={() => onPick?.(entry)} type="button">
  <span class="title">{entry.title}</span>
  {#if entry.username}
    <span class="username">{entry.username}</span>
  {/if}
  <span class="badges">
    {#if entry.has_totp}<span class="badge" title="TOTP">2FA</span>{/if}
    {#if entry.has_passkey}<span class="badge" title="Passkey">🔑</span>{/if}
  </span>
</button>

<style>
  .row {
    display: grid;
    grid-template-columns: 1fr auto auto;
    align-items: baseline;
    gap: 0.75rem;
    width: 100%;
    padding: 0.6rem 0.8rem;
    background: none;
    border: 0;
    border-radius: 8px;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .row:hover {
    background: color-mix(in oklab, currentColor 8%, transparent);
  }
  .title {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .username {
    color: color-mix(in oklab, currentColor 60%, transparent);
    font-size: 0.85em;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .badges {
    display: flex;
    gap: 0.25rem;
  }
  .badge {
    font-size: 0.7em;
    padding: 0.1rem 0.4rem;
    border-radius: 999px;
    border: 1px solid color-mix(in oklab, currentColor 30%, transparent);
  }
</style>
