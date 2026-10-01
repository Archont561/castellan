import { libPreset } from "@castellan/utils/bunup";
import { defineConfig } from "bunup";

export default defineConfig(
  libPreset({
    name: "core",
    entry: ["src/index.ts"]
  })
);
