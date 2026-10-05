<script lang="ts">
import type { OtpImportCandidate, OtpImportSelection } from "@castellan/protocol";

import OtpImport from "@/src/components/OtpImport.svelte";

type Scenario = "review" | "race" | "pending-import";

interface Props {
  scenario?: Scenario;
}

let { scenario = "review" }: Props = $props();

const candidates: OtpImportCandidate[] = [
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

let imported = $state<OtpImportSelection[] | null>(null);
let importCalls = $state(0);
let releaseFirstPreview = $state<(() => void) | undefined>();
let finishImport = $state<(() => void) | undefined>();

async function preview(payload: string): Promise<OtpImportCandidate[]> {
  if (payload.trim() === "garbage") {
    throw new Error("not a recognized import");
  }

  if (scenario === "race") {
    if (payload === "first") {
      return new Promise((resolve) => {
        releaseFirstPreview = () =>
          resolve([
            { issuer: "Stale", account: "stale", otpauth: "otpauth://totp/Stale", problem: null }
          ]);
      });
    }
    if (payload === "second") {
      return [
        { issuer: "Current", account: "current", otpauth: "otpauth://totp/Current", problem: null }
      ];
    }
  }

  return scenario === "pending-import" ? candidates.slice(0, 1) : candidates;
}

async function importAccounts(accounts: OtpImportSelection[]): Promise<number> {
  importCalls += 1;
  if (scenario === "pending-import") {
    return new Promise((resolve) => {
      finishImport = () => {
        imported = accounts;
        resolve(accounts.length);
      };
    });
  }
  imported = accounts;
  return accounts.length;
}

async function decodeImage(_file: File): Promise<string | null> {
  return "decoded-from-qr";
}
</script>

<OtpImport {decodeImage} {importAccounts} {preview} />

<!-- Deterministic test controls resolve the in-browser fake service only after
     the user flow has reached its pending state. -->
{#if releaseFirstPreview}
  <button onclick={releaseFirstPreview} type="button">Release stale preview</button>
{/if}
{#if finishImport}
  <button onclick={finishImport} type="button">Finish import</button>
{/if}

<!-- The fixture exposes callback receipt as observable UI, not a test-side mock. -->
<output aria-label="Import requests">{importCalls}</output>
{#if imported !== null}
  <ol aria-label="Imported accounts">
    {#each imported as account (account.otpauth)}
      <li>{account.title}:{account.username}</li>
    {/each}
  </ol>
{/if}
