# SimplyPDF

SimplyPDF is a planned local-first PDF workstation for architecture, engineering, and construction. The first usable release targets calibrated drawing takeoff on Windows, macOS, and Linux. Write all first-party application, library, and tooling code in Rust.

See the [implementation brief](SimplyPDF-implementation-plan.md) for data contracts and release criteria, and the [progress log](PROGRESS.md) for verification and handoff notes. Check off each item when its acceptance evidence is recorded. Keep each slice reviewable and record the outcome and next step in Markdown and its Git commit.

See the [architecture and backend maintenance guide](docs/architecture.md), [desktop UI direction](docs/desktop-ui.md), and [synthetic fixture manifest](tests/fixtures/README.md).

## Repository policy

Do not commit or push Bluebeam code or any proprietary source code or files to GitHub. This includes vendor installers, extracted or decompiled files, binaries, bundled fonts and assets, and proprietary sample documents. Keep local vendor material under the ignored `references/` directory or outside the checkout. Use original code and generated or explicitly redistributable fixtures. Review staged paths and content before committing, and review outgoing commits before pushing. Git ignore rules can be bypassed and do not protect files Git already tracks.

Project remote (SSH) `git@github.com:pchrisoc/simplypdf.git`

## Stage 0 · Foundation

- [x] Add the roadmap, progress log, and ignore rules for local vendor artifacts and build output.
- [x] Configure a dependency-free Cargo workspace and headless Rust core crate.
- [x] Generate the first deterministic PDF fixture with independently specified geometry and lengths, plus a manifest.
- [x] Implement canonical page geometry and headless tests for rotation, crop and media offsets, UserUnit, and canvas round trips.
- [x] Define and test the shared distance interface for straight and continuous paths.
- [x] Draft the [read-only MCP contract](docs/mcp/README.md) and test the shared distance preview service.
- [ ] Add project-backed MCP service and transport contract tests as persistence and scope resolution land.
- [x] Add the minimum Rust toolchain build and test workflow for Windows x64, macOS arm64, and Ubuntu 24.04 x64.
- [ ] Confirm the three CI jobs pass and record the tested OS coverage.
- [x] Build the [macOS viewer spike](spikes/viewer/README.md) and verify raster and pointer alignment against the four-page fixture.
- [x] Compare PDFium with the macOS reference on the four-page fixture and record its UserUnit behavior.
- [x] Read page boxes, rotation, and UserUnit from the actual PDF in the PDFium probe.
- [ ] Verify PDFium on Windows and Linux and validate parser agreement on a broader fixture corpus before wiring a production adapter.
- [x] Add single-page zoom, pan, and Fit page controls informed by the [Revu navigation study](docs/revu-navigation-study.md).
- [x] Add the macOS file picker, file drop, and keyboard navigation. Exercise both pan gestures through egui input tests.
- [ ] Confirm native chooser selection and cancellation, then verify native input on the supported desktop platforms.
- [ ] Evaluate Rust desktop UI frameworks and Rust-compatible PDF rendering and editing adapters. Pin the dependencies selected for the spike. Check licensing, reproducibility, installer size, and memory. Open and render a fixture and verify point placement.
- [ ] Add a restricted PDF worker, tiled rendering, and cancellation. Benchmark first-page display, zoom, pan, and memory on large vector and scanned sheets.
- [ ] Package each target and verify that all builds open the same fixtures and pass transform and reference-length checks.

## Stage 1 · Calibrated takeoff pilot

- [ ] Implement the unit parser, calibration records, distance engine, and numerical fixtures for metric and imperial values.
- [ ] Ship a vertical slice from opening a sheet through setting a known scale and measuring a line. Add snapping, a scan placement aid, endpoint editing, and undo. Test placement with estimators.
- [ ] Add independent scale checks, presets, detail regions, overlap and boundary handling, revision staleness, and continuous lengths.
- [ ] Add categories, filtering, subtotals, and atomic versioned project saves with relinking and calibration provenance.
- [ ] Add UTF-8 CSV with safe text escaping and explicit scale status. Verify save, reopen, edit, and export round trips and save-failure recovery.
- [ ] Implement the five read-only local MCP tools against the shared engine. Test structured errors and compare results in an actual MCP client.
- [ ] Publish a workflow parity tracker that separates equivalent workflows from exact format compatibility.
- [ ] Verify all takeoff release gates in the brief, including accuracy, scale verification, persistence, usability, and cross-platform consistency. Validate signed Windows and notarized macOS packages and the Linux package, including install, upgrade, and recovery.

## Stage 2 · Advanced takeoff and review

- [ ] Add areas, perimeters, volumes, angles, counts, cutouts, dynamic fill, Spaces, formulas, and XLSX export.
- [ ] Add layers, printing, review markups, statuses, and reusable tool sets. Spike annotation and measurement interoperability and record conversion loss.
- [ ] Add controlled MCP writes with preview, revision checks, idempotency, and undo.
- [ ] Verify mixed-scale exports against in-app quantities and round-trip review markups through independent viewers.

## Stage 3 · Revisions and sets

- [ ] Add sheet detection, labels, Sets, synchronized views, alignment, overlays, and compare markups.
- [ ] Add revision history, batch linking, and slip sheets. Verify markup positions, links, revision lineage, ambiguous matches, and safe cancellation.

## Stage 4 · PDF production

- [ ] Add PDF creation and assembly, page and content editing, forms, offline OCR, redaction, flattening, signing, and a batch CLI.
- [ ] Verify OCR alignment, redaction through text and object inspection, and signature behavior for incremental updates.

## Stage 5 · Collaboration and migration

- [ ] Add optional shared projects, live review, roles, guest access, version history, presence, offline queues, and audit exports.
- [ ] Add migration tools for documents, tool sets, profiles, and reports. Verify offline convergence, permissions and revocation, and pilot migration with a record of unsupported data.

## Stage 6 · Specialized workflows

- [ ] Validate 3D PDF review, CAD/BIM and document-system connectors, print-to-PDF, enterprise deployment, and advanced automation with representative projects.
- [ ] Assess vendor-hosted services, Studio migration, and AI-assisted workflows separately. Publish platform, format, and integration support.
- [ ] Verify representative projects with reviewers, estimators, and document controllers and close required parity gaps before claiming full replacement.

## Foundation build

Install Rust 1.85 or newer with Cargo and rustfmt. Use the native Rust toolchain and linker for your target OS.

```sh
cargo build --workspace --locked --offline
cargo test --workspace --locked --offline
cargo fmt --all -- --check
```

The headless geometry tests run without a PDF engine. The fixture manifest includes the regeneration command.
