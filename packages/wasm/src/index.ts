/**
 * @castellan/wasm — the shared Rust logic, compiled for the browser side.
 *
 * The extension imports this instead of re-implementing otpauth parsing or
 * origin matching in TypeScript: `parseOtpauth` here is `castellan_otp::parse`
 * in the app, one implementation, two targets. Build it from the repo root:
 *
 * ```console
 * $ bun run wasm
 * ```
 *
 * which runs `wasm-pack build crates/wasm --target bundler` into `src/pkg/`
 * (git-ignored: it is a build artifact, and rebuilding is deterministic).
 *
 * The module is loaded lazily and cached: an MV3 service worker that
 * imports WASM eagerly is a service worker that dies on its own schedule
 * with a load it never needed.
 */

/** What the generated binding exports; mirrors crates/wasm/src/lib.rs. */
interface CastellanWasm {
  protocol_version(): number;
  parse_otpauth(uri: string): {
    issuer: string | null;
    account: string;
    digits: number;
    period: number;
  };
  origin_matches(origin: string, candidate: string): boolean;
}

/** The parsed shape of an otpauth URI, without the secret. */
export interface OtpAuthInfo {
  issuer: string | null;
  account: string;
  digits: number;
  period: number;
}

let loaded: Promise<CastellanWasm> | undefined;

/**
 * Load (once) and return the WASM module.
 *
 * Throws with a readable message when the package has not been built — the
 * one failure mode worth naming, because "bun install then import" without
 * "bun run wasm" is an easy state to reach and a confusing one to debug.
 */
export function loadWasm(): Promise<CastellanWasm> {
  loaded ??= (async () => {
    try {
      // wasm-bindgen's bundler target (≥ 0.2.93) self-initializes on
      // import: there is no init()/default to call — the module loads the
      // .wasm as a static ESM import and runs __wbindgen_start itself.
      const wasm = await import("./pkg/castellan_wasm.js");
      return wasm as CastellanWasm;
    } catch {
      throw new Error(
        "@castellan/wasm: the WASM build is missing. Run `bun run wasm` at the repository root, then retry."
      );
    }
  })();
  return loaded;
}

/** The protocol version of the shared logic this build was cut from. */
export async function protocolVersion(): Promise<number> {
  return (await loadWasm()).protocol_version();
}

/** Parse an otpauth URI for preview/validation. Never exposes the secret. */
export async function parseOtpauth(uri: string): Promise<OtpAuthInfo> {
  // wasm-bindgen types the binding as `any` (serde-wasm-bindgen values);
  // the shape is the Rust struct's, documented in the interface above.
  return (await loadWasm()).parse_otpauth(uri) as OtpAuthInfo;
}

/**
 * Whether an entry saved at `candidate` should be offered at `origin`.
 *
 * The same rule the app applies when answering `get_entries`; the extension
 * pre-filters with it so its local view and the app's answer can never
 * disagree.
 */
export async function originMatches(origin: string, candidate: string): Promise<boolean> {
  return (await loadWasm()).origin_matches(origin, candidate);
}
