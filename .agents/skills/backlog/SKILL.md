---
name: backlog
description: Managing project work as Markdown tasks with Backlog.md. Use when creating, breaking down, tracking, or updating tasks; when the user mentions the backlog, a kanban board, a task ID (task-N), acceptance criteria (AC), or a Definition of Done (DoD).
---

Backlog.md keeps every task as a plain Markdown file under `backlog/`, versioned with the code, so task state is a git commit. Drive it through the CLI: prefer `backlog` commands over hand-editing task files, so IDs, filenames, and metadata stay consistent.

## Running it in this repo

This project has `backlog.md` as a root workspace devDependency with a plain
`backlog` script — run it as:

```
bun run backlog <args>
```

- Bun executes the `backlog` binary from `node_modules/.bin` and stands in
  for the `#!/usr/bin/env node` shebang itself — this workspace has no node
  anywhere, and none is needed.
- The version is decided by `bun.lock` (the frozen lockfile installs it);
  never `bun x backlog` from the registry.
- The backlog is scaffolded and populated (`backlog/` is committed); no
  `init` is needed.

## Agent mode

Read commands default to an **interactive TUI** that never returns in a non-interactive shell. Always run non-interactively:

- Add `--plain` to every read (`task list`, `task <id>`, `search`) for stable text.
- Add `--json` when you will parse the output; `--json` is versioned and machine-readable.
- Never launch `backlog board` (bare), `backlog browser`, or `backlog task <id>` without `--plain` — they block on a UI.

## The loop

1. **Find work** — `bun run backlog task list -s "To Do" --plain`, or `bun run backlog search "<query>" --plain`.
2. **Read before coding** — `bun run backlog task <id> --plain`. Read the acceptance criteria and any plan first; match test and interface names to the task's vocabulary.
3. **Claim and plan** — `bun run backlog task edit <id> -s "In Progress" -a @me --plan "approach"`.
4. **Record progress in notes** (execution log), not comments — `bun run backlog task edit <id> --notes "..."` then `--append-notes "..."` for more lines.
5. **Verify against AC** — mark each criterion with `--check-ac <n>`; write a PR-ready `--final-summary "..."` when the work is done.
6. **Close** — set `-s Done`, or `bun run backlog task complete <id>` during cleanup (keeps the record and dependency links). Use `bun run backlog task archive <id>` for cancelled, duplicate, or invalid work.

## Creating tasks

```
bun run backlog task create "Title" -d "description" --ac "First,Second" -l area --priority high --dep task-1
```

Sub-task: add `-p <parent-id>`. Draft: add `--draft`, or `backlog draft create "..."` then `backlog draft promote <id>`. Full flag surface is in [`REFERENCE.md`](REFERENCE.md).

## Input gotchas

- **Multi-line** description/plan/notes: repeat the `--append-*` variant once per line (works in every shell, including agent sandboxes), or put real newlines inside double quotes. Do **not** use `$'line1\nline2'` — tree-sitter agent sandboxes reject it.
- **Literal backticks** in task text: single-quote the argument (`'Document `backlog init` setup'`), or the shell runs command substitution before Backlog.md sees the text.
- **Notes vs comments**: implementation notes and the final summary carry execution progress; comments are append-only review discussion (`--comment "..." --comment-author @you`). A standalone `---` line is reserved as a comment delimiter.

## More

For the exhaustive command and flag table (task edit AC/DoD operations, dependencies, milestones, drafts, board/export, browser, config, JSON shapes) see [`REFERENCE.md`](REFERENCE.md), and trust `bun run backlog <command> --help` as the live source of truth over any cached list here.
