<!--
  One row of a vault entry list: title, username, capability dots.
  Presentational only — data comes in as props, every action is an event
  callback, and the components stay usable in the desktop grid and the mobile
  list alike.

  Styling is UnoCSS: the row's look is the `c-entry-row` shortcut from this
  package's own preset (`../../uno.ts`, layered on the foundation in
  `@castellan/utils/uno`), the one-off metrics are utilities. The leading bare class on each element
  (`title`, `username`, `badge`…) carries no CSS — it is the stable hook the
  component tests and the faces' e2e suites query by, kept deliberately
  separate from the utilities, which are free to change with the design.
-->
<script lang="ts">
import type { EntrySummary } from "@castellan/protocol";

interface Props {
  entry: EntrySummary;
  onPick?: (entry: EntrySummary) => void;
}

let { entry, onPick }: Props = $props();
</script>

<button class="row c-entry-row" onclick={() => onPick?.(entry)} type="button">
  <span class="title font-600 truncate">{entry.title}</span>
  {#if entry.username}
    <span class="username truncate text-[0.85em] text-tint-60">{entry.username}</span>
  {/if}
  <span class="badges flex gap-1">
    {#if entry.has_totp}<span class="badge c-badge" title="TOTP">2FA</span>{/if}
    {#if entry.has_passkey}<span aria-label="Passkey" class="badge c-badge" role="img">🔑</span>{/if}
  </span>
</button>
