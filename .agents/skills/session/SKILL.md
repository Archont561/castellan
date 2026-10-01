---
name: session
description: Start a Castellan work session on an ephemeral agent sandbox — bring the toolchains up (they do not survive the session), check the git state (the workspace may ship without a repository, and .git/config never persists), survey the backlog, and propose what to do this session. Use at the start of any session on this repository, when the user says to initialize/bootstrap the environment, pick up tasks from the backlog, or asks what to work on.
---

# Session startup for Castellan

Two jobs, in this order: **(1)** bring the environment up, **(2)** survey the backlog and
propose the session. Never propose work you cannot execute — an environment that is down
changes what is possible.

## 1. Bring the environment up (airlocked machine)

The workspace at `~/castellan` persists between sessions; everything under `/var/tmp`
(including both toolchains) does not — expect it wiped, check before re-provisioning.
Each step below is idempotent and says how to check.

### Bun (JS toolchain)

```bash
export PATH=/var/tmp/bun113:$PATH
bun --version || {                       # not provisioned this machine:
  curl -fsSL https://github.com/oven-sh/bun/releases/latest/download/bun-linux-x64.zip \
    -o /tmp/bun.zip && unzip -q -o /tmp/bun.zip -d /var/tmp && \
    mv /var/tmp/bun-linux-x64 /var/tmp/bun113
}
bun install --frozen-lockfile            # ~750 packages, ~15s
```

- The postinstall re-downloads the pinned Playwright chromium into `~/.cache/ms-playwright`
  (not persisted — this is by design; check it exists before any e2e run).
- Use `bun x <tool>` — the `bunx` shim does not work from this location.
- Browser launch needs the system libraries (CI runs `bunx playwright install-deps
  chromium`; locally apt-get the usual gtk/nss/at-spi set if e2e fails to launch).

### Rust (cargo workspace)

```bash
export RUSTUP_HOME=/var/tmp/cargo/rustup CARGO_HOME=/var/tmp/cargo PATH=/var/tmp/cargo/bin:$PATH
cargo --version || {                     # not provisioned this machine:
  curl -fsSL https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
}
```

- **Never** run cargo with only `PATH` set: without the `RUSTUP_HOME`/`CARGO_HOME` exports
  it builds into `~/.cargo` and corrupts it. If a stray `~/.cargo`/`~/.rustup` exists,
  delete it before it bloats the snapshot.
- The repo pins the toolchain via `rust-toolchain.toml` (rustfmt + clippy included);
  the first cargo run inside the repo fetches it automatically.
- The Tauri app crates need webkit2gtk, which this sandbox does not have — every
  workspace-wide cargo command carries the excludes:

```bash
cargo test  --workspace --exclude castellan-desktop --exclude castellan-mobile
cargo clippy --workspace --all-targets --exclude castellan-desktop --exclude castellan-mobile
cargo fmt --all
```

### Git (conditional — this workspace ships without a repository)

Workspace files persist, but `.git` itself does not survive an export of
this tree, and `.git/config` (identity, hooks path) is excluded from
workspace snapshots even when the repository exists. Check before relying
on git; when a repository is present, restore the two config lines and the
hooks before the first commit:

```bash
test -d .git || echo "no repository: git init -b main first"
git config user.name  arena-agent
git config user.email 297053741+arena-agent@users.noreply.github.com
bun run hooks:install                  # lefthook install (prepare guards on .git)
```

`bun run lint:commits` (convco, `.versionrc`) and lefthook's commit-msg
hook need a repository; `bun run lint:workflows` does not (explicit paths).

### Baseline before anything else

```bash
bun run test                            # JS packages
cargo test --workspace --exclude castellan-desktop --exclude castellan-mobile   # crates
```

Write the numbers down; they must rise with new work, never fall.

## 2. Survey the backlog

1. **Read the standing context** (skim, do not quote back): `AGENTS.md` (repo map,
   conventions, task commands), `.knowledge/` (OKF notes — `log.md` for what happened
   recently), and `backlog/decisions/` — the decisions are load-bearing; if you think
   one is wrong, bring a measurement, not an opinion.
2. **List the open work**: `bun run backlog task list -s "To Do" --plain`, or read
   `backlog/tasks/*.md` directly; tasks are markdown files whose frontmatter carries
   `dependencies`, `priority`, `ordinal`, `type`, and `documentation` links.
3. **Filter honestly.** A task is a candidate only when every dependency has status
   `Done`. Order candidates by priority (high → low), then `ordinal`. A spike that
   unblocks several tasks may jump the queue — say so explicitly when you propose it.
4. **Do not re-propose finished work.** Check `git log --oneline -15` and the most
   recent `.knowledge/log.md` entries. Done tasks and recorded decisions stay done.

## 3. Propose the session, then stop

Fill in the **Session proposal** template in [`standup-template.md`](standup-template.md)
and **wait for the user to pick** — do not start implementing. That file also carries the
task hand-off and session-close templates; use them at those points rather than inventing
a shape.

## 4. While you work — house rules (AGENTS.md is the full list)

- **One focused conventional commit per task** (`feat:`, `fix:`, `chore:`, `docs:`, …);
  the commit-msg hook enforces it, and changelogs are generated from the history.
- Run the gates yourself before committing: `bun run fmt && bun run lint`, the relevant
  typechecks/tests, and cargo fmt/clippy when Rust moved.
- **Complete the task file in its own format**: every AC `[x]`, status `Done`, bump
  `updated_date`, append the implementation notes and final summary sections — following
  the task's exact existing section markers.
- Stamp new `.knowledge/` entries with the current UTC time; leave historical stamps
  alone.
- Skills changes (`.agents/skills/`) are pinned in `skills-lock.json` — commit both
  together, the same way `bun.lock` is committed.
