---
id: decision-4
title: "SvelteKit static adapter inside Tauri — SSR is not a thing here"
date: '2026-10-01 20:20'
status: accepted
---
## Context

SvelteKit defaults toward SSR; Tauri loads the frontend from local files through a custom
protocol with no server. Every byte of Castellan's data crosses `invoke()` IPC at runtime;
a build-time prerender of a load function would run in Node, where the vault does not
exist.

## Decision

**`adapter-static` with an SPA fallback, `ssr = false` globally, `prerender = true` for
static shells.** Dynamic vault routes (`/entry/[id]`) use the fallback. The decision was
reviewed against "SSG vs SPA" and found immaterial inside a webview: no SEO, no cold
start, so the config optimizes for the least fragile build, not the fastest first paint.

## Consequences

- No server code anywhere in the faces; the SvelteKit apps are compile-to-static UIs over
  one `rpc` command.
- Anything secret compiled into the JS bundle is readable from the app package — the
  AGENTS.md invariant "secrets stay app-side" exists because of this decision.
- A future hosted web face would be a new app, not a config change; that is accepted.
