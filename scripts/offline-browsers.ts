#!/usr/bin/env bun
/**
 * offline-browsers.ts — provision Playwright's chromium from npm, for hosts
 * that cannot reach the Playwright CDN.
 *
 * The supported path is the root `@playwright/browser-chromium`
 * devDependency: `bun install` runs its install script (the package is in
 * `trustedDependencies`) and the browser lands in the user-level Playwright
 * cache. That download comes from `cdn.playwright.dev`, and in a sandbox or
 * a locked-down network it is simply unreachable — which takes out the ui
 * component tests and every e2e suite at once, i.e. most of the gate.
 *
 * This script is the fallback for exactly that case. It is not a second
 * supported setup: nothing in CI calls it, and on a normal machine it is a
 * no-op that prints "already provisioned".
 *
 * How it works: two npm packages ship a *binary inside the tarball* rather
 * than downloading one from a postinstall, so the npm registry is the only
 * host needed.
 *
 *   - `@sparticuz/chromium` — a Lambda-targeted chromium, brotli-compressed
 *     in the package, with its NSS/NSPR shared libraries beside it.
 *   - `@ffmpeg-installer/ffmpeg` — a static ffmpeg, which Playwright needs
 *     to encode the failure videos `e2ePreset` asks for.
 *
 * They are installed into a scratch directory under the user's cache (never
 * the repo and never the lockfile — this is a property of the *host*, not of
 * the project), then shimmed into the layout Playwright's registry expects:
 * a tiny launcher script at the executable path, which exports
 * LD_LIBRARY_PATH and execs the real binary. A launcher rather than a
 * symlink on purpose — it is also what makes Playwright's host-requirements
 * check pass, since `ldd` on a shell script reports no missing libraries.
 *
 * What this does NOT give you: **the extension e2e suite**. That build is a
 * headless shell — it has no extensions subsystem at all (no `extensions::`
 * strings, no `chrome-extension://` scheme), so `--load-extension` is
 * ignored and the MV3 service worker never registers. `packages/ui`'s
 * component tests and the desktop/mobile e2e suites do pass on it.
 *
 * Usage: `bun run browsers:offline` (add `--force` to re-shim over an
 * existing install).
 */

import { existsSync, mkdirSync, readFileSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { brotliDecompressSync } from "node:zlib";

/** The binaries, pinned the way the rest of the repo pins its toolchain. */
const PACKAGES = {
  "@sparticuz/chromium": "153.0.0",
  "@ffmpeg-installer/ffmpeg": "1.1.0"
} as const;

/**
 * Where Playwright looks for each executable inside its browser directory.
 * Mirrors `EXECUTABLE_PATHS` in playwright-core's registry for linux-x64;
 * the revisions are read from the installed playwright, not guessed, so a
 * version bump needs no edit here.
 */
const EXECUTABLES = {
  chromium: ["chrome-linux64", "chrome"],
  "chromium-headless-shell": ["chrome-headless-shell-linux64", "chrome-headless-shell"],
  ffmpeg: ["ffmpeg-linux"]
} as const;

const force = process.argv.includes("--force");
const cacheHome = process.env.XDG_CACHE_HOME || join(homedir(), ".cache");
const registryDir = process.env.PLAYWRIGHT_BROWSERS_PATH || join(cacheHome, "ms-playwright");
const scratch = join(cacheHome, "castellan-offline-browsers");

function fail(message: string): never {
  console.error(`offline-browsers: ${message}`);
  process.exit(1);
}

async function run(cmd: string[], cwd: string): Promise<void> {
  const proc = Bun.spawn(cmd, { cwd, stdout: "inherit", stderr: "inherit" });
  if ((await proc.exited) !== 0) fail(`\`${cmd.join(" ")}\` failed`);
}

/** The revisions of the browser build this repo's Playwright expects. */
function revisions(): Record<string, string> {
  // Resolved through the chain that actually exists in a bun workspace:
  // a package that depends on @playwright/test → playwright → playwright-core.
  const fromUi = createRequire(join(import.meta.dir, "../packages/ui/package.json"));
  const test = createRequire(fromUi.resolve("@playwright/test/package.json"));
  const core = createRequire(test.resolve("playwright/package.json")).resolve(
    "playwright-core/package.json"
  );
  const manifest = JSON.parse(readFileSync(join(dirname(core), "browsers.json"), "utf8")) as {
    browsers: { name: string; revision: string }[];
  };
  return Object.fromEntries(manifest.browsers.map((browser) => [browser.name, browser.revision]));
}

/** `chromium-headless-shell` + `1208` → `<registry>/chromium_headless_shell-1208`. */
const browserDir = (name: string, revision: string): string =>
  join(registryDir, `${name.replace(/-/g, "_")}-${revision}`);

async function main(): Promise<void> {
  if (process.platform !== "linux" || process.arch !== "x64")
    fail(`only linux-x64 is wired up here, this host is ${process.platform}-${process.arch}`);

  const revision = revisions();
  const targets = Object.entries(EXECUTABLES).map(([name, parts]) => ({
    name,
    path: join(browserDir(name, revision[name] ?? fail(`no revision for ${name}`)), ...parts)
  }));

  if (!force && targets.every((target) => existsSync(target.path))) {
    console.log("offline-browsers: already provisioned, nothing to do (--force to re-shim)");
    return;
  }

  mkdirSync(scratch, { recursive: true });
  writeFileSync(
    join(scratch, "package.json"),
    `${JSON.stringify(
      { name: "castellan-offline-browsers", private: true, dependencies: PACKAGES },
      null,
      2
    )}\n`
  );
  console.log(`offline-browsers: installing prebuilt binaries into ${scratch}`);
  // This is a cache, not a deliverable: no frozen lockfile, no workspace.
  await run([process.execPath, "install", "--no-summary"], scratch);
  await provision(revision);
}

/** Decompress what npm delivered and shim it into Playwright's registry. */
async function provision(revision: Record<string, string>): Promise<void> {
  const modules = join(scratch, "node_modules");

  // chromium: one brotli blob, plus the shared libraries it was linked
  // against (the host's own libnss3 is usually a different soname).
  const binary = join(scratch, "chromium");
  if (force || !existsSync(binary)) {
    writeFileSync(
      binary,
      brotliDecompressSync(readFileSync(join(modules, "@sparticuz/chromium/bin/chromium.br"))),
      { mode: 0o755 }
    );
    const tar = join(scratch, "libs.tar");
    writeFileSync(
      tar,
      brotliDecompressSync(readFileSync(join(modules, "@sparticuz/chromium/bin/al2023.tar.br")))
    );
    rmSync(join(scratch, "libs"), { force: true, recursive: true });
    mkdirSync(join(scratch, "libs"), { recursive: true });
    await run(["tar", "-xf", tar, "-C", join(scratch, "libs")], scratch);
    rmSync(tar);
  }

  const launcher = join(scratch, "launch.sh");
  writeFileSync(
    launcher,
    `#!/bin/sh
# Generated by scripts/offline-browsers.ts — the bundled chromium plus the
# NSS libraries it needs, as one executable Playwright can launch.
LD_LIBRARY_PATH="${join(scratch, "libs/lib")}\${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export LD_LIBRARY_PATH
exec "${binary}" "$@"
`,
    { mode: 0o755 }
  );

  for (const [name, parts] of Object.entries(EXECUTABLES)) {
    const dir = browserDir(name, revision[name] as string);
    const target = join(dir, ...parts);
    mkdirSync(dirname(target), { recursive: true });
    rmSync(target, { force: true });
    if (name === "ffmpeg") symlinkSync(join(modules, "@ffmpeg-installer/linux-x64/ffmpeg"), target);
    else writeFileSync(target, readFileSync(launcher), { mode: 0o755 });
    // Playwright refuses a browser directory without this marker.
    writeFileSync(join(dir, "INSTALLATION_COMPLETE"), "");
    console.log(`offline-browsers: ${name} → ${target}`);
  }

  console.log(
    "offline-browsers: done. `packages/ui` component tests and the desktop/mobile e2e\n" +
      "                  suites run on this build; the extension suite does not — a headless\n" +
      "                  shell cannot load an unpacked MV3 extension."
  );
}

await main();
