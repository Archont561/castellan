import tauriConfig from "./src-tauri/tauri.conf.json" with { type: "json" };

/** Tauri owns the development URL; Vite and Playwright derive from it. */
export const DEV_URL = tauriConfig.build.devUrl;
export const DEV_PORT = Number(new URL(DEV_URL).port);

if (!Number.isInteger(DEV_PORT) || DEV_PORT <= 0) {
  throw new Error(`invalid Tauri build.devUrl: ${DEV_URL}`);
}
