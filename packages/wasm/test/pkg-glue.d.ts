/**
 * Types for the generated glue module the artifact test imports by hand.
 *
 * wasm-pack emits declarations for the package's public entry
 * (`castellan_wasm.d.ts`) and for the wasm module itself
 * (`castellan_wasm_bg.wasm.d.ts`), but none for `castellan_wasm_bg.js` — the
 * glue a bundler wires up on its own. `wasm.test.ts` does that wiring by hand
 * (its header says why), so it is the one importer `tsc` sees, and an untyped
 * import under the base config's `noImplicitAny` is TS7016.
 *
 * Declaring the four members the test touches keeps the check on instead of
 * casting the module to `any`. Nothing else imports the glue: the production
 * path goes through `src/index.ts` → `castellan_wasm.js`, which ships its own
 * declarations. If wasm-bindgen ever emits a `castellan_wasm_bg.d.ts`, this
 * file is redundant and should go.
 */
declare module "@/src/pkg/castellan_wasm_bg.js" {
  /** Hands the instantiated exports to the glue's module-global wasm slot. */
  export function __wbg_set_wasm(exports: unknown): void;
  export function protocol_version(): number;
  /** serde-wasm-bindgen value; the test narrows it to the Rust struct's shape. */
  export function parse_otpauth(uri: string): unknown;
  export function origin_matches(origin: string, candidate: string): boolean;
}
