<!--
  A countdown ring around a TOTP code. Receives `secondsRemaining` from the
  protocol answer (the app computes the period arithmetic; the UI only draws
  it), and ticks locally between protocol answers.
-->
<script lang="ts">
interface Props {
  code: string;
  secondsRemaining: number;
}

let { code, secondsRemaining }: Props = $props();

// Zero, not the prop: $state would capture the initial prop value
// (svelte's state_referenced_locally warning is right about that);
// the effect below is what syncs it, immediately and on every change.
let remaining = $state(0);

$effect(() => {
  remaining = secondsRemaining;
  const timer = setInterval(() => {
    remaining = Math.max(0, remaining - 1);
  }, 1000);
  return () => clearInterval(timer);
});

const circumference = 2 * Math.PI * 9;
let dash = $derived((remaining / Math.max(secondsRemaining, 1)) * circumference);
</script>

<span class="totp">
  <svg aria-hidden="true" height="22" width="22" viewBox="0 0 22 22">
    <circle class="track" cx="11" cy="11" r="9" />
    <circle
      class="progress"
      cx="11"
      cy="11"
      r="9"
      stroke-dasharray="{circumference}"
      stroke-dashoffset="{circumference - dash}"
    />
  </svg>
  <code>{code}</code>
</span>

<style>
  .totp {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
  }
  .track,
  .progress {
    fill: none;
    stroke-width: 2.5;
  }
  .track {
    stroke: color-mix(in oklab, currentColor 20%, transparent);
  }
  .progress {
    stroke: currentColor;
    transform: rotate(-90deg);
    transform-origin: center;
    transition: stroke-dashoffset 1s linear;
  }
  code {
    font-size: 1.1em;
    letter-spacing: 0.15em;
  }
</style>
