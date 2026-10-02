# Castellan

Local-first secrets manager: a KeePass-compatible vault, an authenticator,
and a soft security key, in one Rust core with three faces — desktop, mobile,
and a browser extension — that all speak one protocol and share one logic.

Monorepo setup **copied and adjusted from
[Archont561/geoquery](https://github.com/Archont561/geoquery) and
[Archont561/pixi-sandbox](https://github.com/Archont561/pixi-sandbox)** (see
[What came from where](#what-came-from-where) for the exact ledger).

```
┌────────────┐  ┌────────────┐  ┌──────────────────────┐
│   desktop  │  │   mobile   │  │  extension ("Fob")   │
│ Tauri+Kit  │  │ Tauri+Kit  │  │  WXT, all browsers   │
└─────┬──────┘  └─────┬──────┘  └──────────┬───────────┘
      │ invoke("rpc") │    native messaging │ (same protocol, same framing)
      └───────┬───────┴───────────┬─────────┘
              ▼                   ▼
   ┌──────────────────────────────────────────┐
   │  crates: protocol · vault · otp · ipc    │  ← one Rust workspace
   │  native-host · wasm · xtask              │
   └──────┬───────────────────────┬───────────┘
          │ ts-rs (codegen)       │ wasm-pack
          ▼                       ▼
  packages/protocol         packages/wasm
  (generated types)         (shared logic in the browser)
          │                       │
          └─────► packages/core ◄─┘   (one client, any transport)
```

## The promise this repo is built around

**Same scoped logic in every face.** A message is defined once (Rust), the
TypeScript that describes it is derived (`ts-rs` → generated + committed),
and behavior that must not drift — otpauth parsing, origin matching — is
written once in Rust and either linked natively (apps) or compiled to WASM
(extension). The KeePass ecosystem's worst failure modes are two halves
disagreeing about a shape; this structure makes that disagreement a compile
error instead of a support ticket.

## Layout

| Path | What it is |
| --- | --- |
| `apps/desktop` | Tauri 2 + SvelteKit (static adapter, `ssr=false`) — the desktop face |
| `apps/mobile` | Tauri 2 (iOS/Android) + SvelteKit — the mobile face; same dispatcher |
| `apps/extension` | WXT extension, one codebase for chromium + gecko; native-messaging transport; passkey interception skeleton at `document_start`, MAIN world |
| `apps/docs` | this project's documentation site (Astro + Starlight, a bun workspace) |
| `crates/protocol` | the one message language; ts-rs derives the TS bindings |
| `crates/otp` | otpauth parsing + RFC 6238 TOTP (with the RFC's own test vectors) |
| `crates/vault` | KDBX core: open, entry projection, passphrase generation |
| `crates/ipc` | the one native channel: framing + well-known socket path (std-only) |
| `crates/native-host` | the byte pump each browser spawns (the app binary, `--native-host`) |
| `crates/wasm` | the WASM face of shared logic; `packages/wasm` wraps it |
| `crates/xtask` | codegen: Rust types → `packages/protocol/src/generated` |
| `packages/protocol` | generated TS types (committed; regenerate with `bun run codegen`) |
| `packages/core` | the transport-agnostic client + the `Transport` seam |
| `packages/ui` | shared Svelte components (entry row, TOTP ring, lock shield) — with Storybook stories and Playwright component tests beside them |
| `packages/wasm` | npm wrapper around the wasm-pack output |
| `packages/utils` | the shared tsconfig bases + the bun test fixtures (`createFixture`) + the build/test presets (bunup `libPreset`, playwright `e2ePreset`) |

## Quickstart

Prerequisites, either way: [bun](https://bun.sh) 1.3.x (the only JS runtime
— no node anywhere) and Rust with the pinned toolchain
(`rust-toolchain.toml` — rustup picks it up automatically), plus for Linux
desktop builds the [Tauri system libraries](apps/desktop/README.md).

The one-command alternative is [pixi](https://pixi.sh) (decision-8):
`pixi install` provisions that whole toolchain — bun, rust with clippy
and rustfmt, the wasm32 std, wasm-pack, convco, actionlint, cargo-deny,
cargo-nextest, cargo-llvm-cov — pinned through the committed `pixi.lock`.
Pixi manages tools only; scripts stay behind `bun run` either way.

```console
$ pixi install                 # optional: the whole dev-tool env at once
$ bun install                  # link the workspace
$ bun run codegen              # derive TS types from the Rust protocol
$ bun run wasm                 # build the shared-logic WASM package
$ bun run gates                # lint + typecheck + test, every language
$ cargo test --workspace       # or drive Rust directly
```

Faces:

```console
$ bun run dev:desktop          # tauri dev (apps/desktop)
$ bun run dev:mobile           # tauri dev (apps/mobile; see its README for android/ios init)
$ bun run dev:ext              # wxt dev (chromium; :firefox for gecko)
$ bun run dev:docs             # the documentation site (apps/docs/, Astro + Starlight)
```

Validated in this scaffold: `cargo check`/`cargo test` green across all
library crates (64 tests — rstest fixtures/matrices and proptest properties
included — zero warnings under `missing_docs` + `clippy::all`), cargo-deny
clean (bans/licenses/sources/advisories, with one documented transitive
unmaintained-crate ignore), the full TS gate green (biome, tsc/svelte-check
on every package and app through the `@castellan/utils` bases, 38 bun tests
including fast-check round-trip and client properties), actionlint clean on
the workflows, the docs site building (10 pages), and the
codegen pipeline producing the committed `packages/protocol/src/generated`.
The Tauri app crates need the platform webkit libraries to compile
(documented in `apps/desktop/README.md`).

## The task graph

`turbo` runs the monorepo; `bun run <verb>` at the root fans out to every
package. The Rust workspace is **one node** in that graph, through the
façade in `crates/package.json` (`@castellan/rust`) whose scripts `cd ..`
and run cargo — so `bun run all:test` runs the TS suites *and*
`cargo test --workspace`, exactly once, in the order the graph says.
Codegen is a task too: `@castellan/protocol`'s typecheck depends on
`@castellan/rust#codegen`, which means a stale generated directory fails the
build instead of a review.

The library packages (protocol, core, utils) build with **bunup** through
one shared preset — `libPreset` in `@castellan/utils/bunup` — ESM plus
declarations into a gitignored `dist/`: proof today that each package
bundles and emits types, and the publish artifact the day one ships.
Workspace consumers still import `src/` directly (the dev loop stays
rebuild-free), so bunup's `exports` auto-sync stays off. The `wasm` and
`ui` packages keep their own builds: wasm-pack builds Rust, and Svelte
components are compiled inside the apps that consume them.

Browser tests ride the same graph, on one pinned Playwright (1.58.2,
aligned with `@playwright/experimental-ct-svelte` for the ui package's
component tests) and one prebundled chromium: `@playwright/browser-chromium`
at the root downloads the binary during `bun install`, so e2e needs no
manual browser step. Each app's `e2e` task drives its face — desktop and
mobile through their dev servers, the extension loaded *built* into full
Chromium — and the ui components have Storybook stories (`defineMeta`,
`.stories.svelte` next to each component) built by
`bun run all:storybook`.

## Codegen: how the two languages stay one protocol

```console
$ cargo run -p castellan-xtask -- codegen
```

`castellan-xtask` loads `castellan-protocol`'s types and exports each one
(ts-rs) into `packages/protocol/src/generated/`, writes the barrel
`index.ts`, and writes `version.ts` from the Rust `PROTOCOL_VERSION`
constant. The output is committed: a protocol change shows up in review as
exactly the diff of the wire, and a stale generation is visible, not
latent. Hand edits there are overwritten by design.

## What came from where

Copied from **geoquery** (the closer of the two references):

| Pattern | Where |
| --- | --- |
| bun as the only JS runtime (`packageManager` pinned, turbo + biome as workspace devDeps, no node) | root `package.json` |
| the `crates/package.json` **turbo façade** — Rust workspace as one task-graph node, scripts `cd ..` to the workspace root, `cache: false` per task | `crates/package.json`, `crates/turbo.json` |
| ts-rs codegen through an `xtask` crate into `packages/*/src/generated` | `crates/xtask` |
| root `[workspace.dependencies]` with documented version ranges; `[workspace.package]` metadata inheritance; `[workspace.lints]` (`unsafe_code = "forbid"`, `missing_docs`, clippy `all`) | root `Cargo.toml` |
| strict shared TypeScript bases (ES2023, `noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, `verbatimModuleSyntax`, `isolatedModules`) extended through package exports | `packages/utils/tsconfig/` |
| biome config (vcs-aware, double quotes, no trailing commas, lineWidth 100) | `biome.json` |
| lefthook hooks where every body is the command CI runs, `stage_fixed`, conventional-commit `commit-msg` | `lefthook.yml` |
| single-job CI that reads as the gate, three separate caches, `cancel-in-progress` | `.github/workflows/ci.yml` |
| dual MIT/Apache-2.0 licensing | `LICENSE-*` |

Adjusted for this project:

- **The Rust root is a virtual manifest.** geoquery's is also a package
  because `pixi publish` needs `cargo install --path .`; nothing here
  packages a Rust artifact, so the root stays `[workspace]`-only.
- **Pixi came back — as a tool belt, not a runtime** (decision-8). geoquery's
  `pixi.toml` exists for conda packaging, a Python SDK, and an offline sandbox
  transport; this repo has none of those, so decision-5 dropped pixi entirely.
  What brought a slimmer version back: the toolchain grew binaries npm cannot
  carry (convco, actionlint, cargo-deny/nextest/llvm-cov, wasm-pack + the
  wasm32 std) and per-tool install instructions are drift waiting to happen.
  Scripts stay behind `bun run`, rust stays pinned by
  `rust-toolchain.toml` (pixi mirrors the pin), and there is no pixi-run
  indirection anywhere.
- **The Tauri app crates are workspace members** living under `apps/*/src-tauri`
  (geoquery has no apps of that kind); the member glob plus two names, with
  the reason in the root `Cargo.toml`.
- **A `codegen` turbo task** and the generated-dir dependency edge
  (`@castellan/protocol` typecheck depends on `@castellan/rust#codegen`) —
  geoquery's client was scaffolding-only when copied, so this edge is new.
- **WASM**: geoquery has no wasm face; `crates/wasm` + `packages/wasm`
  (wasm-pack → bundler target, git-ignored output, lazy load) are new.
- Cargo **nextest/llvm-cov** stay out of the gates (plain `cargo test` /
  `coverage` tasks) so CI needs only bun + rust; both binaries ship in the
  pixi env for local use — swap the gates over when CI wants it.
- **convco** owns the commit-msg check when installed (grep fallback for
  contributors without it) and **actionlint** lints the workflows — neither
  ships a usable npm CLI, so both live in the pixi env (or arrive as release
  binaries like cargo-deny's). **fast-check** + `@castellan/utils` (tsconfig
  bases, test fixtures) give the TS suites the same property/fixture
  vocabulary the Rust side has in proptest/rstest.

Taken from **pixi-sandbox**: the manifest-header-comment house style (the
*why* and the tradeoff live in the file, not the wiki), `deny.toml` as the
license/bans/sources gate, the `backlog/` + `.knowledge/` systems
(backlog.md tool + Google's OKF for the knowledge bundle), the `skills`
devDependency (the agent-skills CLI), the **pixi dev-tool environment**
(adopted late, decision-8 — tools only), and the **docs site template** —
`apps/docs/` is pixi-sandbox's Astro + Starlight app adapted (its
version-substitution rig stays behind until versioned pages exist), plus
turbo only where the workspace graph earns it — this repo's graph does
(three apps + four packages + one Rust node + the docs site), so turbo
stays.

## Adding things

- **A crate**: directory under `crates/` with a `Cargo.toml` that inherits
  (`version.workspace = true`, `[lints] workspace = true`), a line in root
  `[workspace.dependencies]`, a row in `crates/README.md`. The glob makes it
  a member; no other edit.
- **A TS package**: directory under `packages/` with a `package.json` and
  `tsconfig.json` extending the base; `bun install` links it.
- **A protocol method**: a variant in `RpcMethod` (+ `RpcResult`), `bun run
  codegen`, a method on `CastellanClient`, a match arm in each app's
  dispatcher. Four files, and the compiler finds the ones you forget.

## Planning and knowledge

Two systems, inherited from the pixi-sandbox/geoquery pattern, with a hard
boundary between them:

- **[`backlog/`](backlog/)** — delivery state in the
  [backlog.md](https://backlog.md) format: **37 tasks** across six
  milestones (m-0 foundation → m-5 v1.0), **7 decision records**, and
  **11 planning/spec/research/spike docs**. The whole v0.1→v1.0 roadmap
  lives here: v0.1 daily driver (autofill, TOTP, save path, tray), v0.2
  hygiene + Android alpha, v0.3 the LAN device mesh (LocalSend interop),
  v0.4 the soft security key (passkeys on every platform), 1.0 dev tools,
  recovery, and hardening. Run `bun run backlog` for the kanban board.
- **[`.knowledge/`](.knowledge/)** — durable knowledge in Google's
  **Open Knowledge Format v0.2**: 24 concepts across six categories
  (project, faces, security, features, infrastructure, research), each
  carrying typed YAML front matter. This is the
  *why*: threat model, biometric-unlock architecture, the passkey
  enforcement rule, keepass-rs and LocalSend findings, the naming research
  behind Castellan/Fob.

The boundary: backlog holds *what ships next and whether it shipped*;
`.knowledge` holds *how it works and why it is built that way*. The five
facts an agent needs before touching the repo are in
[`.knowledge/CONTEXT.md`](.knowledge/CONTEXT.md).

## Naming

Product: **Castellan** (the keeper of the castle keys). Browser companion:
**Fob** (the thing you carry that grants access — the soft security key).
Both were checked against the password-manager landscape in October 2026:
`Fob` is clean in-category, `Castellan` has only unrelated hobby projects.
Before publishing: EUIPO/WIPO trademark search (classes 9, 42), GitHub org,
`castellan` on crates.io/npm, and `castellan.app`.
