/**
 * @castellan/ui — shared Svelte components for the desktop and mobile faces.
 *
 * Components are prop-driven and never import a transport or generated face
 * client. Shared surface components may own their local loading/error state;
 * each face still owns composition, client construction, and metrics. The
 * package is consumed as source (the apps' vite configs compile the .svelte
 * files), so there is no build artifact to fall out of date.
 */
export { default as BrowsersPanel } from "./components/BrowsersPanel.svelte";
export { default as EntryRow } from "./components/EntryRow.svelte";
export { default as LockedShield } from "./components/LockedShield.svelte";
export { default as TotpRing } from "./components/TotpRing.svelte";
export { default as VaultHome } from "./components/VaultHome.svelte";
