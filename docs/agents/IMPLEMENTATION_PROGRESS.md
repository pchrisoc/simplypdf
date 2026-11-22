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

## 2026-11-20 — ENG-08.06 publication-token validation

- PR #80 merged into main as `4373533f54ab83cdc7b08236ff1e8dfa95255a27` after CI run `44817938794` and SonarCloud run `44817938203` both passed on `e0d47f8`.
- Follow-up source commit `f2ec8f4` validates that a published token has the exact current version prefix and 16 lowercase hexadecimal hash digits. Previously malformed same-version tokens were incorrectly reported as stale rather than incompatible.
- Regression commit `6166647` tests short, long, invalid-hex, uppercase and suffixed tokens, plus valid-shaped stale tokens. CI pending for this follow-up branch.
- ENG-08.06 remains unchecked: automated edit/undo/redo publication, managed workbook lifecycle, lock retries, and full acceptance are still outstanding. Continue implementation after checks pass.

## 2026-11-20 — ENG-09.05 review status history foundation

- Branch `feat/eng-0905-review-status-history-20261120` adds `crates/measure/src/review_status.rs`: configurable unique statuses, explicit authorship and timestamps, append-only transitions, duplicate suppression, and focused regression tests.
- Acceptance remains incomplete: persist histories with annotation IDs, expose desktop/MCP controls, integrate imports and undo/redo, and verify CI. ENG-09.05 stays unchecked.
- Continue ENG-08 PRs #82/#84/#85 and the rest of ENG-09 onward; do not treat this foundation as completion of the 214-item backlog.

## 2026-11-20 — ENG-09.05 chronological status history follow-up

- PR #86 commit `c2efa75` fixes the CI Clippy `double_ended_iterator_last` error in historical status queries and rejects out-of-order transitions before mutating history, with a focused regression test.
- SonarCloud previously passed on `75cddb6`; CI failed at Clippy. New head checks require verification before merge.
- ENG-09.05 is not yet complete: annotation persistence, UI/MCP exposure, stable annotation identity and undo/redo handling remain. No README checkbox changed.
- Open PRs #82, #84 and #85 need rebasing/conflict resolution and check verification; prioritize coherent milestones rather than merging unverified foundations.

## 2026-11-20 — ENG-09.05 audited history restoration

- PR #86 commit `40a1520`: repair Clippy `filter_next` failure using `rfind`; implement `ReviewStatus::from_history` to validate imported/stored event sequences and `transition_counts` for per-author audit summaries.
- Focused tests cover restoration, author counts, and rejection of nonchronological history. CI and SonarCloud must be rerun on the new head before merge.
- Remaining acceptance: bind review statuses to persistent annotation IDs, document save/load, UI and MCP controls, integration tests. ENG-09.05 remains unchecked; continue all remaining ENG milestones.

## 2026-11-20 — Review workflows and ENG-09.06 metadata foundation

- PR #86 (ENG-09.05 review-status history foundation) merged as `098872bf84a74a5757f773bee4e911e19afb10b7` after CI and SonarCloud passed on `12c8df6`. Full ENG-09.05 acceptance still requires annotation persistence and UI/MCP integration, so the checkbox remains unchecked.
- Branch `feat/eng-0906-review-metadata-20261120` implements a serializable `ReviewMetadata` type with discipline, assignee, priority, specification, package and case-insensitive tags. Validation rejects empty fields, oversized values and duplicate tags. Focused tests cover JSON round trips and invalid input without mutation.
- ENG-09.06 remains unchecked pending annotation association, register editing/filtering, UI/MCP exposure and full CI acceptance. Continue all other unchecked ENG tasks, including existing ENG-08 feature PRs.

## ENG-09.06 follow-up — 2026-11-20 PST

- Commit `5fac82b7c6c86a9f65e5393eb3ed358e71f4bd12` on PR #87 bounds metadata to 64 tags, adds case-insensitive tag removal without mutating unrelated tags, and trims discipline matching.
- Focused regression tests added: `tag_removal_is_case_insensitive_and_preserves_other_tags` and `tag_limit_rejects_overflow_without_mutation`. Tests are committed but not yet verified by CI at this head.
- PR #87 remains open pending required checks. UI/register persistence integration is not yet implemented; ENG-09.06 remains unchecked.
- PR #73 is merged; PRs #82, #84, #85 remain open and need integration/mergeability repair.

## ENG-09.06 tag rename checkpoint — 2026-11-20 PST

- PR #87 commit `2ee80f5bd68f3ce942f585155f6ee97230b76d54` implements atomic case-insensitive tag rename with duplicate validation and an associated regression test.
- Checks on the latest head remain unverified; do not merge or check the README until required checks pass and UI/register/persistence acceptance is satisfied.

## ENG-09.06 assignment and priority checkpoint — 2026-11-20 PST

- Commit `ba7fa47ecf19707fc2010a25fc7e94702e719e17` adds atomic `set_assignee` and `set_priority` operations, preserving valid prior values on rejected updates; `assignee_and_priority_updates_are_atomic` tests the behavior.
- The feature remains a metadata foundation. No annotation storage, UI/register editing or end-to-end acceptance has been verified; do not check ENG-09.06.
- PR #87 latest CI, SonarCloud and platform checks need to finish successfully before any merge.

## ENG-09.06 CI repair — 2026-11-20 PST

- At PR #87 head `894b3ded`, SonarCloud passed, while CI failed at rustfmt, and Git dependency, Windows ARM64, and FreeBSD jobs failed because `Cargo.lock` omitted the new `serde_json` dev dependency for `simplypdf-measure`.
- Commit `7ca131a` adds `serde_json` to the measure package's lockfile dependency list; `d7dbf7a` applies Rust formatting changes to review metadata. Both fixes require fresh workflow verification.
- ENG-09.06 remains unchecked until annotation persistence, UI/register integration, and full acceptance checks pass.

## ENG-09.06 CI and package filtering — 2026-11-20 PST

- At head `02a66ca`, Nix, SonarCloud, Windows ARM64 and FreeBSD passed. Ubuntu CI failed only at `cargo fmt --all -- --check`, with five exact layout diffs in `review_metadata.rs`; commit `0fd125f` applies the formatting shown in the job log.
- Commit `ca0bec3` adds case-insensitive `matches_package` and a focused test for missing, matching and nonmatching package values. These commits require new CI verification; do not merge on the older successful checks.
- PRs #82, #84 and #85 are not currently mergeable; #73 is merged. ENG-09.06 remains unchecked because persistence and UI/register integration are incomplete.

## ENG-09.06 CI and filter checkpoint — 2026-11-20 PST

- PR #87 head `7010e4c`: Nix, SonarCloud, Windows ARM64, FreeBSD passed; Ubuntu CI failed only at `Lockfile unchanged`, with Cargo reordering `serde` and `serde_json` in `simplypdf-measure`.
- Commit `8170d17` fixes canonical lockfile order. Commit `4d76f92` adds case-insensitive, whitespace-tolerant `matches_assignee` and `matches_priority` with missing/nonmatching cases in `assignee_and_priority_filters_handle_case_whitespace_and_missing_values`.
- New head requires CI verification. UI and annotation persistence integration remain unimplemented; ENG-09.06 remains unchecked. The overall README engineering backlog remains the objective.

## ENG-09.06 specification review checkpoint — 2026-11-20 PST

- At PR #87 head `678f5ea`, Nix, SonarCloud, Windows ARM64 and FreeBSD passed; Ubuntu CI failed solely on two rustfmt diffs in review assignee/priority matchers. Commit `26e8c7176eca5070c21aa9895326cc364cecc15e` applies the exact CI formatting corrections.
- Commit `9c65a369e5c578a6e11d3f9f55131ee0037d64cd` adds atomic specification updates and case/whitespace-tolerant filtering with a focused test covering missing values, invalid updates and clearing values.
- CI on these latest commits has not yet been verified. No README checkbox is complete: annotation persistence, register/UI integration and end-to-end tests remain.

## ENG-09.06 stable annotation metadata registry — 2026-11-21 PST

- PR #87 merged after CI, Nix, SonarCloud, Windows ARM64, FreeBSD passed at head `6422c52`; squash commit `5074331bc6ea54c9f15af1b3b1e055f8654ae874`.
- New branch `feat/eng-0906-annotation-metadata-registry-20261121`, commit `9de825c5d5df0fdbdff98d4297eb4256d3c04b82` adds `ReviewMetadataRegistry` keyed by stable annotation ID, validated atomic updates, removal, capacity limits, and JSON roundtrip/invalid-replacement regression test.
- Tests are committed but not yet verified by CI. The registry is a persistence data model, not yet connected to PDF annotation save/reopen or UI; ENG-09.06 remains unchecked.

## ENG-09.06 annotation metadata batch update — 2026-11-21 PST

- PR #88 initial CI run `44883525707` failed at rustfmt in `review_metadata.rs` (blank line and two compact struct literals); SonarCloud run `44883525541` passed.
- Commit `ba0e6effbe119efe027482747b61264046416e1e` applies the reported formatting changes and implements bounded atomic `ReviewMetadataRegistry::set_many` with last-write-wins duplicate ID semantics. Test `registry_batch_is_atomic_and_last_duplicate_wins` covers rollback after invalid metadata and duplicate resolution.
- Required CI checks for the updated head remain pending. Registry remains an in-memory/serializable foundation: PDF annotation persistence and UI integration are not complete. Do not mark ENG-09.06 complete.

## ENG-09.06 deterministic registry filtering — 2026-11-21 PST

- PR #88 merged as `a20320898ea89f672818a2040a0e134c467d0305` after CI and SonarCloud succeeded at head `a4dcb36`.
- New branch `feat/eng-0906-registry-query-20261121`, commit `08b21fe3dfd4398c77e2f52aa45b8a69d4f68bee`, adds composable discipline/assignee/priority/specification/package/tag filters on stable-ID registry records and deterministic ID ordering. Regression test `registry_filters_are_combined_and_results_sorted_by_stable_id` covers multi-filter matching, order, and absent matches.
- Checks on this new branch are not yet verified. Actual PDF save/reopen and desktop register integration remain incomplete, so ENG-09.06 stays unchecked. ENG-08 PRs #82/#84/#85 remain open.

## ENG-09.06 bounded register navigation — 2026-11-21 PST

- PR #89 CI `44892584787` failed at rustfmt in `review_metadata.rs`; SonarCloud `44892584754` passed. Commit `a6018f2245e22539d613a5cae6f8a91168c08b69` applies the reported formatting fixes and implements `ReviewMetadataRegistry::filter_page` with exclusive stable-ID cursor and a maximum 1,000 rows per page.
- Test `registry_filter_pages_use_exclusive_stable_id_cursors` covers zero limit, first page, missing cursor ID and end of listing. New head CI not yet verified; no README checkbox changed.
- PDF annotation save/reopen and desktop register integration remain necessary to complete ENG-09.06; broader README backlog remains open.

## ENG-09.06 filtered counts and CI repair — 2026-11-21

- PR #89 commit `c8471121dc6a229c52ab721d0299f477469e8871` adds `count_filtered`, an allocation-free matching-record count that uses the same filter predicate as paginated register results; `filtered_count_matches_pages_and_respects_empty_query` covers filtered, paginated and unfiltered counts.
- The same commit applies the two precise `cargo fmt --check` changes reported by failed Ubuntu CI run 44898022052. SonarCloud passed on the preceding head; new-head CI remains unverified.
- ENG-09.06 is not complete: persist metadata with PDF annotations, wire the desktop register, and verify save/reopen and end-to-end acceptance before checking README.

## ENG-08.09 Excel Power Query link — 2026-11-21 PST

- Branch `feat/eng-0809-excel-powerquery-20261122`, commits `1097057a03330cf07c6c0b4d88ef60e225337cc8` and `be792eee5d6e3ea694e8c3c3c0d55c3b067ce3e8`: `excel_power_query_source` generates a typed Power Query M import of the existing `TakeoffData` defined range with path validation and quote escaping. Focused unit tests cover path validation, named-range selection, and escaped strings.
- `docs/EXCEL_TAKEOFF_LINK.md` documents loading a separate Excel estimate and refreshing a named-range query, plus the crucial limitation that immutable snapshots cannot yet be overwritten at the same source path.
- ENG-08.09 remains unchecked pending user-facing desktop/CLI entry point, actual Excel smoke test, same-path managed publication (ENG-08.05/08.06), and passing CI. Do not report a fully live Excel link before these prerequisites are implemented.

## ENG-08.09 Power Query schema guard — 2026-11-22

- PR #90 previous head `fe2220a30f2c63ae8ddb0f2d1f307f23062436cf` passed CI run `45001190114` and SonarCloud run `45001189937`.
- New commits `9f4f646642517e1a04ab5d5f735da2b321542fee` and `a9ec4f6c9b1789435bc6c71251d01a7e44f976aa` implement a Power Query schema guard: exact 14-column header matching and per-row schema version validation prevent silent quantity misinterpretation after source changes. The existing query regression test now asserts both guard steps. The second commit corrects M string quoting for expected columns.
- New-head CI is not yet verified. PR remains draft; no README checkbox changed. Outstanding: desktop/CLI integration, same-path managed publication, actual Excel smoke test and final CI.
