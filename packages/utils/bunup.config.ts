import { libPreset } from "@castellan/utils/bunup";
import { defineConfig } from "bunup";

/** The reuse package's three faces: the barrel, the test fixtures, and the
 * bunup preset itself — the preset is consumed at build time by its own
 * package's config, so it had better build too. */
export default defineConfig(
  libPreset({
    name: "utils",
    entry: ["src/index.ts", "src/fixtures.ts", "src/bunup.ts"]
  })
);
