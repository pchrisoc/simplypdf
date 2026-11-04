# SimplyPDF implementation plan

## Product scope

Build a local-first PDF workstation for architecture, engineering, and construction teams. Ship Windows, macOS, and Linux desktop builds from the first milestone. The first release is a takeoff workstation. An estimator can open a drawing, set and check its scale, measure distances, group quantities, and export an auditable takeoff. Full drawing review and revision comparison follow this release. Full replacement adds production PDF editing, batch document workflows, collaboration, migration, and specialized integrations.

Use [Bluebeam’s feature matrix](https://support.bluebeam.com/revu/subscription/subscription-features.html) as the baseline for a parity tracker. Record each workflow, its inputs and outputs, test fixtures, target platforms, and status. Track equivalent workflows and exact format compatibility separately. Track Bluebeam-hosted services, AI features, and vendor-specific integrations explicitly so they stay visible when assessing full replacement.

## Architecture decision

Start with a spike for **Qt 6 Widgets and C++**, using **PDFium** for rendering, text extraction, and coordinate transforms, and **qpdf** for structural editing and PDF object access. Prototype **Tesseract** for offline OCR. Confirm licensing, build reproducibility, installer size, and memory usage before committing. Keep the engine interfaces narrow enough to replace a dependency if the spike fails.

- Use PDF page space as the canonical coordinate system. Centralize transformations for zoom, rotation, crop boxes, and device pixel ratio. Render large sheets as cached tiles with cancelable background work.
- Store standard annotations and appearance streams in the PDF. Keep richer application state in documented, versioned data with stable IDs. Define save modes for ordinary edits, incremental updates, and full rewrites. Detect signed or protected files before offering a write.
- Run PDF parsing and rendering in a restricted worker process with explicit resource limits. Use an undoable command model and recoverable, atomic saves. Keep original documents available for recovery during batch operations.
- Keep collaboration optional for desktop workflows. Add a separate service with a transactional metadata store, versioned object storage, and event delivery when shared projects and review sessions enter scope.

## Delivery milestones

| Milestone | Build | Acceptance gate |
| --- | --- | --- |
| 0. Foundation | Build and package on all three OSes. Establish page geometry, the fixture corpus, and the shared measurement engine interface. | Each packaged build opens the same drawings and passes the coordinate transform and reference-length tests. |
| 1. Calibrated takeoff | Ship the guided calibration flow, per-sheet and per-region scales, snapping, straight and continuous distance tools, measurement list, save/reopen, CSV export, and read-only MCP tools. | Pass the takeoff release gates below. This milestone is the first usable release. |
| 2. Advanced takeoff and PDF review | Add area, perimeter, volume, angles, counts, cutouts, dynamic fill, Spaces, formulas, and XLSX export. Expand the viewer with layers, print, markup tools, statuses, and reusable tool sets. | An estimator completes a mixed-scale plan takeoff and exports quantities that agree with the in-app schedule. Review markups round-trip through another viewer. |
| 3. Revisions and sets | Sheet detection, page labels, Sets, synchronized view, manual and automatic alignment, color overlays, compare markups, revision histories, batch linking, and slip sheets. | A revised drawing set retains the correct markup positions, sheet links, and revision lineage. Ambiguous matches are flagged for review. A canceled batch leaves the originals intact. |
| 4. PDF production | Create and combine PDFs, reorder and replace pages, edit page content, manage forms, run OCR, redact, flatten, sign and validate signatures, and automate batch work through a scriptable CLI. | OCR output is searchable and visually aligned. Redacted material is absent from text extraction and PDF object inspection. Signature tests cover validation, permitted incremental changes, and invalidation. |
| 5. Collaboration and migration | Shared projects and live review sessions, identity and roles, guest access, version history, presence, offline queues, conflict handling, audit exports, and import tools for documents, tool sets, profiles, and reports. | Two clients can edit offline, reconnect, and converge while preserving attributable edits. Permissions and revocation pass adversarial tests. A pilot team can move a real project with an explicit record of any unsupported data. |
| 6. Specialized workflows | 3D PDF review, CAD/BIM and document-system connectors, print-to-PDF workflows, enterprise deployment, and advanced sheet review and overlay automation. Evaluate vendor-hosted integration paths through documented APIs. | Validate each workflow with representative projects and publish its supported operating systems, formats, and integration requirements. |

## Compatibility and release gates

- Build fixtures from public specifications, documented Revu workflows, and PDFs created for compatibility tests. Use the extracted Windows and legacy Mac binaries as inspection references where a specific question warrants it. Keep vendor binaries and assets outside the SimplyPDF build and distribution.
- Test annotation import and export in both directions with Revu and at least one independent PDF viewer. Preserve unknown PDF data where feasible and record any conversion loss before writing. Use representative BTX, BPX, and markup exchange samples before claiming support for those formats.
- Include large vector plans, scanned sheets, mixed page boxes and rotations, forms, layers, fonts, malformed files, encrypted documents, and signed PDFs. Automate rendering comparisons, semantic round trips, save-failure recovery, and batch cancellation.
- Set measured budgets for first-page display, pan and zoom, memory, and batch processing during the foundation spike. Run those benchmarks on representative hardware for all three platforms.
- Ship signed Windows and notarized macOS packages and an initial Linux package against a named distro baseline. Verify clean install, upgrade, document recovery, and rollback before a pilot release.
- Before calling SimplyPDF a full replacement, have reviewers, estimators, and document controllers complete representative projects and close every required gap in the parity tracker. Assess Studio migration, external connectors, and AI-assisted workflows as distinct deliverables.

## Takeoff workflow

1. **Open a sheet and choose Measure.** Display the current scale and its scope next to the cursor or toolbar. A new sheet shows **Scale required**. The user can zoom and place endpoints while setting the scale.
2. **Set the scale.** Default to **Calibrate from a known distance**. Ask for two points on a long, labeled dimension, then accept input such as `12' 6 1/2"`, `4.25 m`, or `4250 mm`. Offer standard scale presets and PDF-embedded scale as shortcuts with visible source labels. Show the reference line, entered dimension, units, and affected sheet or detail before saving.
3. **Check a second distance.** Offer a one-click check with another labeled dimension. Show the expected length, computed length, and difference. Use a reference in the other axis when available, especially on scanned drawings. Show **Calibrated** after the first reference and **Checked** after the independent check passes. Flag failed checks and require the estimator to correct or explicitly accept an unchecked scale before using it in final totals.
4. **Handle mixed-scale sheets.** Start with a sheet scale and allow a named detail region with its own calibration. Show which scale applies as the cursor moves. Ask for a selection when regions overlap. Flag a path that crosses scale boundaries and let the user split it into separate measurements. Copy a scale to other sheets only through an explicit action that previews the affected pages.
5. **Measure.** Click two points for a straight length or continue clicking for a polyline. Show a live length and an endpoint marker. Snap to supported vector endpoints and intersections with a visible indicator and a modifier to bypass snapping. Provide a magnified placement aid for scans. Let the estimator drag endpoints, undo, repeat the last tool, and keep measuring without reopening a dialog.
6. **Organize and export.** Assign measurements to categories, label them, and filter or subtotal by sheet and category. Keep display rounding separate from calculation. Export CSV rows with the quantity, unit, sheet, category, and scale status. Save and reopen the project with editable endpoints and calibration history.

## Scale and measurement implementation

1. **Define geometry first.** Store endpoints in canonical PDF page coordinates. Normalize CropBox and MediaBox offsets, page rotation, and PDF UserUnit exactly once. Test transforms from canvas to page and back at multiple zooms, pan offsets, and display densities. The same saved endpoints must produce the same distance on all three OSes.
2. **Implement the measurement engine as a UI-independent module.** For a two-point calibration, divide the known physical distance by the reference distance in canonical page units. Apply that factor to the measured segment length, or sum physical segment lengths for a polyline. Convert through a canonical physical unit and round only for display or an explicitly configured export precision. Reject zero-length references, nonpositive dimensions, and invalid units.
3. **Persist provenance.** Store the document revision, page identity, scale region, reference endpoints, entered length and units, check result, and calibration ID. Record the calibration ID and version with every measurement. On recalibration, preview affected values and let the estimator explicitly recompute them. When a drawing revision changes, require a check before reusing its calibration.
4. **Treat scans as a separate accuracy case.** Compare independent horizontal and vertical references. If they disagree beyond the configured tolerance, report possible distortion and mark the scale as failed. Defer axis-specific correction or deskew until validated by its own fixtures. Include an unchecked or failed scale status in any draft export.
5. **Verify persistence before interoperability claims.** Round-trip geometry and quantities through save, reopen, edit, and export. Verify PDF annotation appearance and measurement metadata separately in Revu and another viewer. Keep a documented project format as the source of full-fidelity calibration data until PDF interoperability has been demonstrated.

## MCP integration

Build a thin local MCP adapter around the same document and measurement services used by the desktop UI. Put tool schemas and service contract tests in the first sprint so desktop and agent behavior evolve together.

- **First release, read-only.** Support an explicitly selected project. Provide tools to list sheets, inspect calibration status, list measurements, calculate a preview from specified page coordinates, and return takeoff totals. Responses include document revision, sheet and region IDs, units, and calibration provenance. A preview with missing or ambiguous scale returns a structured error.
- **Next release, controlled writes.** Add calibration proposals and measurement creation through the shared undoable command path. Separate preview from apply. Require an expected document and calibration revision, reject stale requests, and use request IDs to make retries safe. Surface changes in the desktop app and record them in the project history.
- **Connection and tests.** Start with a versioned local stdio server and include configuration instructions for a supported MCP client. Scope it to an explicitly selected project and expose only named operations. Test connection, tool discovery, invalid input, stale revisions, and identical results from the desktop and MCP for the same endpoints. Run one end-to-end takeoff query through an actual MCP client before release.

## Takeoff release gates

| Gate | Pass condition |
| --- | --- |
| Calculation accuracy | For exact vector fixture endpoints, computed length differs from the fixture's reference by at most the greater of 0.1 mm or 0.01% of length. Cover metric, decimal imperial, and fractional feet/inches, including rotated, cropped, and non-default UserUnit PDFs. This is a software test with exact endpoints. |
| Scale verification | An independent known dimension passes a configurable tolerance. Start pilot testing at 0.5% and tune with estimators and representative drawings. Include good scales, incorrect presets, resized PDFs, and distorted scans. The result shows the discrepancy and the active scope. |
| Scope and persistence | Page scales, detail scales, missing scales, overlapping regions, and cross-region paths have explicit outcomes. Save/reopen, undo, and export preserve the expected values and their calibration provenance. |
| Ease of use | As an initial usability target, at least four of five estimators unfamiliar with SimplyPDF can set and check a scale, measure three lines, and export the takeoff within five minutes without coaching. Include a mixed-scale sheet and record where users hesitate or make errors. |
| Cross-platform and MCP | Run the same fixtures on Windows, macOS, and Linux. Compare numeric results before display rounding and verify that the desktop, exported values, and MCP use the same geometry and scale version. |

## Implementation sequence

1. Initialize source control, exclude local Bluebeam artifacts, and wire three-platform builds. Define the fixture manifest and generate reference drawings with independently specified lengths, units, and page transforms.
2. Spike the viewer on large vector and scanned sheets. Measure zoom and pan performance, and establish the page-to-canvas transform tests before building takeoff controls.
3. Build the unit parser, calibration records, distance engine, and MCP service contract. Run numerical fixtures on all three platforms before connecting the UI.
4. Implement the calibration wizard and live distance tool together. Put the workflow in front of estimators early and refine endpoint placement, shortcuts, and scale visibility.
5. Add scale checks, detail regions, continuous lengths, persistence, CSV output, and the read-only MCP adapter. Test wrong scales, revised sheets, overlapping regions, and failed saves.
6. Package the takeoff pilot, run the release gates, and fix any accuracy or workflow failures before adding advanced takeoff and broader Revu features.

## Coding-agent handoff

Treat this file as the implementation brief. Start at milestone 0, then build milestone 1 in vertical slices. The broader replacement milestones define the roadmap. Implement the takeoff workflow first and keep later interfaces small until a working slice needs them.

### Workspace and build contract

- Start in the directory containing this plan. Check for existing source, repository status, and local instructions before editing. The initial workspace contains the plan and local reference artifacts. Keep `references/bluebeam/`, the original Revu installer, generated builds, and local projects out of source control and release bundles.
- Use CMake and CTest with C++20 for the initial Qt Widgets spike. Suggested layout is `src/core/` for units, geometry, calibration, and takeoff commands, `src/pdf/` for PDF adapters and worker protocol, `src/app/` for the Qt UI, `src/mcp/` for the local MCP executable, and `tests/fixtures/` for small generated or redistributable PDFs.
- Make the core testable without Qt, a PDF engine, a window system, or network access. Add a named CMake preset for core-only builds and a desktop preset once dependency acquisition works. Pin dependencies, record their acquisition and licenses, and document exact bootstrap commands for each OS in the root README.
- Use Windows x64, macOS arm64, and Ubuntu LTS x64 as the proposed first build matrix. Record the tested OS versions and assess macOS Intel and other Linux distributions separately. Run core and integration tests on each supported target. Keep packaging checks distinct from tests that require an interactive desktop.
- For the first slice, add a deterministic PDF fixture generator and headless core tests before the viewer. For the next slice, open a fixture, display it, and place points that round-trip through the view transform. Then wire calibration, measure, persistence, CSV, and MCP in that order. Each slice should leave a runnable build with reproducible verification instructions.

### Milestone 1 data and behavior contract

| Record | Required data |
| --- | --- |
| Project | Schema version, project ID, source PDF relative path, source SHA-256, and project revision. |
| Sheet | Document revision plus zero-based page index as identity, with the PDF page label for display. |
| Region | Stable ID, sheet ID, display name, and a page-space rectangle for the first release. Define boundary inclusion consistently for both UI and MCP. |
| Calibration | Stable ID and revision, sheet and optional region, scale source, reference endpoints and known length when applicable, canonical scale factor, display units, verification status, tolerance, and independent check result when present. |
| Measurement | Stable ID, sheet, ordered page-space points, category, label, calibration ID and revision, and display unit. Derive distance from the recorded geometry and calibration rather than treating a rounded value as canonical. |

- Define canonical page coordinates as PDF user coordinates mapped into points with `UserUnit` applied, relative to the unrotated MediaBox lower-left corner, with positive Y up. Crop, rotation, zoom, pan, and display density belong to the view transform. Document any renderer API that already applies one of these conversions and test that the adapter applies it only once.
- Use millimeters for physical lengths internally and the exact conversion of 25.4 mm per inch. For a preset, convert canonical page points to paper length using 72 points per inch before applying the real-to-paper ratio. Parse fractional and mixed imperial lengths explicitly. Reject invalid fractions, conflicting signs, nonfinite coordinates, missing units in calibration inputs, and degenerate reference lines. Document the accepted syntax in the UI and tests.
- A region calibration takes precedence over the page default only for geometry contained in that single region. Check every segment against region boundaries, including segments with endpoints outside a region that cross its interior. Treat coincident boundaries and overlaps as ambiguous unless the user explicitly chooses one scale that covers the whole measurement. Use the same resolver for live preview, save, export, and MCP.
- Use scale states **missing**, **unchecked**, **checked**, **failed**, and **stale** in the domain model. “Calibrated” in the UI means unchecked. Reject missing, failed, and stale scales for final totals. Allow unchecked scales in final totals only after explicit acknowledgment, and include the status in the export. Draft outputs visibly distinguish unverified values. Compute check error against the entered reference length.
- Save a versioned JSON project alongside the source PDF for the takeoff pilot. Write through a temporary file in the destination directory and replace the project atomically where supported. Detect concurrent edits before replacing and retain the last readable project on failure. If the PDF moves, allow relinking after its hash matches. If the hash changes, flag calibrations as stale and require review. Keep prior calibration revisions so existing measurements stay reproducible after recalibration. The pilot treats the source PDF as read-only. PDF annotation interoperability is a separate spike.
- Use UTF-8 CSV with stable headers and quoting. Include sheet, measurement ID, category, label, length, unit, scale status, calibration ID, and calibration revision. Group totals only after unit normalization. Apply rounding after aggregation and escape spreadsheet formula prefixes in user-entered text fields.

### Initial MCP tool contract

Provide a separate local stdio entry point that opens an explicitly selected project in read-only mode. Resolve the project path at launch. Tools take project-scoped IDs and do not accept arbitrary file paths. Return tool errors with a machine-readable code and a brief user-facing explanation. Keep stdout reserved for the protocol.

| Tool | Inputs and result |
| --- | --- |
| `list_sheets` | Optional page cursor. Returns stable sheet IDs, labels, scale states, and the project revision. |
| `get_calibration` | Sheet ID and optional region ID. Returns calibration factor, source, units, provenance, status, and revision. |
| `preview_distance` | Sheet ID, two or more canonical page-space points, optional region ID, and requested output unit. Returns an unrounded length, unit, resolved scope, scale status, and calibration revision. |
| `list_measurements` | Optional sheet/category filters and page cursor. Returns measurement IDs, computed values, units, and calibration references. |
| `get_takeoff_totals` | Optional sheet/category filters, output unit, and draft flag. Returns subtotals and identifies any excluded or acknowledged measurements. |

Handle unknown IDs, invalid geometry, ambiguous scope, missing or stale calibration, and unsupported project schema explicitly. Include project and document revisions in results so clients can detect stale context. For the pilot, describe MCP as reading the latest saved project snapshot. Live unsaved state can be added later through an explicit desktop service connection.

### Verification and completion checklist

1. **Core fixtures.** Include 3–4–5 triangles, horizontal and vertical checks, a polyline with known segment lengths, equivalent metric and imperial values, and at least one fractional-inch reference. Include near-zero and malformed inputs. Compute expected values independently of the app.
2. **PDF fixtures.** Cover nonzero MediaBox and CropBox offsets, rotations of 0°, 90°, 180°, and 270°, non-default UserUnit, a large vector plan, and an intentionally stretched scan. Add mixed-scale, overlapping-region, and boundary-crossing cases.
3. **End-to-end pilot.** Start with a new project, open a drawing, calibrate and check a known scale, measure three distances, edit one endpoint, save, restart, reopen, and export. Query the same project through a connected MCP client and compare units, distances, scale status, and totals. Repeat on the three target OSes.
4. **Failure recovery.** Exercise an unwritable destination, interruption during save, PDF relink, revised PDF, recalibration, and malformed project data. Confirm the previous usable project remains readable and that failures cannot silently change quantities.
5. **Handoff evidence.** Record exact build and test commands, OS coverage, numerical and usability results, packaged artifact paths, and remaining gaps. Label tests that were not run. Mark a milestone complete only after its gate is evidenced.

The toolchain, OS baselines, and pilot tolerances above are starting decisions. Record changes and their rationale in the repository when the spike produces contrary evidence. Feature work should resume from the earliest unmet gate.
