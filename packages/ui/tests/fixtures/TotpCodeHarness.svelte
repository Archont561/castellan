<script lang="ts">
import TotpCode from "@/src/components/TotpCode.svelte";

// A scripted protocol: the first answer expires in one second, the
// second is a fresh window. One real tick is the whole timing budget —
// enough to prove the refetch loop, short enough not to flake.
const answers = [
  { code: "111111", secondsRemaining: 1 },
  { code: "222222", secondsRemaining: 30 }
];
let calls = 0;

async function getCode(): Promise<{ code: string; secondsRemaining: number }> {
  const answer = answers[Math.min(calls, answers.length - 1)] as (typeof answers)[number];
  calls += 1;
  return answer;
}
</script>

<TotpCode {getCode} />
