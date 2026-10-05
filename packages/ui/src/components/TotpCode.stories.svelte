<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import TotpCode from "./TotpCode.svelte";

const { Story } = defineMeta({
  title: "Components/TotpCode",
  component: TotpCode
});

/** A scripted protocol: each expiry answers the next code, cycling. */
const codes = [
  { code: "482913", secondsRemaining: 8 },
  { code: "077341", secondsRemaining: 8 },
  { code: "550162", secondsRemaining: 8 }
];
let turn = 0;
async function nextCode(): Promise<{ code: string; secondsRemaining: number }> {
  const answer = codes[turn % codes.length] as (typeof codes)[number];
  turn += 1;
  return answer;
}

async function failingCode(): Promise<{ code: string; secondsRemaining: number }> {
  throw new Error("the vault is locked");
}
</script>

<!--
  The live code display: fetches, draws the ring from the answer, and
  re-asks when the ring runs out. Watch the rotating story for ~8s to
  see a refetch happen — the ticking is real, the protocol is scripted.
-->
<Story name="Rotating" args={{ getCode: nextCode }} />
<Story name="Fetch fails" args={{ getCode: failingCode }} />
