<script module lang="ts">
import type { EntrySummary } from "@castellan/protocol";
import { defineMeta } from "@storybook/addon-svelte-csf";
import EntryRow from "./EntryRow.svelte";

const full: EntrySummary = {
  id: "3f9d2c88-9a41-4b1d-9f6a-6c4f5b2a7e10",
  title: "GitHub",
  username: "ada@castellan.dev",
  url: "https://github.com",
  has_totp: true,
  has_passkey: true
};

const bare: EntrySummary = {
  ...full,
  username: null,
  has_totp: false,
  has_passkey: false
};

const { Story } = defineMeta({
  title: "Components/EntryRow",
  component: EntryRow
});
</script>

<!--
  One row of the vault list. The interesting states are the ones the
  desktop grid and the mobile list share: a fully capable entry, and an
  entry that is nothing but a title. onPick is the seam the apps own.
-->
<Story name="Full" args={{ entry: full }} />
<Story name="Bare" args={{ entry: bare }} />
