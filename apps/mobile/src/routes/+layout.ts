// Tauri has no server, so no SSR: everything renders in the webview, and
// every byte of data crosses invoke() at runtime. Prerendering still emits
// static shells for the routes that exist at build time; dynamic routes
// (vault entries) fall back to index.html.
export const prerender = true;
export const ssr = false;
