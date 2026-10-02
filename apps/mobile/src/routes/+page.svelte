<script lang="ts">
import { tauriTransport } from "@castellan/tauri";
import { VaultHome } from "@castellan/ui";
import { MobileClient } from "@/src/generated/client";

const client = new MobileClient(tauriTransport());
</script>

<!-- Shared behavior and structure, with phone metrics owned by this face. -->
<VaultHome
  {client}
  mainClass="flex flex-col gap-6 p-5"
  titleClass="m-0 text-[1.3rem]"
  sectionTitleClass="text-[0.85rem]"
  phraseClass="mb-[0.6rem] break-all px-[0.8rem] py-[0.6rem] text-[0.95rem]"
  actionClass="w-full rounded-10px p-3"
  emptyMessage="No entries yet — the vault core answers."
/>
