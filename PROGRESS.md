# SimplyPDF progress

## 2026-11-04 · Stage 0, repository and build baseline

- Added the staged README checklist and repository policy. Do not commit or push Bluebeam code or any proprietary source code or files to GitHub. The local installer and references directory are ignored.
- Adopted Rust for all first-party code and updated the implementation brief. Added a dependency-free Cargo workspace with a headless core library crate, Rust 2024 edition, and a declared minimum of Rust 1.85. Desktop UI and PDF adapter choices remain open for the foundation spike.
- Ran `cargo build --workspace --locked --offline` and `cargo test --workspace --locked --offline` on macOS 27.0.1 arm64 with Rust 1.98.1. Both commands succeeded. The scaffold has zero tests. Windows, Linux, and Rust 1.85 verification are pending.
- Attempted `cargo fmt --all -- --check`. The installed toolchain lacks rustfmt.
- Verified ignore rules for the local Revu installer, vendor reference directory, and build output with `git check-ignore`.

Next slice is a deterministic fixture generator in Rust, a fixture manifest with independently specified lengths and transforms, and the first headless geometry tests.

## 2026-11-04 · Stage 0, geometry and synthetic fixture

- Added a renderer-independent Rust transform from raw PDF user space through canonical page coordinates to a top-left canvas. The integration tests check independent expected positions, inverse transforms, crop clipping, rotation, UserUnit, and invalid input.
- Added a deterministic four-page PDF generator in Rust and a manifest. Regenerated the PDF and compared bytes with `cmp`. Rendered all four pages with Ghostscript 10.06.0 using `gs -q -dBATCH -dNOPAUSE -dPDFSTOPONERROR -dUseCropBox -sDEVICE=pnggray -r72 -sOutputFile=/tmp/simplypdf-page-%d.png tests/fixtures/page-geometry.pdf`. Visually checked triangle placement and confirmed raster dimensions match the manifest.
- Documented the engine replacement boundary and the desktop UI layout and review criteria. The viewer and engine selection are pending.
- Verified with `cargo test --workspace --locked --offline --all-targets` on macOS 27.0.1 arm64 with Rust 1.98.1. Four geometry tests passed. Windows, Linux, the minimum Rust toolchain, and rustfmt verification remain pending.

Next slice is the shared measurement contract and cross-platform core CI, followed by the desktop viewer spike using the committed fixture. The renderer adapter must open the actual PDF and match the manifest before the viewer coordinate gate can close.

## 2026-11-04 · Stage 0, shared distance and CI

- Added the Rust `DistanceScale` API for two-point calibration and unrounded straight or continuous lengths in millimeters. Tests cover the fixture triangle, malformed input, and numeric range errors.
- Added a GitHub Actions matrix for Windows Server 2025 x64, macOS 15 arm64, and Ubuntu 24.04 x64. Each job builds and tests with Rust 1.85.0 and records its compiler and target. Hosted results are tracked separately from workflow setup.
- Ran `cargo test --workspace --all-targets --locked --offline` and `cargo test --workspace --doc --locked --offline` on macOS arm64 with Rust 1.98.1. All seven integration tests passed. There are no doc tests. Local rustfmt verification remains pending.
- Reviewed outgoing paths before publication. The synthetic PDF is generated from first-party Rust source. The local Revu installer and vendor reference tree remain ignored.

Next slice is the MCP schema and service contract draft, then a Rust desktop viewer spike that opens the synthetic PDF and demonstrates pointer alignment.

## 2026-11-04 · Stage 0, read-only MCP contract and preview

- Drafted machine-readable input schemas for all five tools and documented the saved snapshot boundary, result fields, unit conventions, and error mapping.
- Added a headless Rust preview service with unrounded output conversion, explicit scale errors, and calibration provenance. Scope resolution remains a caller responsibility until the shared project service lands.
- Verified all eleven integration tests with `cargo test --workspace --all-targets --locked --offline` on macOS arm64. Parsed the draft tool definitions and checked tool names, required fields, and absence of file path inputs. Live MCP protocol, project-backed service tests, and cross-platform results remain unverified in this slice.

Next slice is the Rust desktop viewer spike. Open the synthetic drawing, render each page rotation, and verify pointer placement through the same core transform. Keep the renderer adapter separate from the viewer and record the dependency decision.

## 2026-11-04 · Stage 0, macOS viewer spike

- Added a separate Rust viewer prototype using eframe and macOS Core Graphics. The shell has sheet navigation, fit-to-window rendering, a theme switch, and a click coordinate readout.
- Verified all four fixture rotations with a test that checks real raster ink and the inverse pointer transform at the expected edge midpoint. On the desktop, clicked the visible edge and observed approximately (149.05, 120.23) canonical points, near the expected midpoint (150, 120).
- The adapter constructs the raster transform from shared page geometry after Quartz's convenience transform produced a different scale on this fixture. The Rust UI stays separate from Quartz handles. The headless build and test workflow remains independent of viewer dependencies.
- Tested on macOS 27.0.1 arm64 with Rust 1.98.1. The viewer test passed using `cargo test --locked --offline --manifest-path spikes/viewer/Cargo.toml` after fetching its dependencies. Windows, Linux, the viewer's minimum toolchain, rustfmt, and packaging remain unverified.

Next slice is a cross-platform renderer comparison against the same fixture, then file-open UI, zoom and pan, and background rendering. Full viewer acceptance remains open.

## 2026-11-04 · Stage 0, PDFium renderer comparison

- Added a standalone Rust probe against PDFium chromium/8138. Checked the downloaded macOS arm64 archive against its published SHA-256. The native library was kept outside the checkout.
- On macOS arm64, checked actual ink, PDFium's coordinate mapping, and the core inverse transform at the same edge midpoint on all four rotations. PDFium reported crop dimensions without UserUnit on this fixture. Supplying canonical raster dimensions kept the rendered page aligned with pointer placement.
- Recorded the command and metadata requirement in the viewer guide. The probe supplies fixture metadata directly. Production metadata extraction, Windows and Linux runs, and integration into the desktop UI remain open.

Next slice is to select and test a raw metadata reader for PDFium or identify an engine that supplies the required metadata. File picking, zoom, and pan are separate viewer changes that can use the existing Quartz adapter.

## 2026-11-04 · Stage 0, PDFium metadata reader

- Pinned lopdf 0.42.0 as a dev dependency and isolated it behind a function that returns core page geometry in document order. The probe reads the same bytes with lopdf and PDFium and checks both against the fixture manifest.
- Added three tests for inherited boxes and rotation, indirect values, local overrides, noninherited UserUnit, defaults, malformed boxes, and cyclic parent links. All four viewer and probe tests and all eleven core tests passed on macOS arm64. The native PDFium probe passed on all four fixture rotations using the extracted metadata.
- The viewer still uses Quartz. PDFium on Windows and Linux, broader parser agreement, and minimum toolchain verification are pending.

Next viewer slice is file opening, zoom, and pan. Keep the click marker anchored in canonical page coordinates. The separate PDFium track still needs cross-platform and broader fixture verification before it can replace Quartz.

## 2026-11-04 · Stage 0, Revu navigation and Rust viewer controls

- Used REA for static inspection of the local Revu 21.11 Windows assembly and recorded evidence and limitations in `docs/revu-navigation-study.md`. The binary contains separate wheel preferences. Bluebeam's public navigation documentation corroborates the default single-page wheel behavior and pan gesture. Exact handler flow and live behavior remain unverified.
- Added wheel and toolbar zoom, fit reset, and pan controls using the existing Quartz raster and core transform. The selection stays in canonical page coordinates. High magnification scales the current raster.
- All five viewer and metadata tests and all eleven core tests passed on macOS arm64. The desktop smoke test confirmed toolbar zoom, pointer-centered wheel zoom, marker alignment, and fit reset. The automated drag attempt did not establish panning behavior; gesture verification is still open.

Next slice is the file picker and input verification, then sharper zoom via background rendering. Vendor artifacts and analysis dumps remain excluded from Git.
