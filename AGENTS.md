# AGENTS.md — working in this repository

A short map for any agent (or human) picking this repo up cold. The full
architecture story is in the root README; this is the "don't break the
invariants" file.

## The layout

```
apps/       desktop, mobile (Tauri 2 + SvelteKit), extension (WXT, "Fob"),
            docs (the documentation site — Astro + Starlight)
crates/     the Rust workspace: per-crate @castellan/rust-* turbo packages
            (nextest/clippy per crate) + the @castellan/rust façade for the
            workspace-wide Cargo invocations
packages/   protocol (generated), core (client), ui (Svelte), wasm (wrapper),
            utils (tsconfig bases + the bun test fixtures)
```

## The invariants — check these before trusting a change

1. **One RPC contract.** Every operation is one entry in the
   `rpc_contract!` invocation in `crates/protocol`. That macro emits the Rust
   request/result types and face membership; xtask emits both
   `packages/protocol/src/generated` and the scoped clients under each app's
   `src/generated`. All are committed. Never hand-edit generated files—edit
   the contract, implement one arm in `crates/dispatch`, then run
   `pixi run codegen`.
2. **One logic, two targets.** Shared behavior (otpauth parsing, origin
   matching) lives in Rust crates and reaches the extension through
   `crates/wasm` → `packages/wasm`. If you are about to re-implement either
   in TypeScript, stop: the whole point is that the app and the extension
   cannot disagree.
3. **The client is transport-agnostic but face-scoped.** `packages/core`
   owns request ids, envelopes, result narrowing and errors; xtask-generated
   app clients expose only the contract operations assigned to that face.
   `@castellan/tauri` owns the shared desktop/mobile invoke adapter; native
   messaging stays in the extension. Neither belongs in `@castellan/core`.
4. **There is one native dispatcher.** `crates/dispatch` executes operations.
   Tauri commands and socket handlers are adapters, not a second match over
   `RpcMethod`; domain behavior never belongs in an app shell.
5. **Secrets stay app-side.** The extension never holds the database;
   `EntrySummary` carries no secret material, and WASM never links the vault
   crate.
6. **KDBX save is copy-aside-then-write.** keepass-rs writing is
   experimental; any save path must copy the old file first.
7. **`unsafe_code = "forbid"`** is a workspace lint. Removing it is a
   workspace-level decision, not a branch convenience.

## Commands (all from the repo root)

```console
$ pixi install                         # materialize the locked dev-tool environment
$ pixi run install                     # JS deps (bun only; no node anywhere) + hooks
$ pixi run gates                       # lint + typecheck + test, every language
$ pixi run codegen                     # regenerate types, clients + host manifests
$ pixi run codegen-check               # prove committed generated output is current
$ pixi run wasm                        # rebuild the WASM package
$ pixi run lint-workflows              # actionlint over .github/workflows
$ pixi run lint-commits                # convco: history is conventional (needs git)
$ pixi run hooks-install               # reinstall lefthook explicitly
$ pixi run dev-desktop                 # tauri dev (desktop)
$ pixi run dev-extension               # wxt dev (chromium)
$ pixi run dev-docs                    # Astro + Starlight docs site
$ pixi run docs-build                  # build the docs site (what CI runs)
$ pixi run e2e                         # desktop/mobile/extension browser e2e
$ pixi run storybook-build             # build the UI package's Storybook
$ pixi run bunx playwright-cli open --browser chromium <url>
                                       # agent-driven browser
$ pixi run cargo nextest run --workspace   # Rust suites directly (doc-tests:
$ pixi run cargo test --doc --workspace    # nextest does not run them)
$ pixi run xtask codegen
$ ./scripts/restore.sh                 # restore the offline environment
```

Pixi is the sole entry point for repository commands (decision-11). Named
high-frequency tasks form the command API; `pixi run bun …`, `pixi run bunx
…`, and `pixi run cargo …` cover one-off or package-scoped work without
escaping the locked environment.

## Conventions

- Conventional commits only (`feat:`, `fix:`…) — enforced by the commit-msg
  hook when the repository carries a `.git` (this workspace snapshot ships
  without one; `git init` + `pixi run hooks-install` brings the hooks up),
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
   deliberate, mutation-leaking trade for expensive immutable setup). A
  `createFixture` call registers its hooks against **the test file that
  evaluates it**, and an imported module evaluates once however many files
  import it — so a fixture defined at a shared `support.ts`'s top level
  wires itself into one file and every other file reads it before setup and
  throws. Share the *factory*, let each test file call it
  (`packages/core/test/support.ts`, with its two-file regression in
  `packages/utils/test/fixtures-across-files-*.test.ts`). A new
  discrete example gets a `#[case]`; a new "for any X" claim gets a
  property. If proptest finds a real bug, commit the seed under
  `proptest-regressions/` so the case replays for everyone; if the input
  was out of domain, fix the strategy and delete the seed. fast-check
  prints the failing seed itself — re-run with `{ seed }`; `bun test
  --seed` does not reach it.
- **Case files are generated, not committed.** A test case the toolchain
  can author itself is a row in a case table plus a factory that builds
  it — never a committed file. The worked example is
  `crates/vault/tests/common/mod.rs`: twelve committed fixture binaries
  (the cipher/KDF/keyfile matrix) became `GENERATED_CORPUS` + `build()`,
  a reviewed table whose expectations are exact by construction, because
  the generator wrote every title it asserts — and a config-matrix case
  added later is a table row, not another binary in the repo. A committed
  case file is justified only by what no generator can author: external
  authorship (bytes another client wrote — the KeePassXC 2.7.12 anchor),
  or a format the toolchain cannot write (KDBX 3.1; keepass-rs emits 4.1
  only). Before adding a fixture file, name what it carries that the
  factory could not; if the answer is nothing, it is a case, not a file.
- **Styling is UnoCSS, from a foundation plus a look.** `@castellan/utils/uno`
  exports `unoPreset()`: the tokens (`--bg`, `--accent`…, reachable as
  `bg-bg`/`text-accent` and overridable per subtree), the `*-tint-<n>`
  currentColor mixes, the optional `html`/`body` shell, and the extraction
  pipeline. `@castellan/ui/uno` exports `presetUi()`: the component looks, as
  shortcuts (`c-entry-row`, `c-action`, `c-badge`, `c-field`,
  `c-section-title`, `c-ring-*`), kept with the components they describe. A
  consumer's `uno.config.ts` is one expression around them —
  `export default unoPreset({ presets: [presetUi()] })` in the two faces and
  the library's harnesses, plain `unoPreset({ shell: false })` in the
  extension popup, which renders no shared components and so ships none of
  their looks. Put a new recurring look in `packages/ui/uno.ts` (at the
  package *root*: `src/**` is scanned by the extractor, and a shortcut table
  under it would ship every utility it names); put a one-off size in the
  markup. The *metrics* — padding, widths, corner radius — always stay in the
  markup, because that is exactly where the faces legitimately disagree. Two
  rules to respect: never put a property in a shortcut that a face also sets
  inline (two utilities for one property leave the winner to CSS source
  order), and keep the bare semantic class (`username`, `badge`, `status`,
  `error`) as the first class on an element — it carries no CSS and exists so
  the component tests and e2e suites have a hook that design changes cannot
  break. The extraction pipeline is configured to scan `packages/ui/src` as
  well as the app's own source, because the shared components are consumed as
  source and their classes are built by *the app's* UnoCSS pass — if that
  include ever stops matching, components render unstyled in the apps while
  looking perfect in Storybook (`packages/utils/test/uno.test.ts` guards the
  pipeline and the preset seam; `packages/ui/tests/styling.test.ts` reads the
  looks back out of a real browser).
- **The `@` alias** maps to each workspace member's own root (`@/src/fixtures`),
  wired the way each stack wants it: packages declare tsconfig `paths`,
  SvelteKit apps use `kit.alias` (so tsc *and* vite learn it), WXT generates
  it, Astro reads tsconfig paths. Use it freely in tests and in app source.
  **Never in a shared package's `src/`** — that source is bundled by the
  *consuming* app's bundler, whose `@` points at the app, not the package;
  the import would resolve to the wrong files at the consumer's build, not
  yours. Package source keeps `./`-relative imports. **Enforced by
  `style/noRestrictedImports`** (biome.json): any import that climbs with
  `../` is an error, and `packages/*/src/**` is the one override where the
  rule is off, because that is exactly where `@` is forbidden. If the alias
  does not resolve somewhere, teach that bundler instead of climbing —
  tsconfig `paths` only convinces tsc, so vite-driven harnesses need their
  own `resolve.alias` (`packages/ui`'s `ctViteConfig` is the worked
  example). Runtime path arithmetic — `new URL("../.output/…",
  import.meta.url)` in the extension's e2e fixture — is not an import and
  is not affected: no bundler resolves it, so there is no alias to apply.
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
  its postinstall) — a fresh clone is testable after `pixi run install`,
  nobody runs `playwright install` by hand. E2e runs the two SvelteKit
  faces through their dev servers (desktop 5173, mobile 5174) and loads
  the *built* extension into full Chromium (`channel: "chromium"` — the
  default headless shell cannot load extensions).
- **If `cdn.playwright.dev` is unreachable, run `pixi run bun run browsers:offline`.**
  `scripts/offline-browsers.ts` provisions chromium and ffmpeg from npm
  packages that ship the binary *inside the tarball*
  (`@sparticuz/chromium`, `@ffmpeg-installer/ffmpeg`) into the user cache,
  shimmed into Playwright's registry layout. It is a host-level fallback,
  not a second supported setup: never add those packages to the workspace,
  nothing in CI calls it, and on a healthy machine it prints "already
  provisioned". It unblocks the ui component tests and the desktop/mobile
  e2e suites; it cannot unblock the **extension** suite, because that
  chromium is a headless shell with no extensions subsystem at all.
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
  one. UI implementation work should also consult the Anthropic
  `frontend-design` skill for visual direction and the `webapp-testing`
  skill when a native Playwright script is a better fit than the CLI.
- Adding a crate: directory under `crates/` with its `Cargo.toml` (the glob
  picks it up), a line in root `[workspace.dependencies]`, a row in
  `crates/README.md`. Adding a package: directory under `packages/`, added
  to nothing (workspaces glob), `pixi run install` links it.
- Adding a docs page: an MDX file under `apps/docs/src/content/docs/` plus a
  sidebar entry in `apps/docs/astro.config.mjs` — the sidebar is hand-written
  because a generated one documents the file tree, not the reading order.
  The docs site is the `apps/docs/` bun workspace; its scripts are forced onto
  bun's runtime (`bun --bun`), so Astro never falls back to whatever node
  happens to be on PATH.
- **Implementing a change starts in the local skills, not the code.** Two of
  the vendored skills under `.agents/skills/` are process contracts, not
  reference shelf-ware: before writing any code for a change, consult
  `tdd/SKILL.md` (the red → green loop, seams agreed up front, the
  anti-patterns that make tests worthless) and, whenever the change
  restructures existing code without changing behavior,
  `refactor/SKILL.md` (small steps, tests green after each, behavior
  preserved). They are local on purpose — read them from disk rather
  than from memory, because a skill update lands as a reviewable commit
  like any other. New work goes test-first at a seam; restructuring goes
  through the refactor checklist; both end with the gates, not before.
- Adding an agent skill: `bun x skills add <source>` writes into
  `.agents/skills/` and pins it in `skills-lock.json`; commit both, the
  same way `bun.lock` is committed. The repo already vendors project skills
  for backlog/session/refactor/TDD/browser exploration and the UI loop
  (`frontend-design`, `webapp-testing`), so prefer updating that set over
  scattering agent-specific instructions elsewhere.
- Toolchain: rust pinned by `rust-toolchain.toml`, bun by `packageManager`.
  `pixi.toml` **mirrors** both pins and must move with them (conda's cargo is
  not a rustup proxy, so the pin file does not re-pin inside `pixi run`).
  Do not suggest npx/pnpm/cargo-from-path; hooks and CI assume exactly these.
  `pixi install` (decision-8) provisions every tool the gates call — bun, rust
  with clippy+rustfmt, the wasm32 std, wasm-pack, wasm-bindgen-cli, convco,
  actionlint, cargo-deny, cargo-nextest, cargo-llvm-cov — pinned through the
  committed `pixi.lock`, split across three features (`rust`, `web`, `utils`)
  so a toolchain change is a reviewable diff on its own. Pixi manages **tools
  only**, but it is the sole command entry point (decision-11): named tasks
  delegate into `bun run` and Cargo, while generic `bun`, `bunx`, and `cargo`
  tasks handle one-offs. **CI installs that same environment** (decision-10:
  `setup-pixi`, `--locked`) but does not activate it globally; every repository
  step says `pixi run`, so a missing boundary is visible in review and cannot
  pick up a runner-global binary.
- **The offline airlock is `./scripts/restore.sh`** (generated by `pixi-sandbox
  init` — regenerate it, never edit it). On **0.4.0 it did not finish the job**:
  it `git archive`d `origin/sandbox/developer-<platform>`, which a clone whose
  fetch refspec covers only `main` does not have, and it wired the vendored
  crates by writing `.cargo/config.toml` — then found the tracked one carrying
  `TS_RS_EXPORT_DIR`, rightly left it alone, and still printed `restore
  complete`, leaving 489 crates in `.pixi-sandbox/vendor/` with nothing pointing
  at them. **Both halves were fixed upstream, not here**
  ([pixi-sandbox#55](https://github.com/Archont561/pixi-sandbox/issues/55),
  closed): the repo deliberately carries no wrapper script, so the whole
  adoption was bumping `PIXI_SANDBOX_VERSION` in `publish-sandbox.yml` — done,
  the pinned CLI is **0.5.3**. The launcher now fetches the missing sandbox ref
  itself (opt out with `PIXI_SANDBOX_FETCH=skip`), and `restore` defaults to
  `--cargo-config auto`: it writes `.pixi-sandbox/cargo-home/config.toml` plus
  Pixi activation hooks and leaves the tracked `.cargo/config.toml` alone. One
  command, no manual `CARGO_HOME`:
  ```console
  $ ./scripts/restore.sh
  ```
  Do **not** "fix" anything here by committing the `[source]` block into
  `.cargo/config.toml`: source replacement pointing at a directory that exists
  only after a restore breaks CI and every networked contributor. `cargo
  --config key=value`, `CARGO_SOURCE_*` and `pixi run` do not help either — the
  first two are not config *files*, and `pixi run` chooses the cargo binary, not
  where it looks for sources.
- Hooks and repo linters: **lefthook** is a devDependency, installed by the
  guarded `prepare` script (skips silently where there is no `.git`, so
  `pixi run install` never breaks in a snapshot/zip export). Every hook body
  enters through `pixi run`; the commit-msg hook therefore always uses the
  locked **convco** rather than a PATH-dependent fallback. Accepted types are
  pinned in `.versionrc`. **actionlint** is also supplied by the pixi
  environment and reached through `pixi run lint-workflows` (explicit paths
  let it lint a snapshot export with no `.git`). Neither tool ships a usable
  npm CLI, which is one reason bypassing pixi is unsupported.

## Reading a failed CI run

Neither `gh run view --log` nor `gh api .../actions/jobs/<id>/logs` works
from a sandboxed agent: `gh` follows the redirect to
`productionresultssa*.blob.core.windows.net` itself and dies with `EOF`.
The workflow's annotation step narrows the common cases — a panicking
test, a bun `(fail)`, a cargo `error:` — into check-run annotations, but
a failure outside those patterns still reads as nothing but `Process
completed with exit code N`. **Do not answer this by pushing a throwaway
diagnostic workflow that re-runs the failing command and echoes
`::error::`.** It costs a five-minute round trip per question, truncates
at the annotation size limit, and the real log was reachable the whole
time.

`gh` prints the signed blob URL inside its own error message. Take it and
retrieve it with a plain HTTP fetch — an agent's web-fetch tool resolves
outside the sandbox, where the blob store is reachable:

```console
$ JOB=$(gh run view <run-id> --json jobs --jq '.jobs[0].databaseId')
$ gh api -i "repos/Archont561/castellan/actions/jobs/$JOB/logs" 2>&1 \
    | grep -o 'https://[^"]*job-logs.txt[^"]*'
```

Fetching that URL returns the complete, uncut job log — every step in
order, timestamped. The signature expires roughly ten minutes after it
is minted, so fetch promptly and re-mint rather than reusing a stale
URL.

Read in this order: the check-run annotations first — the workflow's
`Surface test failures as annotations` step distills panicking tests,
bun `(fail)` lines, and cargo `error:` lines into the check result, so
`gh pr checks` plus one annotations call often answers without any
fetch — then the raw log for what a digest cannot carry: which step,
how long it ran, what the compiler said verbatim. And when the log
spans dozens of chunks, do not page from the top: the fetcher reports
the chunk total, a failing gate's output sits in the last chunk (every
step after it was skipped), and the ten-minute signature will not
outlast a sequential walk. Jump straight to the end, and binary-search
backwards by the timestamp each chunk opens with when the failure's
first line matters.

## Planning and knowledge — two systems, one boundary

- **`backlog/`** is delivery state, in the [backlog.md](https://backlog.md)
  format (devDep `backlog.md`, run `pixi run backlog` for the board). Tasks
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

<!-- BEGIN:turborepo-agent-rules -->

# This is NOT the Turborepo you know

Turborepo configuration, task behavior, and CLI commands can vary between installed versions and may differ from your training data. Resolve the `turbo` package from this file's directory or relevant workspace; in monorepos, it may not be visible from the repository root. For example, run `node -p "require.resolve('turbo/package.json')"` from a workspace that depends on `turbo`.

Read `docs/README.md` inside that installed package first, then read the relevant pages from its `docs/` directory before changing Turborepo configuration or commands. Heed deprecation notices. These bundled docs match the installed package version and are available without network access.

This block is written and re-added by `turbo` before repository-scoped commands when an AI agent is detected. In the Turborepo source repository, its template is defined in `crates/turborepo-cli/src/cli/agent_guidance.rs`. Removing the managed block while updates are enabled means a later qualifying invocation will add it again. Set `"agentGuidance": false` in the root `turbo.json` or `turbo.jsonc` to opt out; this does not remove an existing block. Keep the block committed with your work to avoid an uncommitted change on the next agent invocation.
<!-- END:turborepo-agent-rules -->
