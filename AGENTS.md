# AGENTS.md — working in this repository

A short map for any agent (or human) picking this repo up cold. The full
architecture story is in the root README; this is the "don't break the
invariants" file.

## The layout

```
apps/       desktop, mobile (Tauri 2 + SvelteKit), extension (WXT, "Fob"),
            docs (the documentation site — Astro + Starlight)
crates/     the Rust workspace + the @castellan/rust turbo façade
packages/   protocol (generated), core (client), ui (Svelte), wasm (wrapper),
            utils (tsconfig bases + the bun test fixtures)
```

## The invariants — check these before trusting a change

1. **One protocol.** Every message any face sends is a type in
   `crates/protocol`. TypeScript types in `packages/protocol/src/generated`
   are *generated* — `cargo run -p castellan-xtask -- codegen` — and
   committed. Never hand-edit a file in there; edit the Rust, regenerate.
2. **One logic, two targets.** Shared behavior (otpauth parsing, origin
   matching) lives in Rust crates and reaches the extension through
   `crates/wasm` → `packages/wasm`. If you are about to re-implement either
   in TypeScript, stop: the whole point is that the app and the extension
   cannot disagree.
3. **The client is transport-agnostic.** `packages/core` builds requests and
   narrows results; transports (Tauri invoke, native messaging) live in the
   apps. Don't add a dependency to `@castellan/core` to serve one face.
4. **Secrets stay app-side.** The extension never holds the database;
   `EntrySummary` carries no secret material, and WASM never links the vault
   crate.
5. **KDBX save is copy-aside-then-write.** keepass-rs writing is
   experimental; any save path must copy the old file first.
6. **`unsafe_code = "forbid"`** is a workspace lint. Removing it is a
   workspace-level decision, not a branch convenience.

## Commands (all from the repo root)

```console
$ pixi install               # optional: the whole dev-tool env in one command
$ bun install                # JS deps (bun only; no node anywhere) + hooks
$ bun run gates              # lint + typecheck + test, every language
$ bun run codegen            # regenerate packages/protocol/src/generated
$ bun run wasm               # rebuild the WASM package
$ bun run lint:workflows     # actionlint over .github/workflows
$ bun run lint:commits       # convco: history is conventional (needs git)
$ bun run hooks:install      # lefthook install, if bun install ran without git
$ bun run dev:desktop        # tauri dev (desktop)
$ bun run dev:ext            # wxt dev (chromium)
$ bun run dev:docs           # the docs site (apps/docs/, Astro + Starlight)
$ bun run docs:build         # build the docs site (what CI runs)
$ bun run all:e2e            # browser e2e: desktop, mobile, extension — serial
$ bun run all:storybook      # build the ui package's Storybook (static)
$ bunx playwright-cli open --browser chromium <url>
                             # agent-driven browser; snapshots land in
                             # .playwright-cli/ as YAML with element refs
$ bunx playwright-cli install-browser chromium   # once per machine, for the
                             # agent CLI only — the *test* browsers come from
                             # bun install via @playwright/browser-chromium
$ cargo test --workspace     # rust suites directly
$ cargo run -p castellan-xtask -- codegen
```

Any of these also runs with the dev-tool env on PATH through pixi's one
generic task: `pixi run bun run <script>` — the task table deliberately has
no per-script wrappers, so a new script never needs a pixi line (the
`xtask` task is the only other one: `pixi run xtask <subcommand>`).

## Conventions

- Conventional commits only (`feat:`, `fix:`…) — enforced by the commit-msg
  hook when the repository carries a `.git` (this workspace snapshot ships
  without one; `git init` + `bun run hooks:install` brings the hooks up),
  changelogs are generated from them via convco (`.versionrc`).
- Comments explain *why*, and reference the tradeoff that was made. The
  geoquery/pixi-sandbox house style: a comment that restates the code is
  deleted on sight.
- Rust: workspace manifests inherit (`*.workspace = true`); new deps get
  added to the root `[workspace.dependencies]` with a reason, never inline
  in a crate. TS: one set of config bases in `@castellan/utils` — packages
  extend `@castellan/utils/tsconfig/lib.json`, apps extend an *array* of
  their framework-generated config and `…/app.json` (the generated config
  owns paths and ambient types, the base wins on strictness) — no
  per-package compiler config drift.
- Tests live in a `tests/` directory next to the code they prove — Rust
  crates as integration tests in `crates/<name>/tests/*.rs` over the
  public API only (a bin crate like xtask keeps its few unit tests inline —
  a bin exports nothing an integration test could import; the ts-rs export
  checks are derive-generated), TypeScript as `*.test.ts` in the package's
  `test/` under `bun test`. The two sides share one vocabulary: **rstest**
  ↔ `createFixture` from `@castellan/utils/fixtures`, **proptest** ↔
  **fast-check** (root devDep). Rust uses the two dev-only frameworks from
  `[workspace.dependencies]`: `#[fixture]` for shared setups (the sample
  vault — built through the crate's own front door, save → open), `#[case]`
  matrices for behavior with many discrete
  shapes, proptest for invariants over generated inputs (round-trips,
  boundary arithmetic) — which is how the backlog specifies its hardest
  tests (task-8's KDBX round-trip harness, task-27's sync idempotence).
  TypeScript properties wrap `fc.assert(fc.property(...))` in a `test()`,
  `fc.asyncProperty` when the body awaits; expensive properties pin
  `numRuns` (50–200); fixtures default to per-test scope (`"file"` is the
  deliberate, mutation-leaking trade for expensive immutable setup). A new
  discrete example gets a `#[case]`; a new "for any X" claim gets a
  property. If proptest finds a real bug, commit the seed under
  `proptest-regressions/` so the case replays for everyone; if the input
  was out of domain, fix the strategy and delete the seed. fast-check
  prints the failing seed itself — re-run with `{ seed }`; `bun test
  --seed` does not reach it.
- **The `@` alias** maps to each workspace member's own root (`@/src/fixtures`),
  wired the way each stack wants it: packages declare tsconfig `paths`,
  SvelteKit apps use `kit.alias` (so tsc *and* vite learn it), WXT generates
  it, Astro reads tsconfig paths. Use it freely in tests and in app source.
  **Never in a shared package's `src/`** — that source is bundled by the
  *consuming* app's bundler, whose `@` points at the app, not the package;
  the import would resolve to the wrong files at the consumer's build, not
  yours. Package source keeps `./`-relative imports.
- **Library packages build with bunup**, configured once: every
  `bunup.config.ts` is two lines around `libPreset` from
  `@castellan/utils/bunup` (ESM-only, d.ts, sourcemaps, clean `dist/`,
  workspace deps external). The artifact is proof today — consumers still
  import `src/` directly — and the publish target the day a package ships,
  which is why bunup's `exports` auto-sync stays off. `wasm` and `ui` keep
  their own builds: one is a Rust artifact (wasm-pack), the other is
  Svelte compiled inside the apps that consume it.
- **Browser tests are one Playwright, one chromium.** Every playwright
  package in the repo is pinned to the same version (1.58.2 — the last
  line of `@playwright/experimental-ct-svelte`, which the ui component
  tests ride); a mismatch between `@playwright/test` and the browser
  package refuses to launch, so the pins move together or not at all.
  The chromium binary is a root devDependency
  (`@playwright/browser-chromium`, in `trustedDependencies` so bun runs
  its postinstall) — a fresh clone is testable after `bun install`,
  nobody runs `playwright install` by hand. E2e runs the two SvelteKit
  faces through their dev servers (desktop 5173, mobile 5174) and loads
  the *built* extension into full Chromium (`channel: "chromium"` — the
  default headless shell cannot load extensions).
- **`mount()` in component tests resolves to the component's root
  element**, not a wrapper — assert and click the returned locator
  directly, and query only the parts inside it as descendants. Role
  locators never match the element they are called on.
- **Stories are `.stories.svelte` files next to their components**,
  written with `defineMeta` from `@storybook/addon-svelte-csf` (the
  `<Meta>` form is the legacy API the indexer ignores — a story that
  uses it builds fine and silently indexes zero). Storybook's builder
  reads `packages/ui/vite.config.ts` for the Svelte plugin; without that
  file the preview build fails on the renderer's own `.svelte` files.
- **The extension's vite build must resolve Svelte's *client* runtime.**
  `wxt.config.ts` pins `resolve.conditions: ["browser"]`: without it WXT
  bundles the server entry, where `mount()` is a throwing stub — the
  build succeeds and the popup renders nothing. The e2e suite is what
  catches this class of bug; keep it in the loop.
- **Agent browser exploration uses `playwright-cli`** (`@playwright/cli`,
  skill in `.agents/skills/playwright-cli`, installed via
  `skills-lock.json`): `open --browser chromium`, then `snapshot` /
  `screenshot` / `click <ref>` — snapshots go to disk (`.playwright-cli/`,
  gitignored), not into context. It ships its own playwright, so its
  browser comes from `install-browser chromium`, not the repo's pinned
  one.
- Adding a crate: directory under `crates/` with its `Cargo.toml` (the glob
  picks it up), a line in root `[workspace.dependencies]`, a row in
  `crates/README.md`. Adding a package: directory under `packages/`, added
  to nothing (workspaces glob), `bun install` links it.
- Adding a docs page: an MDX file under `apps/docs/src/content/docs/` plus a
  sidebar entry in `apps/docs/astro.config.mjs` — the sidebar is hand-written
  because a generated one documents the file tree, not the reading order.
  The docs site is the `apps/docs/` bun workspace; its scripts are forced onto
  bun's runtime (`bun --bun`), so Astro never falls back to whatever node
  happens to be on PATH.
- Adding an agent skill: `bun x skills add <source>` writes into
  `.agents/skills/` and pins it in `skills-lock.json`; commit both, the
  same way `bun.lock` is committed. The repo ships the CLI (devDependency)
  with no skills vendored yet — capability, not content.
- Toolchain: rust pinned by `rust-toolchain.toml`, bun by `packageManager`.
  `pixi.toml` **mirrors** both pins and must move with them (conda's cargo is
  not a rustup proxy, so the pin file does not re-pin inside `pixi run`).
  Do not suggest npx/pnpm/cargo-from-path; hooks and CI assume exactly these.
  `pixi install` (decision-8) provisions every tool the gates call — bun, rust
  with clippy+rustfmt, the wasm32 std, wasm-pack, wasm-bindgen-cli, convco,
  actionlint, cargo-deny, cargo-nextest, cargo-llvm-cov — pinned through the
  committed `pixi.lock`, split across three features (`rust`, `web`, `utils`)
  so a toolchain change is a reviewable diff on its own. Pixi manages **tools
  only**: scripts stay behind `bun run` (reached from the env as `pixi run
  bun run <script>` — the task table has no per-script wrappers). **CI
  installs that same environment** (decision-10: `setup-pixi`, `--locked`,
  activated onto `PATH`), so a tool is added in one place and a stale
  `pixi.lock` fails the gate instead of drifting; contributors who prefer
  rustup + bun.sh installs get the same binaries, provisioned differently.
- Hooks and repo linters: **lefthook** is a devDependency, installed by the
  guarded `prepare` script (skips silently where there is no `.git`, so
  `bun install` never breaks in a snapshot/zip export). The commit-msg hook
  uses **convco** when it is on PATH (`cargo install convco --locked` or a
  [release binary](https://github.com/convco/convco/releases); its accepted
  types are pinned in `.versionrc`, matching the grep fallback exactly) and
  falls back to a grep of the same rule — convco's verdict is final, the
  grep is not an appeal court. **actionlint** lints the workflows
  (`bun run lint:workflows`, explicit paths so it runs from a snapshot
  export with no `.git`; install from
  [its releases](https://github.com/rhysd/actionlint/releases)). Neither
  ships a usable npm CLI — the pixi env carries both (`pixi run bun run
  lint:workflows`, `pixi run bun run lint:commits`), or install release
  binaries like cargo-deny's.

## Planning and knowledge — two systems, one boundary

- **`backlog/`** is delivery state, in the [backlog.md](https://backlog.md)
  format (devDep `backlog.md`, run `bun run backlog` for the board). Tasks
  (`tasks/`), decisions (`decisions/`), milestones (`milestones/`), docs
  (`apps/docs/`). A task's acceptance criteria are the contract — implement
  against them, tick them, and keep `status` honest. Decisions are append-only
  records: superseding one means a new `decision-N` file, not an edit.
- **`.knowledge/`** is durable knowledge in the
  [Open Knowledge Format v0.2](https://github.com/GoogleCloudPlatform/knowledge-catalog)
  (GoogleCloudPlatform/knowledge-catalog): architecture, contracts,
  rationale, research. Every concept file carries typed YAML front matter
  (`id`, `category`, `status`…); match the shape of the files already
  there, and bump `updated` when you change one. Nothing machine-checks
  this — the conformance validator that used to was removed, so review is
  the only gate.
- **The boundary rule**: task checklists, sequencing, and status live in
  `backlog/`; the *why*, the contracts, and the research live in
  `.knowledge/`. Never a task list inside the knowledge bundle, never
  architecture rationale buried in a task. If a change has both, it gets
  both — cross-linked by path.
- Updating knowledge: edit the concept, bump `updated`, add a line to
  `.knowledge/log.md` (newest first, `* **Update**: …`). New concepts need a
  front matter block (copy a neighbor's), an entry in the category
  `index.md`, and a log line — the validator enforces the shape.

## What is deliberately not here yet

The IPC socket server in the desktop app (the half that talks to
`castellan-native-host`), passkey interception bodies, the vault save path,
and every Tauri mobile plugin. Their homes are named in the code comments
(`TODO(ipc)`, `TODO(passkeys)`) — implement in place; don't restructure
around them.
