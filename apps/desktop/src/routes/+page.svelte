<script lang="ts">
import { tauriTransport } from "@castellan/tauri";
import { OtpImport, VaultHome } from "@castellan/ui";
import { DesktopClient } from "@/src/generated/client";
import { captureScreen, decodeImage } from "@/src/lib/qr";

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

<!--
  The authenticator import flow (task-13). Composition and metrics stay
  at the face boundary, same as VaultHome above; the desktop face is
  also the only one that wires the two QR acquisition callbacks — a
  dropped screenshot and the screen-capture picker are desktop input
  surfaces, while the parsing itself happens app-side behind
  `preview_otp_import`.
-->
<div class="mx-auto max-w-640px px-6 pb-8">
  <OtpImport
    actionClass="rounded-8px px-4 py-2"
    captureScreen={captureScreen}
    decodeImage={decodeImage}
    fieldClass="px-3 py-2 text-[0.9rem]"
    importAccounts={(accounts) => client.importOtpAccounts(accounts)}
    preview={(payload) => client.previewOtpImport(payload)}
    sectionTitleClass="text-[1rem]"
  />
</div>
