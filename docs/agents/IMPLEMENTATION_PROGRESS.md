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

## CI repair checkpoint — 2026-11-18

- PR #51 (`fix/ci-chrono-fill-sign-20261118`) merged into `main` as `f65bac92ca8fe5f7b3880f9178b7fac96acc186e` after its head commit `9d9c5e7dbbf791cc7d861c3f8bae1926e34b05a3` passed both CI run `44469422950` and SonarCloud run `44469422931`.
- The fix adds a scoped `RUSTSEC-2026-0543` unmaintained-chrono advisory exception and changes the Fill & Sign Escape test to compare persisted annotations rather than bytes affected by `/ModDate`. This is a maintenance advisory exception, not a replacement for reviewing future advisories.
- PR #52 merged the agent handoff document as `3a85124770e0d39a22a6e2797866e01bec9e90f0`.
- Prior sandbox `simplypdf-measure` job `job-50ec9695c666` did **not** pass: it timed out while compiling dependencies during Clippy. Do not treat this as a code test failure or a test pass; retry with a warm Cargo cache and confirm `done` plus `ok:true`.
- Post-merge `main` push CI status is **not verified** by the PR-only workflow-run lookup. Confirm the `main` run and its conclusion before declaring the repository fully green.
- Remaining feature work includes ENG-08.01 and live takeoff workbook publishing; no additional feature checkbox was completed by these CI/doc fixes.

## Active agent iteration — measurement identity (2026-11-18 PST)

- The sandbox gateway is healthy (`doctor`: ready, 2 CPUs) and new isolated session `task-a58a1d6354d0` was prepared from `main` at `2b89ee9`.
- PR #53 (`fix/measurement-id-normalization-20261118`) adds `simplypdf_measure::normalize_ids` for missing or repeated imported `/NM` names while preserving the first valid existing name, plus save/reopen regression coverage. **Not merged or verified yet.**
- The first sandbox `cargo fmt --check` found two formatting differences; corrected in PR head `8dc3c769b5c7c2a6f35252132440f25275c41be8`.
- Sandbox focused test job `job-990afd705df7` launched and was compiling dependencies at last check. Poll via the gateway `job` action until `done` and `ok:true`; GitHub CI run `44474007793` and SonarCloud `44474007757` are also being checked.
- The new normalization API alone is not the entire ENG-08.01 feature: UI/automation exposure, schema migration, malformed input coverage, and permission checks remain. Do not tick its README checkbox yet.
- Dependency update PR #47 remains open; its eight grouped dependency updates require full CI validation and should not be merged blindly.

## Dependency modernization checkpoint — 2026-11-18 PST

- Dependabot PR #47 (eight Rust dependency upgrades) passed CI (`44469619980`), SonarCloud (`44469619892`), Nix (`44469619807`), Windows ARM64 (`44469619988`) and FreeBSD (`44469619819`) at head `12de5379dc21134d14703e0714d113ef8e1ec45d`.
- Merged PR #47 into `pchrisoc/simplypdf` main using squash commit `2703c0ac5c38df9101bc81ef9344ae0eab5a1b9b`. Main post-merge verification remains separate; do not assert that push CI passed without checking it.
- PR #53 (measurement IDs) remained open, SonarCloud green and CI in progress at its formatted head commit `8dc3c769b5c7c2a6f35252132440f25275c41be8` when this log was updated.

## Measurement test evidence — 2026-11-18 PST

- Sandbox session `task-a58a1d6354d0` checked out PR #53's first test commit (`2ee5867`) and ran focused test job `job-990afd705df7`: `cargo test --locked -p simplypdf-measure imported_measurement_ids_are_unique_and_persist -- --nocapture` **passed (1/1)** after a cold 5m11s dependency build.
- Full package test job `job-5a9bf9561c77`: `cargo test --locked -p simplypdf-measure` **passed: 40 unit tests plus 3 property tests, 0 failures**. The tested implementation is unchanged by PR #53's later formatting-only commit `8dc3c769b5c7c2a6f35252132440f25275c41be8`.
- PR #53 SonarCloud `44474007757` succeeded. GitHub CI `44474007793` still in progress at last query. Do not merge or check README feature until green, and do not claim the full measurement identity/workbook goal complete from this scoped test.

## Measurement lint/format evidence — 2026-11-18 PST

- `cargo clippy --locked -p simplypdf-measure -j 1 --all-targets -- -D warnings` sandbox job `job-5566c64f857a` **passed**, `ok:true`, after 46.83s.
- Fetched formatted PR #53 head and reran `cargo fmt --check` sandbox job `job-9f79188ccefb` **passed**, `ok:true`.
- Together with 43 passing package tests and passing SonarCloud, the scoped local gates have succeeded. GitHub CI `44474007793` must still conclude success and mergeability must be rechecked against current `main` (which also received PR #47) before PR #53 is merged.

## Measurement identity merge checkpoint — 2026-11-18 PST

- GitHub CI run `44474007793` and SonarCloud run `44474007757` for PR #53 head `8dc3c769b5c7c2a6f35252132440f25275c41be8` both concluded **success**.
- PR #53 merged into `main` as squash commit `d5f698ea358e0e4b49c5b73f16c89f98fca66a60` using GitHub's merge API with an expected head SHA guard.
- The prior measurement test evidence is scoped to 43 tests; it does not establish full cross-platform or post-merge `main` CI status. ENG-08.01 remains unchecked pending full identity acceptance, and managed workbook publishing remains unimplemented.
- Sandbox session `task-7b28aba97166` started verification job `job-306fd2e91a0c` for `simplypdf-measure`; formatting and strict Clippy finished, while scoped test compilation was still running at last check. Poll for `status: done` and `ok: true` before recording a pass.

## Checkpoint 1 schema integration — 2026-11-19 PST

- PR #55 (`agent/test-versioned-takeoff-schema-20261118`) merged into `main` as squash commit `02d99ae86fab44c2fdc49e741d81bea6fc599865`, expected head `c9c1a05e0e865a9800f112b91fd6144619ad9933`.
- Adds a PDF save/reopen and CSV schema regression test, `docs/TAKEOFF_ROW_SCHEMA.md`, and checks the Checkpoint 1 persistent-ID/versioned-row-schema entry. PR #55 GitHub CI run `44480529741` and SonarCloud run `44480529737` both concluded `success` at the expected head.
- Independent sandbox `simplypdf-measure` verification job `job-ba414fec31e4` finished `status: done`, `ok: true`; 41 unit tests and 3 property tests passed, with formatting and strict Clippy gates. Sandbox verification was scoped to the measure crate, not full UI/automation/platform acceptance.
- Codex automated review did not complete because of account review usage limits; this is not a successful review. The PR diff was inspected manually before integration.
- Post-merge `main` push CI is **not confirmed** by the PR-only workflow-run lookup. ENG-08.01 broader automation/UI migration behavior, malformed imported IDs in downstream consumers, and managed workbook sync remain unverified/incomplete; do not mark ENG-08.01 checked on this evidence alone.
- PR #54 remains open and is not mergeable against current main; reconcile or close it rather than overwrite newer progress.

## Vendor annotation preservation regression — 2026-11-19 PST

- Resumed Sandbox Coding Gateway session `task-54f4203a14e9`; verification job `job-351b216373a9` completed with `status: done`, `ok: true`. The job passed formatting, strict Clippy, layering checks, 42 measurement unit tests and 3 property tests (45 total), including `normalize_ids_preserves_unrelated_annotation_fields`.
- Published the exact verified 18-line regression test to branch `agent/measurement-vendor-fields-20261119`, source commit `a9b07ea9a70888c6441bd66351ed856b51647552`, changing only `crates/measure/src/tests.rs` before this log update. The test proves duplicate ID repair preserves distinct unknown `/VendorKey` values in both annotations after save/reopen.
- Sandbox verification is scoped to `simplypdf-measure` and was performed on an isolated checkout; GitHub PR-head CI and post-merge `main` CI are separate gates. Do not claim either green without completed runs. ENG-08.01 remains unchecked; automation/UI integration and workbook publishing remain outstanding.

## Vendor annotation regression integration — 2026-11-19 PST

- PR #58 (`agent/measurement-vendor-fields-20261119`) merged by squash into `main` at `dc34118539761a89ce4379706485f00a91f97d9e`, with expected PR head `510fee910156e7dad1261aa0f21475cd20841205`. The source test commit was `a9b07ea9a70888c6441bd66351ed856b51647552`.
- PR-head GitHub Actions CI run `44520499695` and SonarCloud run `44520499617` both completed successfully before merge. Prior sandbox job `job-351b216373a9` completed `done`, `ok:true`, with 45 measurement tests and formatting, Clippy, and layering checks.
- The merged test verifies preservation of unknown vendor annotation fields when duplicate measurement IDs are normalized and the PDF is saved/reopened. No ENG-08.01 checkbox was changed: automation/UI migration and managed workbook publishing remain incomplete.
- Post-merge `main` push CI has not been independently verified. A fresh isolated sandbox checkout `task-31a711e3136e` was prepared from merge SHA `dc341185`; no subsequent feature edits or verification results are claimed from that checkout.

## GitHub-only automation trial — 2026-11-19 PST

- PR #59 (https://github.com/pchrisoc/simplypdf/pull/59), branch `fix/measure-derived-overflow-20261119`, based on `main` `c6c323e740bae93ca38ee9aafd3b5814e6c3ed25`.
- Source commit `bcc3676cf55cdb6bf1f37262140fc9cd2a0d37b9` adds a guard for all derived finite reading fields. Test commit `8e05345620ac6e87fef58d988c7efe028f4ac0f5` exercises extreme valid coordinate/scale boundaries.
- This is a defensive hardening and boundary test, not a proven pre-fix failing regression or full ENG-06.08 acceptance. Do not check the README feature checkbox.
- GitHub-only execution cannot run local Cargo commands. Poll PR-head CI and SonarCloud to completed success before any merge; verify final PR head after this documentation commit. If checks do not execute, leave PR open and report the precise blocker.

## GitHub-only measurement overflow integration — 2026-11-19 PST

- PR #59 (`fix/measure-derived-overflow-20261119`) merged into `main` with squash commit `8646d84d3b3845758491f2f6b196e65c26f995b5` using an expected-head guard for `19ed96764673d987570dac47862d04edf24a51f2`.
- Source commit `bcc3676cf55cdb6bf1f37262140fc9cd2a0d37b9` rejects non-finite derived readings; test commit `8e05345620ac6e87fef58d988c7efe028f4ac0f5` covers extreme valid scale/coordinate boundaries. Documentation commit `19ed96764673d987570dac47862d04edf24a51f2` records scope and limitations.
- PR-head GitHub Actions CI run `44562185365` and SonarCloud run `44562185356` both completed successfully at the expected head before merge. The GitHub connector cannot independently run local Cargo tests; do not claim additional local tests or post-merge `main` CI green.
- This is scoped hardening only, not complete ENG-06.08 acceptance. ENG-08.01 and opt-in managed workbook publishing remain unfinished. No README feature checkbox changed.

## GitHub-only view coordinate bounds integration — 2026-11-19 PST

- PR #61 (https://github.com/pchrisoc/simplypdf/pull/61), branch `fix/measure-view-coordinate-bounds-20261119`, merged into `main` via squash commit `abe3c0a5a2cc9320755dcad6a42a97730b89309c` after expected head `d6a03b4b7b9f6e08d5f0f9b2f24787dde2bbc78f` was confirmed.
- Source commit `40387d1869e9fe59cf9489c48845a7a3222a0db9` validates finite, bounded transformed output from `user_to_view`; test commit `d6a03b4b7b9f6e08d5f0f9b2f24787dde2bbc78f` adds a `/UserUnit 2` boundary regression and ordinary-coordinate check in `crates/measure/src/tests.rs`.
- PR-head GitHub Actions CI run `44595279636` and SonarCloud run `44595279709` both completed with `success` before the guarded merge. No local Cargo test or post-merge `main` CI result is claimed.
- ENG-06.08 remains unchecked because this is only a coordinate-validation hardening change; ENG-08.01 downstream integration and managed workbook publishing remain incomplete. PR #62 has failing CI and is not mergeable at the time of this entry; investigate independently before merging.

## GitHub-only CSV BOM fix rebased on current main — 2026-11-19 PST

- PR #62 was based on an older main and its latest formatting commit `03ac7d46db69f30240c809123355dc79f0570350` had no GitHub Actions runs returned; earlier head `ffd5de15ec1093ef099fda60ee60fe27069b2dac` failed CI. Do not treat either as verified.
- A new branch `fix/measure-csv-bom-formula-rebased-20261119` starts from current main after PR #63. Source commit `7c319abee8462953f348f917323ae0807b1ee9fe` updates `crates/measure/src/lib.rs` CSV formula-prefix detection to skip Unicode BOM. Test commit `c3b794f1a1765f5b9acea2b15ae7f2fe3e12b64e` adds `csv_escapes_formula_prefixes_after_unicode_bom` in `crates/measure/src/tests.rs`.
- PR-head GitHub Actions and SonarCloud verification remain pending at documentation time. No README checkbox changed. ENG-08.01 and managed workbook publishing remain unfinished.

## CSV BOM protection merged — 2026-11-19 PST

- PR #64: https://github.com/pchrisoc/simplypdf/pull/64. Source `7c319abee8462953f348f917323ae0807b1ee9fe` ignores Unicode BOM while checking CSV formula prefixes; regression `c3b794f1a1765f5b9acea2b15ae7f2fe3e12b64e` covers BOM-prefixed `=HYPERLINK`.
- CI run `44616248077` failed 1 of 6,591 tests because PDF text normalization removed the BOM. Test correction `232d87cabf3455af8245657f370227fc827edbfe` verifies the exported value is apostrophe-prefixed after normalization.
- At exact PR head `232d87cabf3455af8245657f370227fc827edbfe`, GitHub Actions CI `44625500175` and SonarCloud `44625500111` both completed successfully. Squash merge to main: `e9a9b2283be5f3320d7be98dde9bda4c28e0014e`.
- No README engineering acceptance checkbox changed. ENG-08.01 persistent row identity and ENG-08.05/08.06 managed workbook publishing/live updates remain unfinished. Post-merge main CI is not claimed.

## CSV invisible-prefix formula protection integrated — 2026-11-19 PST

- PR #66 (https://github.com/pchrisoc/simplypdf/pull/66) added CSV formula-prefix handling for zero-width Unicode characters and a focused regression covering five invisible prefixes. Verified PR head `74239fd760f8fb99ac3846717b8b6a0a6cab07c9`.
- GitHub Actions CI run `44643437234` and SonarCloud run `44643437193` both completed successfully at that head. Guarded squash merge SHA: `6e8cf5c803194bb150e253cc9b4e7bf88d9b2c1c`.
- No README engineering checkbox changed. ENG-08.01 persistent row identity and ENG-08.05/08.06 managed workbook publishing/live updates remain unfinished; no post-merge main CI result is claimed.

## CSV invisible mathematical operators integrated — 2026-11-19 PST

- PR #68: https://github.com/pchrisoc/simplypdf/pull/68. Source commit `59c8e6174b759f14d0b4431e08756ba5cc1cd94e` adds U+2061–U+2064 to the ignored leading Unicode prefix set when checking CSV cells for formula injection. Test commit `76eb438a1fc8ca139006bdfe2bcbfe10c1b5c444` covers four invisible mathematical operators preceding `=HYPERLINK`.
- Initial GitHub Actions CI `44658364038` failed `cargo fmt --all -- --check` (not a functional test failure); formatting correction commit `35f1b9892cc44e7a60cd60c8974e1f1ea43532fd` applied the exact rustfmt diff.
- At exact verified head `35f1b9892cc44e7a60cd60c8974e1f1ea43532fd`, CI `44663712509` and SonarCloud `44663712447` both completed successfully. Guarded squash merge SHA: `f196418a1b9baeb439796b25fd0ce209be3ea1be`.
- No README acceptance checkbox changed: ENG-08.01 persistent row identity, ENG-08.05 managed workbook publishing, and ENG-08.06 live refresh still require complete acceptance evidence. No post-merge main CI result is claimed.

## CSV bidirectional prefix regression integrated — 2026-11-19 PST

- PR #70: https://github.com/pchrisoc/simplypdf/pull/70. The measurement CSV exporter now treats eleven Unicode bidirectional formatting prefixes as ignorable when identifying spreadsheet formulas; focused tests cover the prefixes. Source/test commits `790ae3e`, `49cdb4f`, formatting commits `2f224e0` and `533a40a0155e6514df38d8b75e2020727f59792f`.
- GitHub Actions CI `44679069754` and SonarCloud `44679069733` passed on exact PR head `533a40a0155e6514df38d8b75e2020727f59792f`; guarded squash merge SHA `b1f84bdac010f663c0b7b3fe991aa389130fe6e1`. Post-merge main CI not yet verified.
- ENG-08.01 remains unchecked: persistent measurement identity requires continued integration and acceptance verification; managed workbook publishing and live refresh remain outstanding.

## ENG-08.01 — Cross-annotation ID collision regression (in progress)

- PR #71: https://github.com/pchrisoc/simplypdf/pull/71. Source commit `472d49989e3c457b953a78e77abb8b8aa34c4da4` reserves existing `/NM` IDs from all annotations during measurement ID normalization, not just measurement annotations. Test commit `baa5874a506caada21e7d336e5007dfd3c030647` adds collision, idempotence and save/reopen coverage.
- Initial CI `44680244866` failed at `cargo fmt --all -- --check`; source formatting commit `2c1b88c06f2197091be43d3429683545f5f51fa2` applies the reported rustfmt diff. SonarCloud `44680244746` passed on the prior head, but neither CI nor SonarCloud is yet claimed green on the new head. Do not merge until both pass and branch currency/review/security checks are satisfied.
- README ENG-08.01 checkbox stays unchecked pending full acceptance, including verification of the new regression and other row-identity export pathways.

## 2026-11-20 — ENG-08.01 PR #73 CI repair

- Inspected `main` at `d1a62ea2298d443546bd18ed50c66dcf5d9a357c`, open PRs #73/#57/#56, README (214 unchecked entries), AGENTS.md, and CLAUDE.md.
- CI run [44708866613](https://github.com/pchrisoc/simplypdf/actions/runs/44708866613) failed in Clippy: `error[E0425]: cannot find value fingerprint in this scope` in the measurement listing response. SonarCloud run [44708866594](https://github.com/pchrisoc/simplypdf/actions/runs/44708866594) succeeded on the preceding head `65c08ae8`.
- Source commit `11eed51b` removes the accidentally added export-only fingerprint field from `measurement_list`, preserving read-only listings for legacy documents while retaining change fingerprints in checked CSV/XLSX export responses.
- Regression commit `44563e20` verifies that listing a synthetic PDF returns its count/truncation without requiring an export fingerprint.
- Verification of the new head is pending GitHub CI; do not merge or check ENG-08.01 until required checks pass and full milestone acceptance is established.
- Remaining ENG-08.01 work: verify persistent row identities across all mutation paths and save/reopen, confirm versioned export schema and migration behavior, resolve any further CI failures, then merge and update the README only if all acceptance requirements are met.

## ENG-08.01 follow-up — 2026-11-20

- PR #73 head before this run: `ab443efa`; CI run 44717226109 failed at Clippy, SonarCloud run 44717226170 succeeded.
- Failure identified from CI job 136223603344: `clippy::field_reassign_with_default` in `crates/measure/src/xlsx.rs` test; fixed by `eb5cf9a` using a struct initializer.
- Added a validated takeoff refresh-token comparison API (`snapshot_needs_refresh`) in `3aac82b`, with first-publish, unchanged, edited, and truncated-listing tests in `2e69784`.
- New CI and SonarCloud results are **pending** for the latest head; no merge or README checkbox change until checks and full acceptance are verified.
- Remaining: integrate persistent-ID normalization into durable PDF save and export entry points; verify imported/duplicate IDs across desktop, CLI and XLSX; implement managed workbook lifecycle and live publishing in later ENG milestones.

## ENG-08.01 — Export refresh status and CI regression repair (2026-11-20)

- PR #73 source commit `1dd6710` adds optional `published_fingerprint` comparison to automation CSV and XLSX export results; `needs_refresh` is true when the published snapshot is missing or stale. The export still validates and writes output, so this is a status indicator, not managed workbook live refresh.
- Test commit `4584eca` covers missing/stale publication tokens and ensures invalid IDs cannot bypass validation. CI regression repair `853cdc9` corrects the CSV quoted-ID assertion after run 44724472473 failed (SonarCloud passed on previous head).
- New CI and SonarCloud on current head remain to be verified. No README checkbox may be checked until durable PDF ID persistence and all acceptance paths are verified. ENG-08.05 managed publishing and ENG-08.06 live refresh are not complete.

### 2026-11-20 — ENG-08.01 / snapshot consistency

- PR #73: source commit `1ab7a65` includes skipped imported annotations in deterministic snapshot fingerprints, so changes to unsupported annotations trigger refresh even when supported takeoff rows are unchanged. Uses length-prefixed inputs to avoid ambiguous concatenation.
- Regression commit `d5833b0` verifies changes to skipped annotations and reasons invalidate the published token.
- CI for this head must be observed before merge; prior green checks on older heads are not sufficient. ENG-08.01 remains unchecked pending durable PDF identity normalization and cross-interface verification. No claim of completed backlog.

## ENG-08.01 checkpoint — 2026-11-20 06:05 PST

- PR #73 merged to main as squash commit `94764cf09c69d985c2c9cb1f7a96d63631befeef` after CI run [44741831961](https://github.com/pchrisoc/simplypdf/actions/runs/44741831961) and SonarCloud run [44741832512](https://github.com/pchrisoc/simplypdf/actions/runs/44741832512) both passed on head `cd01e520b00a7148640f9215bdc0298e1a7799b5`.
- Follow-up PR #75 on branch `feat/eng-0801-persistent-id-validation-20261120`: source `f43e27e` repairs measurement /NM collisions with unrelated annotation types; test `eccaf97` covers preserving unrelated annotations, idempotent migration, PDF save/reopen, and checked CSV.
- PR #75 CI is pending; do not merge without passing checks. ENG-08.01 remains unchecked because durable export/save integration and cross-interface acceptance are still incomplete.
- All 214 numbered README engineering checkboxes remain the overall goal; do not tick partial work.

## ENG-08.01 follow-up — 2026-11-20 (CI formatting and identity preview)

- PR #75 at prior head `d024885`: SonarCloud run 44750304594 passed; CI run 44750304588 failed at `cargo fmt --all -- --check`, before tests. The log identified one line-wrap difference in `crates/measure/src/lib.rs`.
- Commit `6dc56f0` applied the exact rustfmt line-wrap fix.
- Commit `3674478` introduced `pending_id_repairs(&Document)` for a non-mutating preview of imported ID repairs, including collisions with unrelated annotations.
- Commit `4d46fa3` added regression coverage confirming the preview leaves the PDF untouched and becomes zero after normalization.
- **Not yet verified:** CI/SonarCloud on the latest head, the new API's compilation/test pass, and remaining durable-save and cross-interface acceptance. Do not merge or check ENG-08.01 until verified.

## ENG-08.01 — Atomic export preparation (2026-11-20)

- Branch `fix/eng-0801-atomic-export-preparation-20261120`. Source commit `030a9f99` prepares imported-ID repairs on a detached PDF document and commits those changes only after the complete export listing passes CSV validation. This avoids mutating a live document when export preparation fails.
- Test commit `61c562b2` verifies duplicate/blank IDs are repaired, the prepared export has unique IDs, the PDF save/reopen preserves those IDs, and a second preparation performs no further repair.
- GitHub Actions CI and SonarCloud must be checked on the PR head; the GitHub connector cannot run local Cargo tests. ENG-08.01 remains unchecked pending integration and acceptance verification. No claim of complete managed workbook publishing or live refresh.

## ENG-08.01 — XLSX publication identity checkpoint (2026-11-20)

- Branch `feat/eng-0801-xlsx-snapshot-fingerprint-20261120`: checked XLSX snapshots now embed the validated `snapshot_fingerprint` as a workbook-level `TakeoffFingerprint` defined name. This enables external consumers to compare a saved workbook's publication token with a newly scanned PDF snapshot without relying on file timestamps. Legacy `write_new` remains available without checked metadata.
- Regression test `checked_workbook_carries_publication_metadata_distinct_from_legacy_export` covers checked/legacy workbook generation and distinguishes their bytes. CI and SonarCloud verification are pending; no task checkbox is marked complete.
- Remaining ENG-08.01 acceptance: desktop save integration, CLI/automation parity, end-to-end workbook metadata inspection, and cross-interface tests. ENG-08.05 managed publishing and ENG-08.06 live refresh are separate future milestones.

## 2026-11-20 — ENG-08 snapshot publication integration

- PR #77 merged as `54739dc` after CI and SonarCloud passed; checked XLSX workbooks carry a TakeoffFingerprint defined name.
- PR #78 merged as `8054a19` after CI and SonarCloud passed; typed SnapshotStatus and tests cover publication fingerprints and incomplete scans.
- Follow-up branch `feat/eng-0804-automation-status-20261120`, source commit `9a7e0d5`: CSV and XLSX automation exports use the same validated snapshot_status result, avoiding divergent fingerprint/refresh computations. Existing measurement snapshot tests cover this shared API; CI for this branch is pending.
- Do not check ENG-08.01 or ENG-08.04 yet: cross-interface save/refresh acceptance and a complete snapshot-export acceptance demonstration remain to be verified. Continue with managed workbook lifecycle and tests.

## 2026-11-20 — Publication-state diagnostics

- PR #79 merged to main as `f14261230f7ca629882c340422491ccebd2572cd` after CI and SonarCloud success. CSV and XLSX automation now share validated snapshot status.
- New feature branch `feat/eng-0806-publication-state-20261120`: `7a90d23` adds typed `PublicationState` (unpublished/current/stale/incompatible_schema); `4a1aeef` adds transition/version-mismatch regression tests; `3abfbdb` exposes state in both automation export responses; `4b1d7f8` formats tests.
- This is a diagnostic component of ENG-08.06, not full live-refresh acceptance. Required next: managed workbook lifecycle, edit/undo/redo event wiring, transient-lock retry, and CI verification. README engineering checkboxes remain unchecked.

### 2026-11-20 — parallel ENG-08 implementation and CI verification

- PR #81 merged after CI and SonarCloud passed; publication fingerprint format validation is integrated.
- PR #83 merged after CI and SonarCloud passed; versioned editable classification-list validation and focused tests are integrated. ENG-08.03 remains unchecked pending UI and workflow integration.
- PR #82 adds unit-aware material/labor rate calculation and focused tests; formatting repaired, CI rerun pending. ENG-08.02 remains unchecked pending UI, persisted fields and export integration.
- PR #84 adds unit-separated filtered quantity aggregation; CI formatting failure still needs repair and integration with source PDFs and additional filter fields.
- PR #85 adds serializable snapshot manifests and tests; CI formatting failures still need repair.
- The full engineering backlog remains in scope; no task was marked complete on partial acceptance evidence.

### 2026-11-21 — ENG-08.02 rate aggregation continuation

- PR #82: `601d0ad` adds unit-checked aggregate rate totals, finite-sum overflow handling, and focused same-unit/mixed-unit/overflow tests; `7b29f15` formats the overflow test.
- Focused test commands for CI: `cargo test -p simplypdf-measure quantity_rates` and `cargo fmt --all -- --check`. These were not executed locally; GitHub checks on the new head are pending.
- ENG-08.02 remains unchecked: persisted rate fields, desktop editing, estimate export, full acceptance verification, and green required CI are outstanding. PR #82 is not currently mergeable.


### 2026-11-21 — ENG-08.02 mixed-unit estimate totals

- PR #82 commit `f2d39d040117bd7c75ce122840862ee4218a83c8` adds `aggregate_estimate_cost` and `EstimateCost`: material, labor, and total monetary costs can be summed across physical units without falsely summing quantities. Callers must ensure the rate currency matches.
- Focused regression tests `mixed_units_can_sum_costs_but_not_quantities` and `estimate_cost_rejects_invalid_lines_and_overflow` cover cross-unit monetary aggregation, zero-line estimates, invalid input and overflow. These are committed but not yet executed/verified; check `cargo test -p simplypdf-measure quantity_rates` and `cargo fmt --all -- --check` in CI.
- GitHub returned no workflow runs or commit statuses for this new head at inspection. PR #82 remains unmergeable; do not merge or mark ENG-08.02 complete. Outstanding: persist/edit rate fields in desktop, integrate estimate exports and headless automation, resolve branch conflicts, and verify required CI.

## ENG-08.02 mixed-unit rate grouping — 2026-11-21 PST

- Commit `b7dc4f05540079d7f09b04dc337400b40eb6c397` adds `aggregate_rates_by_unit`, grouping physical quantities and costs deterministically by unit without combining incompatible quantities. Focused test `grouped_estimates_keep_units_separate_and_sorted` checks ordering, grouped totals, empty input and invalid input.
- Verification: GitHub CI on the new head remains required; test has been committed but not locally executed. PR #82 is not yet mergeable. Do not check ENG-08.02 until persisted rate fields, desktop editing, estimate export, and end-to-end acceptance are implemented and verified.

### 2026-11-21 — ENG-08.02 streaming unit aggregation

- PR #82 commit `f2dcec03f8984ee8838fc5d73c1a792cb2d1f81b` replaces cloned per-unit `Vec<RateLine>` buckets with streaming `BTreeMap<String, RateTotal>` accumulation, retaining deterministic unit ordering while reducing memory growth for large takeoffs. It also fixes accidentally escaped literal newline sequences in the overflow regression test.
- New regression `grouped_estimates_trim_units_and_reject_group_overflow` checks normalized unit grouping and finite-sum overflow rejection. Tests are committed, not yet independently verified; run `cargo fmt --all -- --check` and `cargo test -p simplypdf-measure quantity_rates`.
- ENG-08.02 remains unchecked: rate persistence, desktop editor, estimate exports, headless automation, branch conflicts and required CI remain unresolved.

## ENG-08.02 rate schedule resolution — 2026-11-21

- Feature commit `89e9d1f403082fae3973f63cdbf624ff90eda4e9` on PR #82 adds serializable named `RateScheduleEntry` and `rate_line_from_schedule` with code lookup, duplicate-code rejection, physical-unit matching, and reuse of checked cost arithmetic.
- Two focused Rust regression tests cover successful schedule-based material/labor/waste estimates and rejection of missing/duplicate codes, incompatible units, invalid quantities, and negative rates.
- Tests were committed but not executed in this GitHub-only run; no GitHub Actions runs or combined statuses were returned for previous PR head `444c45d`. Do not claim a green build or merge.
- Remaining: persisted rate schedules and measurement rate associations, desktop editing, export/automation integration, PR branch conflict resolution, and required passing CI. ENG-08.02 remains unchecked. Continue all other ENG milestones afterward.
