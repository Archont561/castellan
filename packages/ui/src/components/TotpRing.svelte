<!--
  A countdown ring around a TOTP code. Receives `secondsRemaining` from the
  protocol answer (the app computes the period arithmetic; the UI only draws
  it), and ticks locally between protocol answers.

  The ring's two circles are the `c-ring-track` / `c-ring-progress`
  shortcuts from this package's UnoCSS preset (`../../uno.ts`); `track` and
  `progress` stay on the elements as test hooks (see EntryRow's header for
  the convention).
-->
<script lang="ts">
interface Props {
  code: string;
  secondsRemaining: number;
  /**
   * Fired once when the local countdown reaches zero — the caller's cue
   * to ask the protocol for the next code. The ring itself never knows
   * the period; it only runs down what the last answer said was left.
   */
  onExpired?: () => void;
}

let { code, secondsRemaining, onExpired }: Props = $props();

// Zero, not the prop: $state would capture the initial prop value
// (svelte's state_referenced_locally warning is right about that);
// the effect below is what syncs it, immediately and on every change.
let remaining = $state(0);

$effect(() => {
  remaining = secondsRemaining;
  const timer = setInterval(() => {
    remaining = Math.max(0, remaining - 1);
    if (remaining === 0) {
      // Stop ticking at zero: the owner refetches and the new
      // `secondsRemaining` restarts this effect with a fresh timer.
      clearInterval(timer);
      onExpired?.();
    }
  }, 1000);
  return () => clearInterval(timer);
});

const circumference = 2 * Math.PI * 9;
let dash = $derived((remaining / Math.max(secondsRemaining, 1)) * circumference);
</script>

<span class="totp inline-flex items-center gap-2">
  <svg aria-hidden="true" height="22" width="22" viewBox="0 0 22 22">
    <circle class="track c-ring-track" cx="11" cy="11" r="9" />
    <circle
      class="progress c-ring-progress"
      cx="11"
      cy="11"
      r="9"
      stroke-dasharray="{circumference}"
      stroke-dashoffset="{circumference - dash}"
    />
  </svg>
  <code class="text-[1.1em] tracking-[0.15em]">{code}</code>
</span>
