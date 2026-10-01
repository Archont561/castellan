/**
 * The WASM artifact and its glue, proven against the real build.
 *
 * Bun (the test runtime) cannot import wasm-ESM the way bundlers do — the
 * dynamic `import()` of `castellan_wasm.js` dies at `wasm.__wbindgen_start`
 * because bun surfaces no named exports for the .wasm module. The
 * extension's bundler (vite, through WXT) does this wiring natively, which
 * is the production path and stays as-is in `src/index.ts`.
 *
 * So the test does by hand what vite does: instantiate the .wasm with the
 * glue's own import functions, hand the exports to `__wbg_set_wasm`, and
 * call through. The artifact is a `createFixture(..., "file")` because the
 * glue's module-global caches (`cachedUint8ArrayMemory0` and friends)
 * assume a single wasm instance per module lifetime — one build, shared by
 * every test in the file, torn down by nothing because the process ends
 * when the run does. The last test pins the wrapper's behavior under
 * exactly this runtime: a readable error, not a stack trace — the day bun
 * grows wasm-ESM interop, that test fails, and the right response is to
 * delete it and this comment, not to "fix" the wrapper.
 *
 * Requires `bun run wasm` first; the package's turbo task graph
 * (`test` dependsOn `build`) guarantees the ordering.
 */
import { describe, expect, test } from "bun:test";

import { createFixture } from "@castellan/utils/fixtures";

import { loadWasm } from "@/src/index";
import * as glue from "@/src/pkg/castellan_wasm_bg.js";

/** The vite path, done by hand — once per file, via the fixture helper.
 * @castellan/utils' own tests pin the lifecycle; this is the helper's
 * flagship use: expensive, immutable, shared. */
const wasm = createFixture(
  async () => {
    const wasmUrl = new URL("../src/pkg/castellan_wasm_bg.wasm", import.meta.url);
    const bytes = await Bun.file(wasmUrl).arrayBuffer();
    const module = new WebAssembly.Module(bytes);

    // Every import the module declares is a function the glue exports under
    // the same mangled name — the mapping is mechanical, not curated.
    type WasmFn = (...args: never[]) => unknown;
    const glueFns = glue as unknown as Record<string, WasmFn>;
    const importObject = {} as Record<string, Record<string, WasmFn>>;
    for (const { module: moduleName, name } of WebAssembly.Module.imports(module)) {
      const fn = glueFns[name];
      if (fn === undefined) {
        throw new Error(
          `wasm imports ${moduleName}.${name}, but the glue exports no such function`
        );
      }
      if (importObject[moduleName] === undefined) {
        importObject[moduleName] = {};
      }
      importObject[moduleName][name] = fn;
    }

    const { exports } = await WebAssembly.instantiate(module, importObject);
    glue.__wbg_set_wasm(exports);
    (exports as { __wbindgen_start?: () => void }).__wbindgen_start?.();
    return glue;
  },
  undefined,
  "file"
);

describe("@castellan/wasm (artifact)", () => {
  test("reports the protocol version of the build", () => {
    expect(wasm().protocol_version()).toBe(1);
  });

  test("parses an otpauth URI without exposing the secret", () => {
    const info = wasm().parse_otpauth("otpauth://totp/GitHub:octocat?secret=JBSWY3DPEHPK3PXP") as {
      issuer: string | null;
      account: string;
      digits: number;
      period: number;
    };
    expect(info.issuer).toBe("GitHub");
    expect(info.account).toBe("octocat");
    expect(info.digits).toBe(6);
    expect(info.period).toBe(30);
    expect("secret" in info).toBe(false);
  });

  test("applies the same origin rule the app enforces", () => {
    expect(wasm().origin_matches("login.example.org", "example.org")).toBe(true);
    expect(wasm().origin_matches("example.org", "login.example.org")).toBe(false);
    expect(wasm().origin_matches("evil.example.org", "login.example.org")).toBe(false);
  });
});

describe("@castellan/wasm (wrapper)", () => {
  test("the lazy loader fails with the readable message under a runtime without wasm-ESM", async () => {
    // Under bun the bundler-path import throws at module evaluation (see
    // the file header); the wrapper's contract is to convert that into the
    // one error message worth reading.
    await expect(loadWasm()).rejects.toThrow("@castellan/wasm: the WASM build is missing");
  });
});
