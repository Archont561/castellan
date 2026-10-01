<!-- The lock state, as a component: the vault icon every face shows. -->
<script lang="ts">
interface Props {
  locked: boolean;
}

let { locked }: Props = $props();
</script>

<svg
  aria-label={locked ? "vault locked" : "vault unlocked"}
  height="18"
  role="img"
  viewBox="0 0 24 24"
  width="18"
>
  {#if locked}
    <path
      d="M17 9V7a5 5 0 0 0-9.6-2M5 11a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2z"
      fill="none"
      stroke="currentColor"
      stroke-linecap="round"
      stroke-width="2"
    />
  {:else}
    <path
      d="M7 11V7a5 5 0 0 1 10 0v4M5 11a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2z"
      fill="none"
      stroke="currentColor"
      stroke-linecap="round"
      stroke-width="2"
    />
  {/if}
</svg>
