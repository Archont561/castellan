/**
 * @castellan/ui — shared Svelte components for the desktop and mobile faces.
 *
 * Components are presentational and prop-driven: no fetches, no transports,
 * no app state. The faces own composition; this package owns the look. It is
 * consumed as source (the apps' vite configs compile the .svelte files), so
 * there is no build step here to fall out of date.
 */
export { default as EntryRow } from "./components/EntryRow.svelte";
export { default as LockedShield } from "./components/LockedShield.svelte";
export { default as TotpRing } from "./components/TotpRing.svelte";
