<!--
  A live TOTP code: fetches through the supplied callback, draws the
  TotpRing from the protocol's answer, and refetches when the ring runs
  out. The division of labor is the task-13 contract: the app computes
  the code and `seconds_remaining`; this component only displays and
  re-asks. No period arithmetic happens in TypeScript — a client that
  guessed "30 seconds" would drift on every 60-second seed.

  The bare `totp-code`, `totp-error`, `totp-loading` classes are test
  hooks (EntryRow's header documents the convention).
-->
<script lang="ts">
import TotpRing from "./TotpRing.svelte";

interface Totp {
  code: string;
  secondsRemaining: number;
}

interface Props {
  /** Ask the app for the entry's current code — `client.getTotp(id)`. */
  getCode: () => Promise<Totp>;
}

let { getCode }: Props = $props();

let totp = $state<Totp | null>(null);
let error = $state("");

async function refresh(): Promise<void> {
  try {
    totp = await getCode();
    error = "";
  } catch (cause) {
    error = String(cause);
  }
}

$effect(() => {
  void refresh();
});
</script>

{#if error}
  <p class="totp-error text-danger" role="alert">{error}</p>
{:else if totp}
  <!-- Keyed by the answer object: every protocol answer remounts the
       ring, so its countdown restarts even when two consecutive answers
       carry equal numbers (a fresh 30 after a fresh 30). -->
  {#key totp}
    <span class="totp-code">
      <TotpRing code={totp.code} secondsRemaining={totp.secondsRemaining} onExpired={refresh} />
    </span>
  {/key}
{:else}
  <span aria-label="Loading authenticator code" class="totp-loading text-muted" role="status">······</span>
{/if}
