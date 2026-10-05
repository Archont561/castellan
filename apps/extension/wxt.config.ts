import UnoCSS from "unocss/vite";
import { defineConfig } from "wxt";

// One codebase, every engine: WXT compiles this to chromium (Chrome, Edge,
// Brave, Vivaldi...) and gecko (Firefox) with per-engine manifests, and the
// engine differences that matter live in src/adapters/, not scattered
// through the code. The extension name is the product's sub-brand: the
// passkey companion is "Fob", the app is "Castellan".
export default defineConfig({
  modules: ["@wxt-dev/module-svelte"],
  vite: () => ({
    // The same UnoCSS pass the apps run, reading ./uno.config.ts (the
    // shared preset with `shell: false`). WXT builds each entrypoint as its
    // own vite build, so the popup gets exactly the utilities the popup
    // uses and the content script pays for none of them.
    plugins: [UnoCSS()],
    server: {
      host: "0.0.0.0",
      allowedHosts: true
    },
    resolve: {
      // Svelte's package exports split client/server runtimes on the
      // "browser" condition, and WXT's multi-entry build resolves some
      // entries without it — which bundles the *server* runtime into the
      // popup, where `mount()` is a stub that throws
      // `lifecycle_function_unavailable` (caught by the e2e suite; the
      // build itself succeeds silently). Adding the condition pins every
      // entry to the client runtime.
      conditions: ["browser"]
    }
  }),
  srcDir: ".",
  manifest: {
    name: "Fob — Castellan companion",
    short_name: "Fob",
    description:
      "Autofill, TOTP and passkeys from your Castellan vault. Local-first: no cloud, no accounts.",
    // Firefox native-host manifests allow extension IDs rather than Chrome
    // origins, so development needs a stable ID shared with host.json.
    browser_specific_settings: {
      gecko: { id: "fob-dev@castellan.app" }
    },
    // nativeMessaging is the whole point: stdio to the app's host binary,
    // no localhost socket, no firewall prompt, nothing for other processes
    // to poke at.
    permissions: ["nativeMessaging", "storage", "activeTab"],
    // host_permissions stay minimal: the content script needs the page only
    // where the user invokes a fill. The passkey layer injects at
    // document_start, MAIN world, and needs nothing more than <all_urls>.
    host_permissions: ["<all_urls>"]
  }
});
