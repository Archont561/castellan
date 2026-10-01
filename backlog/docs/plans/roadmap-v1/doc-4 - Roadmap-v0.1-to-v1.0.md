---
id: doc-4
title: "Roadmap v0.1 to v1.0"
type: plan
created_date: '2026-10-01 21:25'
updated_date: '2026-10-01 21:25'
tags:
  - roadmap
  - planning
---
# Plan — the road from here to 1.0

**Scope.** How the 37 tasks map to milestones, what each milestone's exit
test is, and the do-not-cut list. Delivery state (status, AC) lives in the
tasks; this document is sequencing and cut policy.

## §1 Milestone → tasks → exit test

| Milestone | Tasks | Exit test |
| --- | --- | --- |
| m-0 foundation (done) | 1–6 | the gates: green CI, codegen verified |
| m-1 v0.1 daily driver | 7–17 | **the dogfood gate**: the author uses Castellan as their only password manager for two weeks |
| m-2 v0.2 hygiene + mobile alpha | 18–24 | a stranger's KeePass vault imports, audits clean, restores from backup, on an Android phone with biometrics |
| m-3 v0.3 device mesh | 25–28 | two devices sync a day of edits over LAN with no cloud, against a pcap |
| m-4 v0.4 soft security key | 29–32 | passkeys on GitHub/npm/Facebook from desktop and one mobile platform |
| m-5 v1.0 | 33–37 | reproducible builds + published threat model + external review triaged |

## §2 Sequencing rules

- m-1's save path (task-8) blocks everything that mutates the vault
  (importers, sync, extension saves) — it is the critical path.
- The IPC server (task-9) blocks every extension feature after autofill;
  association comes before passkeys, always, because enforcement without
  association is theater.
- Mobile milestones (23, 30, 31) run on hardware cadence, not CI cadence —
  start the device-checklist work early, expect the wall-clock drag.
- Sync (m-3) before passkeys (m-4): the soft-key recovery story depends on
  the mesh existing.

## §3 Cut lines (what v0.1 is allowed to ship without)

Search can be slow-but-correct; tray and hotkey can slip a point release;
autofill multi-step logins can be v0.2. **Not cuttable from v0.1**: lock
tiers + clipboard hygiene (15), copy-aside saves (8), and the connection
panel (9) — a password manager without those is a liability.

## §4 Do-not-cut list (any version)

1. No secrets in list answers (protocol invariant).
2. Origin/RP-ID enforcement in the app, never only in the extension.
3. Copy-aside save rule.
4. No localhost TCP for browser integration.
5. The sync mesh never gains a cloud relay — if this line is ever revisited,
   it is a new product decision with a new decision record, not a task.
