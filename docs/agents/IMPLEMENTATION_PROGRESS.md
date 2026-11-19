# SimplyPDF — implementation progress and agent handoff

Updated: 2026-11-18 (America/Los_Angeles)
Repository: `pchrisoc/simplypdf`; integration branch: `main`.

## Instructions for the next LLM

1. Read `AGENTS.md`, `CLAUDE.md`, `docs/development-workflow.md`, this file and `README.md` before editing.
2. Check `main`, recent CI and existing PRs; use a fresh isolated task branch or sandbox gateway checkout. Never overwrite concurrent work.
3. Select one unverified checkbox or a failing regression. Scope work tightly, reproduce it, implement, add tests, run required gates, and only then check off the README entry.
4. Record each task ID, changed files, commands, test counts/results, commit SHA, and CI URLs below. Never mark work shipped on the basis of a job merely starting.
5. After passing checks, integrate safely into `pchrisoc/simplypdf` `main` and confirm the remote tip and CI. Continue with the next task only within an active authorized agent session; the sandbox coding gateway is manual-only.

## Baseline audit (2026-11-18)

- Source inspected: `main` at `224959d22850eb4307d8ce8d6539e294e42305aa` via Sandbox Coding Gateway session `task-aa7b12c8482b`, branch `codex/task-aa7b12c8482b`.
- The README checklist already marks the native CSV/XLSX snapshot writer and several desktop takeoff improvements complete.
- `crates/measure/src/lib.rs` defines `TAKEOFF_SCHEMA_VERSION = 1`, 14 shared columns, and a `Measurement.id` read from PDF `/NM`.
- `crates/measure/src/xlsx.rs` writes the `TakeoffData` named range and atomically refuses to overwrite an existing workbook.
- `crates/automation/README.md` warns imported `/NM` values can be absent or duplicated, so ENG-08.01 and Checkpoint 1 persistent identity remain open.
- Managed live workbook publishing after edit/undo/redo remains open. No continuous scheduled coding process was started.
- These findings are inspection, not full verification. Do not tick unchecked items from this audit alone.

## Priority next steps

1. ENG-08.01: define durable identity for imported measurements and versioned row migration. Preserve PDF interoperability and avoid silently changing unrelated annotations; test duplicates, missing names, save/reopen, CSV/XLSX, and automation results.
2. ENG-08.05 / Checkpoint 1: opt-in managed workbook start/status/refresh/stop with atomic safe publication; support desktop and headless automation.
3. ENG-08.06: publish after add/edit/delete/undo/redo, with reporting of incomplete listings or locked outputs.
4. Verify Calc `TakeoffData` link against a separate estimate. Keep platform-specific verification honest.

## Task log

| Date | Task | Status | Evidence / next action |
| --- | --- | --- | --- |
| 2026-11-18 | Baseline and agent handoff | Documented; no feature checked | Audit facts above; run scoped tests before any implementation claim |

## Verification log

| Date | Commit | Command or CI | Result | Notes |
| --- | --- | --- | --- | --- |
| 2026-11-18 | `224959d` | Repository inspection via n8n sandbox | Completed | No CI/test pass claimed from inspection |
| 2026-11-18 | `224959d` | `verify` `simplypdf-measure` | Submitted; status not yet confirmed | Job `job-50ec9695c666`; poll to `done` and `ok: true` |
