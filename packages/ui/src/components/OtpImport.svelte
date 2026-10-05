<!--
  The authenticator import flow: payload in, per-account review, one
  confirmed batch out. Prop-driven like every component here — the two
  protocol calls arrive as callbacks, and the two ways of *acquiring* a
  payload that need platform APIs (decoding a QR image, capturing the
  screen) are optional callbacks the face provides, so the component
  stays mountable in Storybook and the component tests without either.

  The review is the point: every account the payload carried is listed,
  importable ones with a checkbox (checked by default), refused ones
  with the reason — a migration that silently drops accounts is how
  people get locked out. Import sends only what stayed checked.

  Bare classes (`import`, `payload`, `candidate`, `problem`, `summary`,
  `import-error`) are test hooks, per the EntryRow header convention.
-->
<script lang="ts">
import type { OtpImportCandidate, OtpImportSelection } from "@castellan/protocol";

interface Props {
  /** Parse a payload into reviewable accounts — `client.previewOtpImport`. */
  preview: (payload: string) => Promise<OtpImportCandidate[]>;
  /** Store the accepted accounts — `client.importOtpAccounts`. */
  importAccounts: (accounts: OtpImportSelection[]) => Promise<number>;
  /** Decode a QR from an image file into its text, when the face can. */
  decodeImage?: (file: File) => Promise<string | null>;
  /** Capture the screen and decode a QR from it, when the face can. */
  captureScreen?: () => Promise<string | null>;
  sectionTitleClass?: string;
  fieldClass?: string;
  actionClass?: string;
}

let {
  preview,
  importAccounts,
  decodeImage,
  captureScreen,
  sectionTitleClass = "",
  fieldClass = "",
  actionClass = ""
}: Props = $props();

interface Row {
  candidate: OtpImportCandidate;
  accepted: boolean;
}

let payload = $state("");
let rows = $state<Row[]>([]);
let error = $state("");
let summary = $state("");
let previewing = $state(false);
let importing = $state(false);
let previewVersion = 0;

async function runPreview(text: string): Promise<void> {
  const version = ++previewVersion;
  previewing = true;
  error = "";
  summary = "";
  try {
    const accounts = await preview(text);
    // A later Preview action is the user's newer intent. A slow older
    // response must not replace the review they are looking at now.
    if (version !== previewVersion) return;
    rows = accounts.map((candidate) => ({
      candidate,
      accepted: candidate.otpauth !== null
    }));
  } catch (cause) {
    if (version !== previewVersion) return;
    rows = [];
    error = String(cause);
  } finally {
    if (version === previewVersion) previewing = false;
  }
}

async function fromFile(event: Event): Promise<void> {
  const input = event.currentTarget as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  input.value = "";
  error = "";
  // Images go through the face's QR decoder; everything else (an export
  // file, a .txt of URIs) is text and previews directly.
  if (file.type.startsWith("image/")) {
    if (!decodeImage) return;
    const decoded = await decodeImage(file);
    if (decoded === null) {
      error = "No QR code found in that image.";
      return;
    }
    payload = decoded;
    await runPreview(decoded);
    return;
  }
  const text = await file.text();
  payload = text;
  await runPreview(text);
}

async function fromScreen(): Promise<void> {
  if (!captureScreen) return;
  error = "";
  const decoded = await captureScreen();
  if (decoded === null) {
    error = "No QR code found on the captured screen.";
    return;
  }
  payload = decoded;
  await runPreview(decoded);
}

async function runImport(): Promise<void> {
  if (importing) return;
  const selection = rows
    .filter((row) => row.accepted && row.candidate.otpauth !== null)
    .map((row) => ({
      title: row.candidate.issuer ?? row.candidate.account,
      username: row.candidate.account === "" ? null : row.candidate.account,
      otpauth: row.candidate.otpauth as string
    }));
  if (selection.length === 0) return;
  importing = true;
  error = "";
  try {
    const imported = await importAccounts(selection);
    summary = `Imported ${imported} ${imported === 1 ? "account" : "accounts"}.`;
    rows = [];
    payload = "";
  } catch (cause) {
    error = String(cause);
  } finally {
    importing = false;
  }
}

const importable = $derived(
  rows.filter((row) => row.accepted && row.candidate.otpauth !== null).length
);
</script>

<section aria-busy={previewing || importing} class="import flex flex-col gap-3">
  <h2 class={`c-section-title ${sectionTitleClass}`}>Import authenticator codes</h2>
  <p class="text-muted">
    Paste otpauth:// links, a Google Authenticator export QR's text, or an Aegis / andOTP
    export — then review before anything is stored.
  </p>

  <textarea
    aria-label="Import payload"
    bind:value={payload}
    class={`payload c-field font-mono ${fieldClass}`}
    rows="3"
  ></textarea>

  <div class="flex flex-wrap items-center gap-2">
    <button
      class={`preview-action c-action ${actionClass}`}
      disabled={payload.trim() === ""}
      onclick={() => runPreview(payload)}
      type="button"
    >
      Preview
    </button>
    <label class="file-action cursor-pointer underline">
      Import a file or QR image
      <input class="sr-only" onchange={fromFile} type="file" />
    </label>
    {#if captureScreen}
      <button class="capture-action cursor-pointer underline bg-transparent b-0 text-inherit p-0" onclick={fromScreen} type="button">
        Scan a QR on screen
      </button>
    {/if}
  </div>

  {#if previewing}
    <p role="status">Previewing accounts…</p>
  {/if}

  {#if error}
    <p class="import-error text-danger" role="alert">{error}</p>
  {/if}

  {#if rows.length > 0}
    <ul aria-label="Accounts to import" class="review m-0 flex list-none flex-col gap-1 p-0">
      {#each rows as row, index (index)}
        <li class="candidate flex items-baseline gap-2">
          {#if row.candidate.otpauth !== null}
            <input
              aria-label={`Import ${row.candidate.issuer ?? row.candidate.account}`}
              bind:checked={row.accepted}
              disabled={importing}
              type="checkbox"
            />
          {:else}
            <span aria-hidden="true" class="c-badge">skipped</span>
          {/if}
          <span class="candidate-name font-600">
            {row.candidate.issuer ?? row.candidate.account}
          </span>
          {#if row.candidate.issuer && row.candidate.account}
            <span class="candidate-account text-[0.85em] text-tint-60">{row.candidate.account}</span>
          {/if}
          {#if row.candidate.problem}
            <span class="problem text-[0.85em] text-danger">{row.candidate.problem}</span>
          {/if}
        </li>
      {/each}
    </ul>
    <button
      class={`import-action c-action ${actionClass}`}
      disabled={importable === 0 || importing}
      onclick={runImport}
      type="button"
    >
      {#if importing}
        Importing…
      {:else}
        Import {importable} {importable === 1 ? "account" : "accounts"}
      {/if}
    </button>
  {/if}

  {#if summary}
    <p class="summary" role="status">{summary}</p>
  {/if}
</section>
