<script module lang="ts">
import type { OtpImportCandidate, OtpImportSelection } from "@castellan/protocol";
import { defineMeta } from "@storybook/addon-svelte-csf";
import OtpImport from "./OtpImport.svelte";

const { Story } = defineMeta({
  title: "Components/OtpImport",
  component: OtpImport
});

/** A scripted preview: whatever is pasted reviews as one healthy batch
 * with one refused account — the shape a Google migration QR with a
 * HOTP straggler produces. */
async function preview(_payload: string): Promise<OtpImportCandidate[]> {
  return [
    {
      issuer: "GitHub",
      account: "octocat",
      otpauth: "otpauth://totp/GitHub:octocat?secret=JBSWY3DPEHPK3PXP",
      problem: null
    },
    {
      issuer: "Example",
      account: "alice@example.org",
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
  return accounts.length;
}

async function previewFails(_payload: string): Promise<OtpImportCandidate[]> {
  throw new Error("This export could not be read.");
}
</script>

<!--
  The import review flow with a scripted protocol: paste anything, hit
  Preview, uncheck what should stay behind, import. The refused row
  shows how a problem account is named instead of dropped.
-->
<Story name="Review" args={{ preview, importAccounts }} />
<Story name="Preview fails" args={{ preview: previewFails, importAccounts }} />
