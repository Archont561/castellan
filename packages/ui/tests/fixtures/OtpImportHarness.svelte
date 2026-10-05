<script lang="ts">
import type { OtpImportCandidate, OtpImportSelection } from "@castellan/protocol";

import OtpImport from "@/src/components/OtpImport.svelte";

// A scripted protocol pair: preview answers a fixed review (two
// importable accounts, one refused), import records what it was handed
// so the test can read the selection back out of the DOM.
let imported = $state<OtpImportSelection[] | null>(null);

async function preview(payload: string): Promise<OtpImportCandidate[]> {
  if (payload.trim() === "garbage") {
    throw new Error("not a recognized import");
  }
  return [
    {
      issuer: "GitHub",
      account: "octocat",
      otpauth: "otpauth://totp/GitHub:octocat?secret=JBSWY3DPEHPK3PXP",
      problem: null
    },
    {
      issuer: "Example",
      account: "alice",
      otpauth: "otpauth://totp/Example:alice?secret=JBSWY3DPEHPK3PXP",
      problem: null
    },
    {
      issuer: "Legacy",
      account: "bob",
      otpauth: null,
      problem: "HOTP accounts need counter support (task-42)"
    }
  ];
}

async function importAccounts(accounts: OtpImportSelection[]): Promise<number> {
  imported = accounts;
  return accounts.length;
}

// The QR path, scripted: any image "decodes" to a fixed payload — the
// component's contract is that whatever the face decodes is previewed,
// and the decoder itself is the face's business (jsqr on desktop).
async function decodeImage(_file: File): Promise<string | null> {
  return "decoded-from-qr";
}
</script>

<OtpImport {decodeImage} {importAccounts} {preview} />

{#if imported !== null}
  <ol class="imported-record">
    {#each imported as account (account.otpauth)}
      <li>{account.title}:{account.username}</li>
    {/each}
  </ol>
{/if}
