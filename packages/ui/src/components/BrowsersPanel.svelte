<!--
  The connected-browsers panel: one place that answers "who is talking to
  my vault right now?" — pending enrollment prompts to allow or deny, live
  connections with their face, version and last request, and the keys
  remembered from earlier sessions.

  Presentational only — the snapshot comes in as a prop (the app-side
  IpcServer's `panel()`), every action is a callback (`onConfirm`,
  `onDeny`, `onKill`), so the desktop shell owns the wiring and this
  component stays testable without a socket.

  Styling is UnoCSS: the looks come from this package's shortcuts
  (`../../uno.ts`), the one-off metrics are utilities, and the leading bare
  class on each element (`panel`, `prompt`, `connection`…) carries no CSS —
  it is the stable hook the component tests and the faces' e2e suites
  query by.
-->
<script lang="ts">
import type { FaceKind, PanelSnapshot } from "@castellan/protocol";

/** The one-line label for each way a manifest can be stale (task-10). */
const problemLabels: Record<string, string> = {
  missing: "Manifest missing",
  unreadable: "Manifest unreadable",
  stale_path: "Manifest points at an old app path",
  missing_id: "Manifest is missing an extension id",
  foreign: "Manifest belongs to another host"
};

interface Props {
  snapshot: PanelSnapshot;
  onConfirm: (keyId: string) => void;
  onDeny: (keyId: string) => void;
  onKill: (connectionId: number) => void;
}

let { snapshot, onConfirm, onDeny, onKill }: Props = $props();

const faceLabels: Record<FaceKind, string> = {
  chrome: "Chrome",
  edge: "Edge",
  brave: "Brave",
  vivaldi: "Vivaldi",
  firefox: "Firefox",
  safari: "Safari",
  cli: "The CLI",
  other: "A browser"
};

function faceLabel(face: FaceKind): string {
  return faceLabels[face];
}

/** Date-only, ISO — stable across locales, and a remembered key's exact
 * hour is nobody's question. */
function dateOnly(unixSeconds: number): string {
  return new Date(unixSeconds * 1000).toISOString().slice(0, 10);
}
</script>

<section class="panel" aria-label="Connected browsers">
  {#if snapshot.pending.length > 0}
    <section class="prompts" aria-label="Waiting for approval">
      <h2 class="prompt-title c-section-title text-[0.7em]">Wants to connect</h2>
      <ul class="list-none p-0 m-0">
        {#each snapshot.pending as key (key.key_id)}
          <li class="prompt flex items-center gap-2 py-[0.4rem]">
            <span class="label flex-1">{key.label}</span>
            <button
              class="allow c-action px-[0.6rem] py-[0.25rem] rounded-8px"
              onclick={() => onConfirm(key.key_id)}
              type="button"
            >
              Allow
            </button>
            <button
              class="deny px-[0.6rem] py-[0.25rem] rounded-8px b-1 b-solid border-border bg-transparent cursor-pointer"
              onclick={() => onDeny(key.key_id)}
              type="button"
            >
              Deny
            </button>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  <h2 class="connections-title c-section-title text-[0.7em]">Connected</h2>
  {#if snapshot.connections.length === 0}
    <p class="empty text-muted">No browsers connected.</p>
  {:else}
    <ul class="list-none p-0 m-0">
      {#each snapshot.connections as connection (connection.id)}
        <li class="connection flex items-center gap-2 py-[0.4rem]">
          <span class="face font-600">{faceLabel(connection.face)}</span>
          <span class="version text-[0.85em] text-tint-60">
            {connection.client_version ?? "unknown version"}
          </span>
          {#if connection.state === "ready"}
            <span class="state c-badge" title="Ready">ready</span>
          {:else}
            <span class="state c-badge" title="Waiting for approval">waiting</span>
          {/if}
          <span class="last-request text-[0.85em] text-tint-60" title="Last request">
            {connection.last_request ?? "—"}
          </span>
          <button
            class="kill ml-auto px-[0.6rem] py-[0.25rem] rounded-8px b-1 b-solid border-border bg-transparent cursor-pointer"
            aria-label="Disconnect {faceLabel(connection.face)}"
            onclick={() => onKill(connection.id)}
            type="button"
          >
            Disconnect
          </button>
        </li>
      {/each}
    </ul>
  {/if}

  {#if snapshot.manifest_problems.length > 0}
    <section class="manifests" aria-label="Native messaging manifests">
      <h2 class="manifest-title c-section-title text-[0.7em]">Needs repair</h2>
      <ul class="list-none p-0 m-0">
        {#each snapshot.manifest_problems as problem (problem.detail)}
          <li class="problem flex items-baseline gap-2 py-[0.25rem]">
            <span class="face font-600">{faceLabel(problem.browser)}</span>
            <span class="kind text-[0.85em] text-danger">
              {problemLabels[problem.kind] ?? "Manifest needs repair"}
            </span>
            <span class="detail text-[0.85em] text-tint-60">{problem.detail}</span>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  {#if snapshot.remembered.length > 0}
    <section class="remembered-keys" aria-label="Remembered browsers">
      <h2 class="remembered-title c-section-title text-[0.7em]">Remembered</h2>
      <ul class="list-none p-0 m-0">
        {#each snapshot.remembered as key (key.key_id)}
          <li class="remembered flex items-baseline gap-2 py-[0.25rem]">
            <span class="label">{key.label}</span>
            <span class="since text-[0.85em] text-tint-60">since {dateOnly(key.added_at)}</span>
          </li>
        {/each}
      </ul>
    </section>
  {/if}
</section>
