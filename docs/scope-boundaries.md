# SimplyPDF capability and scope boundaries

This document qualifies product claims for **ENG-30.04**. It separates work that SimplyPDF actually performs from workflows that require a separate, configured system. Do not infer that a feature exists from an engineering roadmap checkbox, an inherited PdfCraft implementation, a command name, or a planned integration.

## Current product boundary

SimplyPDF is a **local PDF workbench**, not a construction project-management system, CAD/BIM model authoring tool, or engineering analysis service. The Rust desktop, CLI, and opt-in MCP interface may provide PDF opening, markup/review, calibrated measurements, HVAC route takeoffs, and on-demand CSV/XLSX snapshot export. Support for each action depends on the installed build, document permissions, source file, and the specific tool's input limits.

A feature is **implemented in source** only when the referenced entry point exists; it is **tested** only within the cited automated test's scope. Tests do not establish clean-install behavior, external-system compatibility, or support on every operating system. See the [project guide](project-guide.md) for detailed use and the [task checklist](../README.md) for uncompleted acceptance work.

| Claim allowed in product text | Source and regression evidence | Limit; do **not** claim |
| --- | --- | --- |
| Calibrated distance/area and independent axis scale calculations exist | [measure tests](../crates/measure/src/tests.rs): `calibrated_lengths_concave_areas_and_axis_scales`, `viewport_precedence_first_point_user_unit_and_roundtrip` | All drawing formats, engineering-grade accuracy, or automatically recognized scales |
| Snapping supports selected geometry including endpoints, paths, midpoints and intersections | [measure tests](../crates/measure/src/tests.rs): `snap_paths_endpoints_midpoints_and_intersections`, `snap_intersection_rejects_overflow_and_never_returns_non_finite_points` | Complete layer-aware CAD constraints or arbitrary curves |
| CSV export escapes formula-like cell prefixes; XLSX snapshot has row bounds and no-overwrite behavior | [measure tests](../crates/measure/src/tests.rs): `measurement_csv_quotes_newlines_and_blocks_formula_cells`; [XLSX tests](../crates/measure/src/xlsx.rs): `excel_row_bounds_allow_last_measurement_and_reject_overflow`, `empty_snapshot_is_a_valid_xlsx_and_never_replaces_an_existing_file` | Live spreadsheet synchronization, total protection from hostile spreadsheet software, or every-platform release acceptance |
| Headless numbered editable annotations can be added and saved/reopened | [automation tests](../crates/automation/src/tests.rs): `numbered_symbols_are_editable_and_undo_as_one_batch`, `numbered_symbols_survive_save_reopen_without_renumbering` | A completed desktop numbered-symbol tool, globally persistent counter, or complete ENG-03.04 |
| A basic desktop PDF UI opens documents and displays comments/bookmarks | [UI integration tests](../crates/ui-egui/tests/it/ui.rs): `opening_a_pdf_shows_comments_and_bookmarks` | A production-certified Windows/macOS/Linux installer or universal file support |

The names above are **traceable source-level test references**, not a claim that a particular release's tests have passed. Only assert a passing build after reviewing its own GitHub Actions result.

## Adjacent construction workflows: supported boundary versus separate system

| Workflow | SimplyPDF scope | Separate capability **not** provided by the currently verified product |
| --- | --- | --- |
| RFIs | Annotate a drawing or export supporting PDF/markup evidence where supported; a reference to an RFI number is document metadata, not an RFI record | RFI submission, routing, assignment, official response, due dates, permissions, audit trail, and project-system synchronization |
| Submittals | Review and mark up an imported PDF; produce a local PDF deliverable where supported | Submittal packages, specification registers, approval workflows, revision issuance, distribution, and authoritative submittal tracking |
| Punch lists / field observations | Record local PDF comments, statuses, or numbered symbols where available | Company-wide issue register, mobile field capture/synchronization, responsible-party assignment, SLA tracking, formal closeout, and project-management reporting |
| Estimates | Calculate local drawing quantities and export **snapshot** CSV/XLSX values to a separate workbook | Pricing database, bid leveling, labor/material cost engine, automatic bid updates, bid management, or a managed live Calc/Excel Quantity Link |
| CAD authoring | Review and annotate a rendered PDF drawing, or create supported PDF markup geometry | Native DWG/DXF editing, parametric drafting, CAD model modification, constraint solving, or CAD drawing round-trip |
| Engineering solvers | Measure geometry under a user-selected scale and export quantities | Structural, mechanical, civil, electrical, hydraulic, thermal, or code-compliance calculations; engineering stamps or certification |
| BIM coordination / clash detection | Review provided PDF sheets and record observations | Native Revit/IFC coordination, element-level clash detection, federated model alignment, conflict resolution, or model-to-sheet issue synchronization |
| Signatures and seals | Place supported user-provided visible review assets where an available tool permits | Independently validated cryptographic signing, professional licensure, certified seals, or a legal signing workflow unless separately implemented and verified |

A PDF hyperlink, free-text annotation, exported spreadsheet, or planned integration is **not** evidence of an external service connector. A successful API proof of concept is **not** evidence of a production integration. Do not publish service logos, badges, customer claims, or app-store/platform claims without separately verified rights and acceptance tests.

## Use and privacy

- Document operations are local by default. The MCP server is **opt-in**; running the CLI manually does not register a system-wide background listener. Configure the drawing root deliberately and review [MCP conventions](mcp.md) before granting tool access.
- Do not upload documents to model providers, external project systems, or cloud storage without explicit user configuration and an actual implemented integration.
- Treat annotations and computed quantities as review aids. Confirm drawing scale, revision, units, measurement assumptions, layer visibility and source information before using them in bids or engineering decisions.
- Do not promise accurate quantities from incomplete scans or unsupported geometry. Preserve warnings and failure status.

## How contributors qualify new product claims

1. Find the concrete shared Rust/desktop/CLI/MCP entry point and a deterministic test for the claim. Link both in the relevant feature guide or PR.
2. State the exact tested scope, affected input types, and unsupported variants. Distinguish code coverage from integration, packaging, and live service tests.
3. For any RFI, submittal, punch, estimating, CAD, solver, BIM, or external-connector feature, explicitly name the **separate** system responsible for identity, workflow, records, pricing, or engineering sign-off; do not imply SimplyPDF replaces it.
4. Preserve explicit opt-in, permission and offline limitations. No live-link or cloud claims until an end-to-end test on a configured sandbox verifies them.
5. Keep the item in [README.md](../README.md) unchecked until **all** its acceptance requirements are met. Verify changed documentation links and recorded tests in CI before merging.

## Acceptance evidence for ENG-30.04

- RFI/submittal/punch/estimate workflows are separated in the table above from dedicated management systems.
- CAD authoring, engineering solvers and BIM clash detection are each explicitly out of scope for currently verified functionality.
- Every affirmative capability claim in this document links to a source test, including known limitations; existing high-level product positioning is restricted to a PDF workbench.
- This is a **documentation boundary**, not implementation of the many engineering backlog items or a cross-platform release sign-off.
