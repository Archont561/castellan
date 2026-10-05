<script lang="ts">
import TotpCode from "@/src/components/TotpCode.svelte";

type Scenario = "refresh" | "loading" | "rejected";

interface Props {
  scenario?: Scenario;
}

let { scenario = "refresh" }: Props = $props();

const answers = [
  { code: "111111", secondsRemaining: 1 },
  { code: "222222", secondsRemaining: 30 }
];

let calls = 0;
let finishCodeLoading = $state<(() => void) | undefined>();

function getCode(): Promise<{ code: string; secondsRemaining: number }> {
  if (scenario === "rejected") {
    return new Promise((_resolve, reject) => {
      queueMicrotask(() => reject(new Error("the vault is locked")));
    });
  }
  if (scenario === "loading") {
    return new Promise((resolve) => {
      finishCodeLoading = () => resolve({ code: "123456", secondsRemaining: 30 });
    });
  }

  const answer = answers[Math.min(calls, answers.length - 1)] as (typeof answers)[number];
  calls += 1;
  return Promise.resolve(answer);
}
</script>

<TotpCode {getCode} />

{#if finishCodeLoading}
  <button onclick={finishCodeLoading} type="button">Finish code loading</button>
{/if}
