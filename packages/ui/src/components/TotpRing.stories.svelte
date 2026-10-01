<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import TotpRing from "./TotpRing.svelte";

const { Story } = defineMeta({
  title: "Components/TotpRing",
  component: TotpRing
});
</script>

<!--
  The countdown ring around a TOTP code. The component ticks locally
  between protocol answers, so the stories watch it run: one ring with a
  fresh window, one about to rotate.
-->
<Story name="Fresh window" args={{ code: "482 913", secondsRemaining: 30 }} />
<Story name="About to rotate" args={{ code: "077 341", secondsRemaining: 3 }} />
