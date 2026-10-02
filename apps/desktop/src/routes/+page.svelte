<script lang="ts">
import { tauriTransport } from "@castellan/tauri";
import { VaultHome } from "@castellan/ui";
import { DesktopClient } from "@/src/generated/client";

const client = new DesktopClient(tauriTransport());
</script>

<!--
  Shared behavior and structure live in VaultHome. Desk-sized metrics stay
  here, at the face boundary, so mobile can make genuinely different choices
  without forking loading, error, and RPC behavior.
-->
<VaultHome
  {client}
  mainClass="mx-auto flex max-w-640px flex-col gap-8 px-6 py-8"
  titleClass="m-0 text-[1.4rem]"
  sectionTitleClass="text-[1rem]"
  phraseClass="mb-3 px-4 py-3 text-[1.05rem]"
  actionClass="rounded-8px px-4 py-2"
  emptyMessage="No entries yet — the vault core answers, and it is honestly empty."
  explainPassphrases
/>
