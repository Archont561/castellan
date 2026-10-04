# Castellan

<p align="center">
  <a href="https://github.com/Archont561/castellan/actions/workflows/ci.yml"><img src="https://github.com/Archont561/castellan/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="./LICENSE-MIT"><img src="https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg" alt="License: MIT OR Apache-2.0"></a>
  <a href="https://www.conventionalcommits.org"><img src="https://img.shields.io/badge/Commits-Conventional%201.0.0-yellow.svg?logo=conventionalcommits" alt="Conventional Commits"></a>
  <img src="https://img.shields.io/badge/Platforms-linux%20%7C%20macos%20%7C%20windows-brightgreen.svg?logo=linux" alt="Platforms">
  <img src="https://img.shields.io/badge/Faces-desktop%20%7C%20mobile%20%7C%20extension-8ddc44.svg" alt="Faces">
  <a href="https://github.com/Archont561/castellan/pulls"><img src="https://img.shields.io/badge/PRs-welcome-ff69b4.svg" alt="PRs Welcome"></a>
</p>

<p align="center">
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/Rust-1.85%2B-dea584.svg?logo=rust" alt="Rust"></a>
  <a href="https://bun.sh"><img src="https://img.shields.io/badge/Bun-1.3%2B-f9f1e5.svg?logo=bun" alt="Bun"></a>
  <a href="https://tauri.app"><img src="https://img.shields.io/badge/Tauri-2-24c8d8.svg?logo=tauri" alt="Tauri 2"></a>
  <a href="https://svelte.dev"><img src="https://img.shields.io/badge/Svelte-5-ff3e00.svg?logo=svelte" alt="Svelte 5"></a>
  <a href="https://wxt.dev"><img src="https://img.shields.io/badge/WXT-0.20-34d399.svg" alt="WXT"></a>
  <a href="https://pixi.sh"><img src="https://img.shields.io/badge/Pixi-0.81%2B-yellow.svg?logo=condaforge" alt="Pixi"></a>
  <img src="https://img.shields.io/badge/Protocol-v3-9b59b6.svg" alt="Protocol v3">
</p>

<p align="center">
  <strong>Local-first secrets manager — a KeePass-compatible vault, an authenticator, and a soft security key.</strong><br/>
  One Rust core, three faces (desktop, mobile, browser extension), one protocol: they cannot disagree.
</p>

---

> [!NOTE]
> Every face speaks **one RPC contract**, defined once in Rust: the TypeScript that describes it is
> derived (`ts-rs` → committed generated code), and behavior that must not drift — otpauth parsing,
> origin matching — is written once in Rust and linked natively or compiled to WASM. Adding an
> operation is one entry in `rpc_contract!`, one arm in the dispatcher, then `pixi run codegen`.

## 🖥️ Face & Platform Support

| Face | Stack | Status | Notes |
|------|-------|--------|-------|
| 🖥️ Desktop | Tauri 2 + SvelteKit (static, `ssr=false`) | 🟡 m-1 in development | Linux/macOS/Windows; vault session, unlock/lock and copy-aside saves landed; Linux builds need the [webkit system libraries](apps/desktop/README.md) |
| 📱 Mobile | Tauri 2 (Android/iOS) + SvelteKit | 🟡 scaffolded | Same dispatcher, same protocol; runs on hardware cadence from m-2 (task-23) |
| 🧩 Extension — “Fob” | WXT, one codebase for chromium + gecko | 🟡 core landed | Protocol-v3 transport with the association handshake (enroll → challenge → HMAC proof); fill UX is m-1 (tasks 11–12) |
| ⌨️ CLI | Rust, over the same IPC socket | ⏳ planned (task-34) | `get`, `totp`, env injection, git credential helper |

Milestones: **m-0 foundation (done)** → **m-1 v0.1 daily driver** → m-2 hygiene + Android alpha →
m-3 LAN device mesh → m-4 soft security key → m-5 v1.0 hardening. The whole roadmap lives in
[`backlog/`](backlog/); m-1's exit test is the dogfood gate — the author uses Castellan as their
only password manager for two weeks.

## 📦 Architecture

```text
  ┌────────────┐   ┌────────────┐   ┌──────────────────────┐
  │  desktop   │   │   mobile   │   │  extension ("Fob")   │
  │ Tauri+Kit  │   │ Tauri+Kit  │   │  WXT, all browsers   │
  └─────┬──────┘   └─────┬──────┘   └──────────┬───────────┘
        │ invoke("rpc") │  native messaging    │ (byte-identical framing:
        └───────┬───────┴───── host = the app ─┘  4-byte LE length + JSON)
                ▼                 binary, --native-host
   ┌──────────────────────────────────────────────────────┐
   │ protocol · dispatch · vault · otp · ipc · ipc-server │
   │ manifests · native-host · wasm · xtask               │
   └──────┬───────────────────────┬──────────────────────┘
          │ ts-rs (xtask codegen) │ wasm-pack
          ▼                       ▼
  packages/protocol         packages/wasm
  (generated wire types)    (shared logic in the browser)
          │                       │
          └────► packages/core ◄──┘  (one client base + generated face clients)
```

**Same scoped logic in every face.** A message is defined once (Rust), the TypeScript that
describes it is derived and committed, and shared behavior is linked natively (apps) or compiled
to WASM (extension). The KeePass ecosystem's worst failure modes are two halves disagreeing about
a shape; this structure makes that disagreement a compile error instead of a support ticket.

**No localhost TCP, ever** (decision-2): browsers reach the app through a Unix domain socket
(`$XDG_RUNTIME_DIR/castellan/castellan.sock`) or a Windows named pipe, with same-user enforcement
via kernel peer credentials (`SO_PEERCRED` / `getpeereid`) — no firewall prompt, no port conflict,
nothing for other local processes to query. In front of it sits the association handshake: a
browser enrolls its key once (the user confirms in the panel), and every later connection proves
possession with an HMAC over a fresh nonce. Refusal is *silence* — zero bytes — so an
unassociated host learns nothing, not even why.

---

## 🚀 Key Features

| Icon | Feature | Description |
|------|---------|-------------|
| 🔐 | **KeePass-compatible vault** | KDBX 4 open/save through keepass-rs; the unlock corpus includes a KeePassXC-2.7.12-authored anchor fixture |
| 🛟 | **Copy-aside saves** | Every save copies the old file aside (timestamped) before writing a temp file and renaming; a failed save leaves the original byte-identical (decision-3) |
| ⏱️ | **Lock tiers** | Soft lock on blur/wake/idle, hard lock after the horizon; derived keys zeroize on drop; events push so faces reflect lock state live |
| 🧪 | **Round-trip harness** | Open → save → reopen → compare every parsed field, over a generated case table + anchor fixtures and a 50-case property — “drops a field” is a red build, not a lost database |
| 🧬 | **One protocol, derived** | `rpc_contract!` emits the Rust types and face membership; xtask emits committed TS wire types and scoped face clients |
| 🚫 | **No TCP, same user only** | UDS/named-pipe native channel with peer-credential checks; unassociated extensions get silence until the user confirms them |
| 🤝 | **Association handshake** | Enroll once over the credential-checked local socket; every later connection claims its key id and answers a nonce challenge with HMAC-SHA256 — the material never crosses the wire twice (protocol v3) |
| 🧭 | **Connected-browsers panel** | Live connections with face, version and last request; a per-connection kill switch; remembered keys; enrollment prompts — visible instead of silent |
| 🔧 | **Manifest installer** | One pass writes Chrome/Edge/Brave/Vivaldi/Firefox native-messaging manifests from `host.json` (Linux/macOS; the Windows layout table is complete, its registry write lands with a Windows CI lane); the audit flags stale manifests and repair rewrites them idempotently, reporting what it fixed |
| 🕵️ | **Secrets stay app-side** | `EntrySummary` carries no secret material; the extension never holds the database; WASM never links the vault crate |
| 📦 | **Offline-capable toolchain** | Every dev tool pinned through `pixi.lock`; cargo runs offline against vendored crates on an airlocked machine |

---

## ⚡ Quick Start

### 1. Bootstrap the toolchain

Prerequisite: [pixi](https://pixi.sh). It provisions bun, Rust with clippy
and rustfmt, the wasm32 std, wasm-pack, convco, actionlint, and cargo-deny from
the committed `pixi.lock`. Linux desktop builds additionally need the
[Tauri system libraries](apps/desktop/README.md). Pixi manages tools rather
than application dependencies, and it is the sole command entry point so no
hook, CI step, or contributor silently selects a global binary.

```bash
pixi install                  # materialize the locked dev-tool environment
pixi run install              # link the workspace (+ git hooks)
pixi run codegen               # derive TS types from the Rust protocol
pixi run wasm                  # build the shared-logic WASM package
```

### 2. Run the gates

```bash
pixi run gates                 # turbo: lint + typecheck + test, every language
pixi run codegen-check         # fail on stale or untracked generated output
pixi run cargo test --workspace        # or drive Rust directly
```

### 3. Run a face

```bash
pixi run dev-desktop           # tauri dev (apps/desktop)
pixi run dev-mobile            # tauri dev (apps/mobile; see its README for android/ios init)
pixi run dev-extension               # wxt dev (chromium; :firefox for gecko)
pixi run dev-docs              # the docs site (apps/docs, Astro + Starlight)
```

> [!TIP]
> On a machine without the webkit stack (or offline), scope cargo to the library crates — the
> session skill's standing commands: `pixi run cargo test --workspace --exclude castellan-desktop
> --exclude castellan-mobile`. The Tauri shells compile in CI, which installs the GTK stack.

---

## 📖 Planning & Knowledge

Two systems with a hard boundary between them:

- **[`backlog/`](backlog/)** — delivery state in the [backlog.md](https://backlog.md) format:
  50 tasks across six milestones, 11 decision records, 20 planning/spec/research/spike docs.
  Run `pixi run backlog` for the board.
- **[`.knowledge/`](.knowledge/)** — durable knowledge in Google's Open Knowledge Format:
  35 documents across six categories. This is the *why*: threat model, biometric-unlock
  architecture, the passkey enforcement rule, keepass-rs and LocalSend findings, the naming
  research behind Castellan/Fob. The five facts an agent needs before touching the repo are in
  [`.knowledge/CONTEXT.md`](.knowledge/CONTEXT.md).

Naming: product **Castellan** (the keeper of the castle keys), browser companion **Fob** (the
thing you carry that grants access). Both were checked against the password-manager landscape in
October 2026.

## 📂 Repository Architecture

| Path | Description |
|------|-------------|
| `apps/desktop` | Tauri 2 + SvelteKit — the desktop face; one `rpc` command, unlock/lock/status commands, event forwarding |
| `apps/mobile` | Tauri 2 (iOS/Android) + SvelteKit — the mobile face; same dispatcher, same shell shape |
| `apps/extension` | WXT extension (“Fob”), one codebase for chromium + gecko; protocol-v3 native-messaging transport with the association handshake; passkey interception skeleton |
| `apps/docs` | The documentation site (Astro + Starlight, a bun workspace; deploy lands with task-44) |
| `crates/protocol` | The one RPC contract: `rpc_contract!` emits paired request/result types, each operation's client faces, and the association/panel wire types |
| `crates/dispatch` | The one transport-independent dispatcher every native face calls |
| `crates/otp` | otpauth parsing + RFC 6238 TOTP (with the RFC's own test vectors) |
| `crates/vault` | KDBX core: open, entry projection, passphrase generation, the in-memory session with lock tiers, copy-aside save, the round-trip harness |
| `crates/ipc` | The native channel's framing + the well-known socket path (std-only, no async runtime) |
| `crates/ipc-server` | The app-side socket server: connection multiplexing, the association handshake (enroll/prompt/proof), silence on refusal, the connected-browsers panel data |
| `crates/manifests` | The native-messaging manifest installer: browser detection, the one-pass write, the staleness audit, idempotent repair |
| `crates/native-host` | The byte pump each browser spawns (the app binary, `--native-host`) |
| `crates/wasm` | The WASM face of shared logic; `packages/wasm` wraps it |
| `crates/xtask` | Codegen: Rust contract → generated TS wire types, native-host manifests + the installer's identity file, scoped face clients |
| `packages/protocol` | Generated TS wire types (committed; `pixi run codegen` regenerates) |
| `packages/core` | Transport-independent client mechanics + the `Transport` seam; generated app clients extend it |
| `packages/tauri` | The tested `invoke("rpc")` transport shared by desktop and mobile, incl. event subscription |
| `packages/ui` | Shared Svelte components with Storybook stories and Playwright component tests beside them |
| `packages/wasm` | npm wrapper around the wasm-pack output |
| `packages/utils` | Shared tsconfig bases, the bun test fixtures (`createFixture`), build/test presets (bunup `libPreset`, playwright `e2ePreset`) |

---

## 🛠️ Development & Quality Gates

```bash
pixi run gates                 # turbo: lint + typecheck + test, every language
pixi run codegen && pixi run codegen-check   # regenerate, then fail on uncommitted drift
pixi run wasm                  # rebuild the WASM package
pixi run e2e               # browser e2e: desktop, mobile, extension — serial
pixi run storybook-build         # build the ui package's Storybook (static)
pixi run lint-workflows        # actionlint over .github/workflows
pixi run lint-commits          # convco: history is conventional
pixi run backlog               # the kanban board (backlog.md)
pixi run cargo test --workspace        # rust suites directly
pixi run xtask codegen
./scripts/restore.sh          # restore the offline environment (see the airlock note below)
```

`turbo` runs the monorepo and `bun run <verb>` fans out to every package. The Rust crates are
**turbo packages of their own** (`@castellan/rust-*` in `crates/*/package.json`, pathway's
shape): per-crate `nextest`/`clippy` tasks, selected by `turbo run test/lint --affected` and
keyed on precise per-crate inputs. Workspace-wide Cargo invocations (rustfmt, cargo-deny,
coverage, the Tauri app crates' clippy, codegen) stay on the façade in `crates/package.json`
(`@castellan/rust`) — so `bun run test` runs the TS suites *and* every Rust crate's suite,
in the order the graph says. Generated consumers depend
on `@castellan/rust#codegen`, and CI runs `pixi run codegen-check` to reject stale or untracked
generated output before review.

**Testing vocabulary (both languages, one shape):** `#[fixture]` ↔ `createFixture`, `#[case]`
matrices ↔ example tests over explicit lists, proptest ↔ fast-check. Case files are generated
from reviewed case tables, never committed — except the two anchors no generator can author (the
KeePassXC-written file and the KDBX 3.1 fixture). Rust lints run `clippy -D warnings` +
`missing_docs` + `unsafe_code = "deny"` — with exactly one sanctioned exception, the
peer-credential `getsockopt` in the IPC server, each function carrying a SAFETY case (see the
`[workspace.lints]` comment in the root manifest). TS runs biome + tsc/svelte-check on strict
shared bases. Browser tests ride one pinned Playwright (1.58.2) and one prebundled chromium that
`pixi run install` downloads.

> [!IMPORTANT]
> **The offline airlock.** `./scripts/restore.sh` (plus the documented two-command workaround in
> [AGENTS.md](AGENTS.md)) provisions the vendored-cargo, offline environment. Never commit the
> `[source]` vendored-sources block into `.cargo/config.toml`; a new Rust dependency needs the
> airlock — it only works if the crate is already vendored.

**Adding things:** a crate is a directory under `crates/` with an inheriting `Cargo.toml`, a line
in root `[workspace.dependencies]`, and a row in `crates/README.md`. A TS package is a directory
under `packages/` — `pixi run install` links it. A protocol operation is one entry in `rpc_contract!`
plus one arm in `crates/dispatch`; app shells never match on `RpcMethod`.

## 🧬 What came from where

The monorepo setup is copied and adjusted from
[Archont561/geoquery](https://github.com/Archont561/geoquery) and
[Archont561/pixi-sandbox](https://github.com/Archont561/pixi-sandbox).

| Pattern | Source |
|---------|--------|
| bun as the only JS runtime; turbo + biome as workspace devDeps; no node | geoquery |
| the `crates/package.json` **turbo façade** — Rust workspace as one task-graph node | geoquery |
| ts-rs codegen through an `xtask` crate into committed `packages/*/src/generated` | geoquery |
| root `[workspace.dependencies]` with documented ranges; `[workspace.lints]` (`unsafe_code = "deny"`, one documented exception) | geoquery |
| strict shared TS config bases (`noUncheckedIndexedAccess`, `exactOptionalPropertyTypes`, …) | geoquery |
| lefthook hooks whose bodies are the commands CI runs; conventional-commit `commit-msg` | geoquery |
| single-job CI that reads as the gate; three separate caches | geoquery |
| manifest-header-comment house style (the *why* lives in the file, not the wiki) | pixi-sandbox |
| `deny.toml` as the license/bans/sources gate; `backlog/` + `.knowledge/` systems | pixi-sandbox |
| the pixi dev-tool environment, tools only (decision-8) | pixi-sandbox |
| the Astro + Starlight docs site template | pixi-sandbox |

Adjusted for this project: the Rust root is a virtual manifest (nothing packages a root artifact);
pixi came back as a tool belt, not a runtime; the Tauri app crates are workspace members under
`apps/*/src-tauri`; a `codegen` turbo task and the generated-dir dependency edge; the WASM face
(`crates/wasm` + `packages/wasm`), the IPC server and the manifest installer are new.

## 🔖 Changelog & Release

Conventional commits only, enforced by the `commit-msg` hook; the changelog will be generated
from them via convco. The release flow (versioning, tags, store submissions) is task-43 — no
releases until m-1's dogfood gate passes.

## 📜 License

Dual-licensed under MIT OR Apache-2.0 — see [LICENSE-MIT](LICENSE-MIT) and
[LICENSE-APACHE](LICENSE-APACHE). Third-party packages and vendored crates retain their
upstream licenses (the vault test corpus carries its own attribution in
`crates/vault/tests/fixtures/README.md`).

## 🌟 Docs

The documentation site lives in `apps/docs/` (Astro + Starlight):

```bash
pixi run dev-docs               # live-reload dev server
pixi run docs-build             # the static build CI verifies
```

Publishing to GitHub Pages is task-44; the config is already shaped for it. Until then, the
closest things to a handbook are [AGENTS.md](AGENTS.md) (the invariants), this README, and
[`backlog/`](backlog/) + [`.knowledge/`](.knowledge/).
