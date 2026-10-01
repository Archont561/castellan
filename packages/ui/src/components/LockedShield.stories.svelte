<script module lang="ts">
import { defineMeta } from "@storybook/addon-svelte-csf";
import LockedShield from "./LockedShield.svelte";

const { Story } = defineMeta({
  title: "Components/LockedShield",
  component: LockedShield
});
</script>

<!--
  The vault state as an icon. Both faces render it in their header, so
  both states are stories: the loop closed, and open.
-->
<Story name="Locked" args={{ locked: true }} />
<Story name="Unlocked" args={{ locked: false }} />
