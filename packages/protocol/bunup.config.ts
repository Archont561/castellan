import { libPreset } from "@castellan/utils/bunup";
import { defineConfig } from "bunup";

/** Two public entries: the barrel, and the generated types on their own —
 * a consumer may want the wire types without the (tiny) runtime barrel. */
export default defineConfig(
  libPreset({
    name: "protocol",
    entry: ["src/index.ts", "src/generated/index.ts"]
  })
);
