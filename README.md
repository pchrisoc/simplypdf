# SimplyPDF

SimplyPDF is a Rust desktop PDF workbench for construction takeoff and estimating, based on
[PdfCraft](https://github.com/storytold/pdfcraft). The desktop targets Windows, macOS, and Linux.
Agents can use the same measurement tools through the CLI or an opt-in MCP server.

**Upstream credit:** SimplyPDF builds on [PDFCraft](https://github.com/storytold/pdfcraft) by the
ArtCraft team and PDFCraft contributors. [Planned project website](https://pdf.simplyhost.org)
(not live yet).

## What works today

- Calibrate a drawing from two points or a scale ratio, then measure distance, perimeter, and area.
- Save measurements as PDF annotations and export their quantities as numeric XLSX cells.
- Use MCP to open drawings, create and list measurements, and export a takeoff workbook.

In **Measure → Common scales**, choose **Full size** or **Half size** before picking an architectural,
engineering, or metric scale. The presets include `1/4" = 1'`, `1" = 20'`, and `1:50`.
Use **Half size** when the PDF itself was reduced to 50% of its original linear dimensions.
If the scale printed on the sheet already describes the reduced PDF, use **Full size** with
that scale. For other reductions, calibrate against a known dimension.

MCP accepts the preset label and sheet size through `measure_scale`, for example
`{"doc":1,"page":1,"preset":"1/4\" = 1'","sheet_size":"half"}`.

For a drawing stretched in one direction, enter its horizontal scale and a **Vertical scale
multiplier** in the Measure panel. For example, a multiplier of `1.02` makes a vertical
segment read 2% longer while leaving horizontal distances unchanged. **Calibrate from two
points** starts a new uniform calibration. The MCP `measure_scale` tool accepts the same
correction as `vertical_multiplier` with a horizontal ratio.

The XLSX export is an on-demand snapshot. Live publishing to a Calc-linked workbook and
end-to-end validation on Windows and Linux are pending.

## HVAC takeoffs

Use **Continuous length** to click successive route vertices. Each click places the next
segment. The cursor preview shows the current segment and running total. Press **Enter** or
double-click to finish, **Backspace** to remove the last vertex, and **Escape** to cancel.
Hold **Shift** to lock the nearest 45° direction, then release it to choose a new direction.

The tool palette and command search include **Pipe takeoff** and **Sheet-metal takeoff**.
Shortcuts use **Ctrl** on Windows/Linux and **⌘** on macOS.

| Tool | Shortcut |
| --- | --- |
| Continuous length | Ctrl/⌘ + Shift + U |
| Pipe takeoff | Ctrl/⌘ + Shift + J |
| Sheet-metal takeoff | Ctrl/⌘ + Shift + K |

Select **Pipe takeoff** or **Sheet-metal takeoff** in Measure. The **HVAC specifications**
controls appear near the top of the tool pane. Choose pipe, round duct, or rectangular duct;
rectangular ducts have separate width and height controls.

| Route | Materials | Systems | Standard sizes |
| --- | --- | --- | --- |
| Pipe | Copper | HHW Supply, HHW Return, CHW Supply, CHW Return | 3 in, 2.5 in, then 2 in down to 0.25 in in quarter-inch steps |
| Sheet metal | Galvanized (default), Stainless steel, Aluminum | SA (supply air), R (return air), EA (exhaust air) | 4–48 in in two-inch steps, for diameter or each rectangular dimension |

Use the **− / +** buttons to move to the adjacent size without opening a menu, or select a
size from the dropdown. Pipe defaults to 3 in, Copper, HHW Supply; sheet metal defaults to
4 in, Galvanized, SA. Standard presets are enforced by both the desktop app and automation.
Copper is orange, galvanized is blue, stainless steel is purple, and aluminum is green.

Specifications are saved with the PDF measurement and exported in dedicated XLSX columns.
Quantities measure the drawn centerline. Fittings, rise/drop, insulation, duct surface area,
and fabrication allowances remain on the roadmap.

MCP exposes `takeoff_pipe` and `takeoff_duct`, with the same optional `hvac` fields
(`profile`, `material`, `size`, and `service`). `measure_update` edits a saved takeoff's
quantity, unit, label, author, HVAC specification, or visibility using the one-based page and
annotation index from `measure_list`.

## Measurement workspace

The **Takeoff table** opens at the bottom after the first measurement is saved. Drag its top border to make
it taller or shorter. Search across measurement and HVAC fields, then click any column header
to sort. Clicking the header again reverses the order. Quantities sort numerically, and the
filtered totals keep each unit separate. Selecting a row selects its markup and page.

Click a **Value**, **Unit**, **Label**, or **Author** cell to edit it directly. Press **Enter**
to save or **Escape** to cancel; leaving the field also saves. HVAC profile, material, size,
and system cells have inline preset controls. Quantity edits change the takeoff's local
conversion without moving its route geometry. Edits support undo/redo and save/reopen.

Use each row's visibility checkbox, or **Visibility** to show or hide takeoffs by profile,
material, or system. Visibility is saved in the PDF, including changes made while the table
is collapsed. Hidden takeoffs remain in the table and quantity totals.

Use **Table placement** to attach the table to the bottom, left, or right, or open it in a
floating window. Drag a floating window by its title bar and resize its edges. The table
contains saved measurements and HVAC routes.

In Measure, the docking icons move the controls to the left sidebar, right sidebar, or a
floating window. Positions and the bottom table height are remembered. Scale presets and HVAC
properties use dropdowns. Expand **Custom scale** or **Snap options** for additional controls.

The far-left tool navigator lists pinned tools first, then other tools opened in this session.
Collapse it to icons or expand it to show labels. The active tool's settings occupy a separate
column immediately to its right; collapse those settings independently with **Toggle tool
settings**. Clicking another tool icon opens its settings in that column. A dot marks the
active tool, and pinned tools appear only in the pinned section. Use **Browse all tools** for
the complete catalog. Right-click a tool to pin it. Drag a tool row, catalog tile, or measurement action onto the top mode bar to save
a quick-access icon. Right-click an icon to remove it. Quick-access icons persist.

Use **Export takeoff…** in the takeoff bar to choose XLSX or CSV before selecting a file.

The opt-in UI control channel supports `ui.set` keys `measurement-settings`,
`takeoff-table`, `takeoff-search`, `takeoff-sort`, `tool-rail`, `tool-settings`, and `quick-tool`.
Use `tool-rail=icons|expanded` and `tool-settings=collapsed|expanded`. Quick-tool values are a
catalog group ID or `command:<command-id>`; prefix `remove:` to remove a shortcut. Sort values are column names such as
`Value` or `-Value` for descending order.

## SimplyPDF roadmap

The first release targets a local estimating workflow. Open a plan, calibrate it, take off
quantities, review the underlying markups, and see the estimate update in Calc. The desktop app,
CLI, MCP server, and workbook publisher will share Rust implementations. Calc is a separate
application. Check off each item after its acceptance test passes and push the checkpoint to
`main` on the [SimplyPDF repository](https://github.com/pchrisoc/simplypdf).

The [engineering feature checklist](#engineering-feature-checklist) extends these first-release
checkpoints to the complete engineering workflow inventory. Agents follow the
[completion and push loop](#agent-completion-and-push-loop) for every task.

### Checkpoint 0 · Establish the baseline

- [x] Identify the local Revu 21.11.0 installer with REA. Static inspection identified an x86
  Windows PE wrapper with 2,597,057,624 bytes after its last mapped section. SHA-256
  `ca9e54e230b9a97b586094e35f13bb1d3a5694ebe6e21d8b59ee54107344d622`.
  The package layout establishes the inspection boundary. Application behavior still requires
  observing installed Revu on Windows.
- [x] Map the documented workflow. Revu combines page and viewport calibration, saved
  measurements and counts, a sortable Markups List, reusable tools, and filtered measurement
  totals linked into Excel. [Measurements](https://support.bluebeam.com/user-manual/menus/window/measurements-panel.html) ·
  [Markups List](https://support.bluebeam.com/user-manual/menus/window/markups-list.html?highlight=markups+list) ·
  [Tool Chest](https://support.bluebeam.com/revu/how-to/create-and-save-custom-markups-in-tool-chest.html) ·
  [Quantity Link](https://support.bluebeam.com/revu/how-to/tips-and-tricks/improve-takeoffs-with-quantity-link.html)
- [x] Verify the existing Rust measurement and MCP baseline on macOS, including numeric XLSX
  snapshot export and protection against overwriting an existing workbook.
- [ ] Observe Revu on Windows with synthetic drawings. Record how calibration, counts, filters,
  save/reopen, and quantity refresh behave when a markup is edited, deleted, or undone.

Use public specifications, public documentation, and black-box observations to implement
features. Keep the proprietary installer and any captured artifacts outside the repository.

### Checkpoint 1 · Publish live takeoff quantities to Calc

- [x] Share the [Rust workbook writer](crates/measure/src/xlsx.rs) between desktop export and
  CLI/MCP. Both produce the same typed rows and `TakeoffData` range.
- [ ] Define persistent measurement IDs and a versioned row schema for saved and imported
  markups. Verify IDs survive edits, page reordering, save/reopen, and undo/redo.
  Partial (ENG-08.01): CSV/XLSX schema v1 now includes annotation IDs and HVAC properties;
  imported missing/duplicate IDs still require normalization.
- [ ] Add an opt-in managed workbook link to the [Measure panel](crates/ui-egui/src/tools/measure_ui.rs)
  and matching MCP start, status, refresh, and stop tools. Publish the same document state for
  edits from either entry point.
- [ ] Publish after add, edit, delete, undo, and redo. Replace only the dedicated workbook
  created by SimplyPDF. Retain the last good file and report stale status when publication
  fails. Retry transient file locks and surface incomplete measurement scans.
- [ ] Link the named range from a separate Calc estimate. Verify a formula updates after an
  addition and undo, including when the range shrinks or becomes empty. Verify estimate formulas
  and pricing survive refresh on Windows, macOS, and Linux.
  [Calc external links](https://help.libreoffice.org/latest/en-US/text/scalc/01/04090000.html)

### Checkpoint 2 · Make takeoffs reviewable and repeatable

- [x] Refine measurement settings into icon rows, dropdowns and expandable advanced controls.
- [x] Open takeoffs after the first saved measurement and move CSV/XLSX export into a format chooser.
- [x] Add mode-bar drag-and-drop shortcuts and a pinned/open tools sidebar with active indicators.
- [x] Split the tool navigator and settings into independently collapsible columns.
- [x] Edit takeoff quantities and specifications directly in the table, with undo and save/reopen.
- [x] Show or hide takeoffs individually or by profile, material, and system.
- [x] Constrain HVAC presets to standard materials, systems, and sizes, with material colors and
  adjacent-size buttons.
- [x] Remove the top-right Discord promotion.

- [x] Add a resizable bottom takeoff table with search, column sorting, unit-separated totals,
  and selection linked to the drawing.
- [x] Support left/right docking and floating measurement settings and takeoff table windows.
- [x] Add continuous route takeoff with segment/total preview and a held Shift direction lock.
- [x] Add pipe and duct presets, shortcuts, nominal sizes, materials, and system tags saved to
  PDF measurements and exported to XLSX.
- [ ] Add fitting counts, rise/drop lengths, insulation, sheet-metal surface area, and
  fabrication allowances with explicit formulas and unit tests.
- [x] Apply separate horizontal and vertical calibration factors to distances and areas.
  Verify rotated pages and save/reopen through MCP.
- [x] Add architectural, engineering, and metric construction presets with full-size and
  half-size calibration in the desktop app and MCP.
- [ ] Add editable page and viewport scales, with a visible
  indicator for uncalibrated drawings and the scale applied to each markup.
- [ ] Make vector snaps respect drawing layer visibility. Test endpoints, midpoints,
  intersections, and path snaps on synthetic layered and rotated plans.
- [ ] Add named markup layers with visibility, assignment, and persistence. Verify the same
  visibility and assignment from the desktop and MCP.
- [ ] Add point counts with editable groups and visible totals. Verify click, delete, regroup,
  save/reopen, undo, MCP, and XLSX export.
- [ ] Extend the takeoff table with grouped totals by sheet, type, subject, and label, plus
  general markup rows and custom columns. Keep incompatible units separate.
- [ ] Add reusable takeoff presets, named spaces, and multi-sheet quantity totals. Verify a
  fixture with two scales and two trades produces the expected totals in the app, MCP, and Calc.
- [ ] Add area cutouts and depth-based volume after the core quantities and sync pass.

### Checkpoint 3 · Ship the desktop workflow

- [x] Rename app, CLI, crates, application IDs, executable names, and installer metadata to SimplyPDF.
  Preserve upstream credits, asset provenance, and legacy file/settings compatibility.
- [ ] Finish original release artwork and verify installed SimplyPDF builds can coexist with PDFCraft
  on Windows, macOS, and Linux.
- [ ] Package SimplyPDF for Windows, macOS, and Linux. Test opening a PDF and starting the opt-in
  MCP server from a clean install on each platform.
- [ ] Run a release smoke test on each platform using the same synthetic drawing and linked
  Calc estimate. Publish the verified builds with the test results and known gaps.

## Engineering feature checklist

This is the implementation and verification backlog for SimplyPDF, derived from the engineering
workflow inventory discussed on 2026-11-05. Bluebeam is a functional reference, not a dependency
or a source of code, artwork, or assets. Use public documentation, PDF specifications, and
synthetic black-box observations. Implement original Rust behavior and our existing shared UI.

**Status:** every new `ENG-*` item starts unchecked because this expanded inventory has not been
audited end to end. An unchecked item can already have a partial or inherited implementation:
inspect and extend it rather than duplicate it. Existing checked first-release checkpoints above
remain historical records. Checking an engineering item requires the evidence below, including
for inherited code. These boxes are not a claim of Bluebeam parity or release readiness.

Each checkbox supplies an implementation route and an observable acceptance condition. Every
item also inherits the shared definition of done. Preserve task IDs when moving or splitting
work; leave a parent unchecked until all of its requirements pass.

### Agent completion and push loop

1. Read `AGENTS.md`, `CLAUDE.md`, this checklist, the existing checkpoints, and relevant crate
   READMEs. Inspect existing tools, tests, issues, and work before choosing the next task.
   The local-only `plan/` directory may be absent; this README is the SimplyPDF backlog.
2. Use `main` as the SimplyPDF integration branch. Fetch `simplypdf`, inspect its URL, reconcile
   incoming changes, and begin from the current `simplypdf/main`. The writable project is
   `https://github.com/pchrisoc/simplypdf.git`; `origin` is the PdfCraft upstream, not this
   project's publish target. Do not push this fork's changes to upstream.
3. Choose the next unblocked first-release checkpoint, then an `ENG-*` task in dependency order.
   Prioritize crashes and data loss. Take one reviewable increment at a time; record dependencies
   and blockers beside the unchecked item. A missing credential or unavailable platform does
   not justify checking a box.
4. Implement through shared Rust engine APIs, desktop controls, and CLI/MCP tools. For a UI-only
   preference, provide headless inspection/control without inventing a document mutation API.
   Keep all servers and external connections opt-in. Never discard unrelated work.
5. Verify the task-specific acceptance condition and the definition of done. Add evidence beside
   the item: relevant file/tool links, exact passing test names or commands, tested platforms,
   and any bounded limitations. A code stub, screenshot alone, or generic open/save test is not
   completion evidence. If only part passes, record partial progress and keep `[ ]`.
6. Change `[ ]` to `[x]` only after that evidence passes. Update the corresponding first-release
   checkpoint when applicable, `parity/acrobat-features.toml` where applicable, `ROADMAP.md`, and
   `docs/roadmap.md`. Do not inflate upstream Acrobat parity numbers for new engineering work.
7. Review the diff, run the required checks, commit the code and its checked box together using
   the task ID, and push each verified checkpoint with `git push simplypdf main`. Fetch and
   reconcile if rejected; routine work must not force-push or overwrite concurrent changes.
   Work done on a temporary branch must be integrated into `main` before this checkpoint is done.
8. Verify GitHub's `main` contains the checkpoint, then continue with the next unblocked item
   during the active work session. If a push fails, keep the local commit, report the error,
   and retry safely; never report it as published. These instructions guide future agent
   sessions; they do not install or start a background agent or scheduler.

The owner's 2026-11-05 authorization permits direct checkpoint pushes to **SimplyPDF's** `main`.
The one-time replacement of the older Draw prototype must preserve its tip on an archive
branch first and use an explicit lease. That authorization is not a standing instruction to
rewrite history on subsequent iterations. Upstream contributions still use PRs.

### Shared definition of done

- Implement real behavior, with actionable errors for malformed inputs, allocation limits,
  permission checks, atomic output, and `--root` confinement. No new panic or unsafe paths.
- For document edits, verify undo/redo, save/reopen, rotated pages, units, unknown-data
  preservation, and unchanged unrelated content as applicable. Use small synthetic fixtures;
  commit no proprietary drawings or assets. New allowed assets require attribution.
- Expose user-facing behavior through the engine and `crates/automation` tool table, with an
  end-to-end test. Exercise it using `simplypdf-cli run --script` or MCP and inspect `page_render`.
  For UI changes, use opt-in `--control` with CLI inspect/click/screenshot and look at the result.
- Run `cargo fmt --check`, scoped `cargo clippy -p <touched-crate> -j 2 --all-targets -- -D warnings`,
  scoped `cargo test -p <touched-crate> -j 2`, `cargo xtask layers`, `cargo xtask assets`, and the
  applicable WASM check. Bound build/test duration; do not launch a full workspace build by
  default. Follow any additional gates in `CLAUDE.md`; document unavailable checks honestly.
- Test every claimed platform. Keep platform-specific tasks unchecked if the required platform
  is unavailable. For connectors, test with an explicitly configured account and sandbox data;
  a mock-only connector is not a completed integration.
- Document how a user invokes the feature, how an agent invokes it, and its limitations. Attach
  passing evidence to the checkbox before committing it as complete.

### ENG-01 · Drawing viewing and navigation

- [ ] **ENG-01.01 — Engineering document viewing.** Reuse the renderer and document session to
  open multipage drawings, specifications, calculations, reports, and manuals. Verify mixed
  page sizes, encrypted access, large sheets, cancellation, and malformed-file errors.
- [ ] **ENG-01.02 — Sheet navigation.** Connect thumbnails, labels, bookmarks, and hyperlinks to
  one navigation model. Verify each opens the correct page/view after page reordering and reopen.
- [ ] **ENG-01.03 — Split views.** Add horizontal/vertical splits for different sheets or two
  locations on one sheet. Verify independent scrolling, focus, selection, and closing a split.
- [ ] **ENG-01.04 — Synchronized views.** Link supported zoom/pan/page actions between splits with
  an explicit toggle. Verify alignment with different sheet sizes and safe unlinking.
- [ ] **ENG-01.05 — Multiple windows and monitors.** Detach documents and restore window layouts.
  Verify moving between monitors, display scaling, focus, close/save prompts, and missing monitors.
- [ ] **ENG-01.06 — Viewing modes.** Implement view-only rotation, full-screen/presentation modes,
  and line-weight/display controls. Verify view rotation does not alter saved page geometry.
- [ ] **ENG-01.07 — File access.** Add recent files and user-organized pinned locations. Verify
  missing files, clearing history, persistence, and no disclosure of recent files in test captures.

### ENG-02 · Engineering redlines and annotations

- [ ] **ENG-02.01 — Text review tools.** Support text boxes, typewriter notes, leader callouts,
  and insert/delete/replace text comments. Verify Unicode, multiline appearances, and text anchoring.
- [ ] **ENG-02.02 — Clouds and pointers.** Implement revision clouds, cloud-plus-callout groups,
  arrows, lines, and dimension annotations. Verify endpoints, cloud resizing, and saved appearances.
- [ ] **ENG-02.03 — Shape tools.** Support rectangles, ellipses, polygons, polylines, and arcs.
  Verify vertex editing, closure, degenerate shapes, and geometry on rotated pages.
- [ ] **ENG-02.04 — Freehand review.** Support pen strokes, erasing supported strokes, and text/area
  highlighting. Verify pointer cancellation, long-stroke bounds, and text-highlight quad placement.
- [ ] **ENG-02.05 — Appearance standards.** Expose colors, opacity, fills, borders, line weights,
  arrowheads, custom line styles, and hatches. Verify preview, saved defaults, and PDF rendering.
- [ ] **ENG-02.06 — Geometry editing.** Move, resize, rotate, and edit vertices/curves/control
  points. Verify hit targets and reversible edits at different zooms and page rotations.
- [ ] **ENG-02.07 — Multi-annotation operations.** Align, distribute, group, ungroup, arrange,
  copy, duplicate, and lock annotations. Verify group transforms and locked-item protection.
- [ ] **ENG-02.08 — Flattening.** Flatten selected annotations into page content with an explicit
  recovery option where supported. Verify identical appearance and honest recovery limitations.
- [ ] **ENG-02.09 — Image and stamp placement.** Place images and stamps as review annotations.
  Verify cropping/transforms, transparent appearance, provenance, and reopening in another reader.

### ENG-03 · Reusable tools and engineering symbols

- [ ] **ENG-03.01 — Tool library.** Save named custom tools in discipline-specific sets; import,
  export, and share a versioned format. Verify round trips, unknown versions, and corrupt inputs.
- [ ] **ENG-03.02 — Reuse modes.** Separate appearance-only reuse from full saved geometry;
  preserve default subjects/comments and grouped assemblies. Verify each mode creates editable tools.
- [ ] **ENG-03.03 — Scale-aware tools.** Resize calibrated tool geometry for page and viewport
  scales. Verify the same real-world equipment footprint across two differently scaled details.
- [ ] **ENG-03.04 — Numbered symbols.** Add sequential labels with editable prefixes and counters.
  Verify copying, deleting, undo, and restart do not unexpectedly renumber existing annotations.
- [ ] **ENG-03.05 — Snapshot symbols.** Capture selected PDF regions into reusable annotations
  while preserving supported vector content. Verify clipping, recoloring, and source independence.
- [ ] **ENG-03.06 — Legends.** Generate and update legends from selected annotation classes and
  quantities; save legend presets. Verify filtering, changed counts, layout, and saved output.

### ENG-04 · Scaled sketches and proposed changes

- [ ] **ENG-04.01 — Sketch to scale.** Add dimension-driven rectangles, ellipses, polygons, and
  polylines using the measurement calibration model. Verify specified real-world dimensions.
- [ ] **ENG-04.02 — Precision construction aids.** Share snapping, alignment, direction locks,
  and supported centroid display. Verify numerical geometry independently of screen zoom.
- [ ] **ENG-04.03 — Proposed layout sketches.** Combine scaled footprints, routes, openings, and
  clearance envelopes in editable groups. Verify a synthetic equipment-layout review workflow.
- [ ] **ENG-04.04 — Offsets.** Offset supported lines and closed geometry by a specified distance.
  Define joins and invalid/self-intersection behavior; verify concave shapes and unit conversion.
- [ ] **ENG-04.05 — Annotation conversions.** Convert or duplicate compatible shapes, routes, and
  measurements. Publish a compatibility table; verify geometry/metadata retention and safe refusals.

### ENG-05 · Calibration and measurement accuracy

- [ ] **ENG-05.01 — Known-distance calibration.** Unify two-point calibration, preset/custom
  ratios, and saved presets. Verify full-size/half-size sheets and a known reference dimension.
- [ ] **ENG-05.02 — Page and viewport scales.** Edit scales per page and named viewport, apply
  them to page ranges, and show each markup's effective scale. Verify overlapping/missing viewports.
- [ ] **ENG-05.03 — Independent axes.** Preserve horizontal/vertical calibration and rotated-page
  transforms. Verify anisotropic distances and areas against analytical expected values.
- [ ] **ENG-05.04 — Units and precision.** Configure metric/imperial units, decimal/fractional
  captions, and rounding. Verify stored values retain precision independently of displayed rounding.
- [ ] **ENG-05.05 — Vector snaps and constraints.** Snap to visible endpoints, midpoints,
  intersections, and paths; constrain directions. Verify hidden layers and raster-only pages.
- [ ] **ENG-05.06 — Measurement feedback.** Show placement previews, configurable captions, and
  uncalibrated indicators. Verify displayed values match saved geometry and exported quantities.
- [ ] **ENG-05.07 — Scale changes.** Define explicit recalculation for existing measurements and
  protected/read-only documents. Verify permission failures and undo preserve original quantities.

### ENG-06 · Measurement geometry and derived quantities

- [ ] **ENG-06.01 — Linear tools.** Complete length, continuous polylength, and perimeter with
  per-segment captions. Verify Enter/double-click completion, Backspace, Escape, and exact totals.
- [ ] **ENG-06.02 — Area geometry.** Measure rectangular, irregular, and curved regions. Verify
  winding, closure, degenerate geometry, and analytical rectangle/circle reference cases.
- [ ] **ENG-06.03 — Circular and angular tools.** Add diameter, center radius, three-point radius,
  and angle measurements. Verify collinear points, zero radius, orientation, and unit precision.
- [ ] **ENG-06.04 — Area cutouts.** Subtract polygonal/elliptical openings from measured regions.
  Verify overlapping holes, exterior holes, nested cases, and no double subtraction.
- [ ] **ENG-06.05 — Volume and wall area.** Derive quantities from entered depth/height with
  explicit formulas. Verify mixed units, openings, zero/negative input policy, and XLSX values.
- [ ] **ENG-06.06 — Slope adjustments.** Accept pitch, degrees, and grade with a clear reference
  plane. Verify equivalent inputs produce identical surface/length corrections and reject singularities.
- [ ] **ENG-06.07 — Rise/drop routes.** Add explicit vertical allowances to pipe/conduit routes.
  Verify displayed route length, rise/drop, and combined quantity remain separately auditable.
- [ ] **ENG-06.08 — Related values and totals.** Show supported length/area/perimeter/wall/volume
  values per markup and selected/document totals. Verify edits recompute every dependent value.
- [ ] **ENG-06.09 — HVAC quantity extensions.** Add fitting counts, insulation, duct surface area,
  and fabrication allowances to existing pipe/duct presets. Verify formulas, dimensions, and exports.

### ENG-07 · Counts and assisted takeoff

- [ ] **ENG-07.01 — Count tools.** Add editable count groups, symbols, component types, and running
  totals. Verify click/delete/regroup, undo, save/reopen, automation, and workbook output.
- [ ] **ENG-07.02 — Quantity classification.** Classify counts/measurements by subject, label,
  layer, material, service, and custom fields. Verify grouping never combines incompatible units.
- [ ] **ENG-07.03 — Symbol matching.** Implement bounded visual search with adjustable matching
  and reviewable results. Verify rotated/scaled symbols, false positives, and cancellation.
- [ ] **ENG-07.04 — Search-result actions.** Convert reviewed matches into counts or annotations
  as a reversible batch. Verify duplicates, ignored results, and partial/error handling.
- [ ] **ENG-07.05 — Region fill.** Implement assisted bounded-region selection with temporary
  barriers and preview. Verify gaps, noisy scans, fill limits, and user correction before applying.
- [ ] **ENG-07.06 — Fill outputs.** Generate selected measurements or named Spaces from approved
  fills. Verify boundaries, cutouts, scale, and subsequent editable geometry.

### ENG-08 · Calculations and workbook links

- [ ] **ENG-08.01 — Persistent row identity.** Add stable measurement IDs and a versioned export
  schema. Verify edits, imports, reordered pages, undo, and reopen retain correct identity.
  Partial: [schema v1](crates/automation/README.md#takeoff-schema-v1) is implemented;
  imported missing/duplicate `/NM` normalization and identity lifecycle evidence remain pending.
  macOS evidence: `cargo test -p simplypdf-automation -j 2 takeoff_` covers
  `takeoff_exports_declare_schema_and_share_identity_and_hvac_columns` and
  `takeoff_empty_exports_keep_headers_and_schema_metadata`; CLI rotated-page
  CSV/XLSX/save/`page_render` smoke verifies the same 120 ft quantity in both exports.
  Scoped Clippy, measurement/automation tests, fmt, assets, layers, and WASM compilation pass
  (five inherited renderer WASM warnings; native-only XLSX reports a CSV alternative in browser builds).
- [ ] **ENG-08.02 — Quantity formulas.** Add numeric/formula fields and unit-aware material/rate
  inputs. Verify dependencies, cycles, invalid expressions, and no silent unit conversion errors.
- [ ] **ENG-08.03 — Classification lists.** Share editable choice lists for materials, cost codes,
  rates, and systems. Verify importing choices and handling values absent from the current list.
- [ ] **ENG-08.04 — Snapshot exports.** Verify CSV and typed XLSX exports share engine rows and
  numeric quantities. Check escaping, named ranges, empty data, and existing-file protection.
  Partial audit: shared schema/empty-export coverage is recorded under ENG-08.01. CSV and
  XLSX now share the engine copying-permission guard; unrestricted CSV replacement policy
  still needs reconciliation with the fresh-path workbook policy before this box is checked.
  macOS evidence: `takeoff_exports_honor_copy_permissions_and_preserve_outputs` (observed
  failing before the fix) and `copy_restricted_documents_refuse_extraction_unless_opened_by_the_owner`
  pass. Native `--control` inspect/command/screenshot verifies disabled takeoff export while
  restricted measurements remain visible; CLI refusal creates no file. The desktop
  `measurement_tools_draw_live_calibrate_save_and_export` regression, scoped engine/automation/UI
  strict Clippy and WASM checks, native builds, fmt, assets, and layers pass (existing WASM warnings).
- [ ] **ENG-08.05 — Managed workbook publishing.** Add opt-in start/status/refresh/stop controls
  and tools for a dedicated data workbook. Verify atomic publication and last-good-file retention.
- [ ] **ENG-08.06 — Live refresh.** Publish after add/edit/delete/undo/redo; retry transient locks
  and report stale/incomplete scans. Verify shrinking and empty datasets without stale rows.
- [ ] **ENG-08.07 — Filtered multi-PDF quantities.** Link totals by subject, label, author, layer,
  color, and custom fields across source PDFs. Verify source identity and distinct unit totals.
- [ ] **ENG-08.08 — Calc estimate workflow.** Link a separate Calc estimate to `TakeoffData`.
  Verify additions/deletions refresh while pricing/formulas survive on Windows, macOS, and Linux.
- [ ] **ENG-08.09 — Excel workflow.** Provide and document a supported Excel live-link method
  equivalent in outcome to quantity-linked cells. Verify refresh/filter changes in actual Excel;
  exporting XLSX alone does not complete this item.

### ENG-09 · Review register and issue tracking

- [ ] **ENG-09.01 — General annotation register.** Extend the takeoff table to all supported
  annotations with author, page, subject, comments, timestamps, and properties. Verify completeness.
- [ ] **ENG-09.02 — Register navigation.** Sort, group, search, and filter rows; save reusable
  filters. Verify selecting a result reveals the correct page and annotation.
- [ ] **ENG-09.03 — Custom fields.** Implement text, number, date, checkbox, choice, and formula
  columns. Verify typed persistence, validation, schema migration, and exported types.
- [ ] **ENG-09.04 — Comment threads.** Add replies with stable authorship/thread IDs. Verify
  imports, deleted parents, order, and persistence without flattening away discussion data.
- [ ] **ENG-09.05 — Status models.** Define custom review statuses and record status history.
  Verify allowed transitions, actor/time records, undo policy, and saved reports.
- [ ] **ENG-09.06 — Review metadata.** Supply configurable discipline, assignee, priority,
  specification, tag, and package fields. Verify filters and outstanding-item reports use them.
- [ ] **ENG-09.07 — Annotation exchange.** Import/merge supported PDF/FDF/XFDF annotations with
  conflict/duplicate handling. Verify geometry, threads, IDs, and unsupported-data warnings.

### ENG-10 · Spaces and layers

- [ ] **ENG-10.01 — Named regions.** Create editable Spaces for rooms, zones, and floors, with
  nesting. Verify boundary containment, overlaps, renaming, and saved geometry.
- [ ] **ENG-10.02 — Location association.** Associate annotations with Spaces and report by
  location. Verify moved annotations, changed boundaries, and nested-space precedence.
- [ ] **ENG-10.03 — Room import.** Map supported Revit room metadata to Spaces through an
  explicitly configured adapter. Verify units/transforms and report unmatched or unsupported rooms.
- [ ] **ENG-10.04 — Annotation layers.** Create named layers and assign/move annotations. Verify
  visibility, persistence, undo, and consistent desktop/automation results.
- [ ] **ENG-10.05 — Discipline isolation.** Show/hide relevant PDF and annotation layers without
  losing quantities. Verify layer-aware snapping, rendering, exports, and saved visibility policy.

### ENG-11 · Comparison and overlays

- [ ] **ENG-11.01 — Revision comparison.** Compare original/revised sheets and create navigable
  difference annotations. Verify known additions, removals, and unchanged regions on synthetic plans.
- [ ] **ENG-11.02 — Comparison options.** Add sensitivity/rendering presets for vector and scanned
  sources plus annotation inclusion controls. Verify noise suppression without hiding seeded changes.
- [ ] **ENG-11.03 — Alignment.** Offer automatic alignment and editable registration/offsets.
  Verify translations, rotations, scale changes, failed alignment, and visible confidence/errors.
- [ ] **ENG-11.04 — Color overlays.** Overlay two or more sheets using configurable colors and
  layer appearance. Verify alignment and saved results expose known cross-discipline differences.
- [ ] **ENG-11.05 — Batch revision review.** Match multiple sheets and batch compare/overlay them.
  Verify unmatched sheets, duplicate labels, per-sheet failures, and a complete result manifest.

### ENG-12 · Sets and drawing revision control

- [ ] **ENG-12.01 — Virtual drawing Sets.** Navigate separate PDFs as one collection without
  merging sources. Verify relative paths, missing files, ordering, and moved Set folders.
- [ ] **ENG-12.02 — Tags and revision groups.** Categorize sheets, group revisions, and distinguish
  current/superseded sheets. Verify duplicate numbers and explicit current-revision selection.
- [ ] **ENG-12.03 — Drawing logs.** Export sheet numbers, titles, disciplines, revisions, and
  source paths. Verify logs match the selected Set and flag missing metadata.
- [ ] **ENG-12.04 — Slip-sheet matching.** Preview page matches using labels/regions/user rules.
  Verify ambiguous or missing matches require resolution before replacement.
- [ ] **ENG-12.05 — Revision replacement.** Batch insert/replace sheets with configurable markup
  transfer and superseded-sheet retention. Verify rollback, links, transforms, and retained originals.

### ENG-13 · PDF assembly and page management

- [ ] **ENG-13.01 — Assemble packages.** Combine PDFs and insert existing or blank pages. Verify
  forms, bookmarks, labels, links, resources, permissions, and mixed page sizes survive.
- [ ] **ENG-13.02 — Page operations.** Extract, delete, reorder, and rotate pages; split by supported
  rules. Verify page references, undo, output conflicts, and unchanged original source files.
- [ ] **ENG-13.03 — Crop and resize.** Edit page boxes, resize pages, and deskew scans. Verify
  content, annotation, viewport, and link coordinates follow the documented transformation policy.
- [ ] **ENG-13.04 — Replace page content.** Retain annotations while replacing underlying sheets.
  Verify differing page dimensions, scale, resources, and user-visible transfer warnings.
- [ ] **ENG-13.05 — Package identification.** Add/edit headers, footers, numbering, and automatic
  page labels. Verify ranges, rotations, collisions, and no changes to unselected pages.
- [ ] **ENG-13.06 — Attachments and layered pages.** Embed files and insert pages as layers.
  Verify extraction, names, visibility, unsupported inputs, and output confinement.
- [ ] **ENG-13.07 — File-size reduction.** Offer fidelity-aware optimization presets and preview
  savings. Verify readable drawings, retained metadata policy, signatures, and no silent loss.

### ENG-14 · Editing, search, OCR, and conversion

- [ ] **ENG-14.01 — Content editing.** Edit text and erase/cut selected content through existing
  APIs. Verify font handling, geometry, unknown content retention, and save/reopen fidelity.
- [ ] **ENG-14.02 — Color processing.** Recolor supported content with scoped previews. Verify
  image/vector/text coverage and prevent changes to unrelated pages or resources.
- [ ] **ENG-14.03 — Permanent redaction.** Remove selected visible/hidden content and sanitize
  affected resources. Verify extraction and rendered pixels cannot recover redacted synthetic data.
- [ ] **ENG-14.04 — OCR.** Make scanned engineering text searchable/selectable with language and
  region options. Verify a measured synthetic corpus, skew, searchable output, and failure reporting.
- [ ] **ENG-14.05 — Document text search.** Search pages, open files, folders, and Sets with bounded
  results. Verify Unicode, matches at boundaries, incomplete scans, and cancellation.
- [ ] **ENG-14.06 — Search and replace.** Preview text replacements and supported result actions.
  Verify page scope, font availability, dry run, and atomic failure before mutation.
- [ ] **ENG-14.07 — Office export.** Export supported Word/Excel/PowerPoint equivalents, including
  OCR-derived schedules. Verify editable structure, numeric cells, and explicit fidelity limitations.
- [ ] **ENG-14.08 — PDF creation.** Implement supported Office/file-to-PDF and Windows virtual
  printer workflows with safe adapters. Verify clean installs, cancellation, and matching outputs.
- [ ] **ENG-14.09 — Image output.** Export supported raster formats with resolution/page controls.
  Verify dimensions, color/transparency policy, large-page limits, and output filenames.

### ENG-15 · Navigable engineering packages

- [ ] **ENG-15.01 — Bookmarks and labels.** Manually edit and generate them from page content or
  regions. Verify title extraction, hierarchy, destination accuracy, and undo/save/reopen.
- [ ] **ENG-15.02 — Linked contents.** Generate a table of contents from bookmarks with clickable
  destinations. Verify page insertions do not leave incorrect references.
- [ ] **ENG-15.03 — Link authoring.** Link regions/annotations to pages, named views, files, and
  URLs. Verify relative paths, broken targets, navigation history, and explicit external opening.
- [ ] **ENG-15.04 — Batch links.** Detect sheet/detail references and preview generated hyperlinks.
  Verify duplicates, ambiguous references, exclusions, and reversible application.
- [ ] **ENG-15.05 — Link preservation.** Remap links during combine/split/replace and supported
  CAD publishing. Verify every internal destination in a synthetic assembled handover package.

### ENG-16 · Shared review Sessions

- [ ] **ENG-16.01 — Session model.** Design an opt-in shared review service with authenticated
  invitations and document identity. Verify no server starts or network connection occurs by default.
- [ ] **ENG-16.02 — Concurrent markup.** Synchronize simultaneous/asynchronous annotations with
  stable authorship and conflict handling. Verify two clients editing/reconnecting without lost work.
- [ ] **ENG-16.03 — Shared discussions.** Synchronize replies, statuses, and markup alerts. Verify
  permissions, ordering, unread state, and correct navigation to the referenced annotation.
- [ ] **ENG-16.04 — Roles and permissions.** Enforce host/attendee save, print, markup, invite,
  and add-document rights on the service. Verify unauthorized clients cannot bypass UI controls.
- [ ] **ENG-16.05 — Activity records.** Record review events and generate final reports. Verify
  actor/time attribution, history integrity, and that finishing a Session preserves the review.
- [ ] **ENG-16.06 — Offline review.** Queue local work, show sync state, and reconcile reconnects.
  Verify revoked access, changed documents, conflicts, and preservation of unpushed local edits.

### ENG-17 · Shared Projects and file control

- [ ] **ENG-17.01 — Project storage.** Add opt-in storage for PDFs and other project files with
  folders/uploads. Verify quotas, large-file limits, path validation, and interrupted transfers.
- [ ] **ENG-17.02 — Checkout and versions.** Implement check-out/check-in and immutable revisions.
  Verify concurrent checkout refusal, abandoned locks, and restoring an earlier version safely.
- [ ] **ENG-17.03 — Access and sharing.** Enforce user/group permissions and explicit sharing.
  Verify revoked users, expired links, and least-privilege file access.
- [ ] **ENG-17.04 — Offline files.** Sync selected files with visible freshness/conflict state.
  Verify local edits survive remote revisions, deletions, and network failures.
- [ ] **ENG-17.05 — Project-to-review flow.** Start a Session from versioned Project PDFs and return
  reviewed results with traceable version links. Verify source identity across the complete cycle.
- [ ] **ENG-17.06 — Cross-platform access.** Expose supported Projects operations on desktop,
  browser, and mobile. Verify capabilities and permissions match a documented platform matrix.

### ENG-18 · CAD/BIM publishing

- [ ] **ENG-18.01 — CAD publishing contract.** Define versioned adapters, supported applications,
  and retained metadata. Verify capability discovery and clear errors when an adapter is absent.
- [ ] **ENG-18.02 — 2D publishing.** Add supported AutoCAD/Revit/SolidWorks PDF creation paths.
  Verify scale, fonts, page sizes, sheet selection, and actual licensed-host execution.
- [ ] **ENG-18.03 — Batch CAD publishing.** Convert selected drawings/sheets with per-file results.
  Verify cancellation, output conflicts, failures, and no modification of source CAD files.
- [ ] **ENG-18.04 — AutoCAD metadata.** Retain supported sheet sets, hyperlinks, and SHX text
  workflows. Verify searchable text and references against synthetic source drawings.
- [ ] **ENG-18.05 — Revit metadata.** Publish room/viewport transforms and sheet identity needed
  by Spaces and connected review. Verify coordinates with a synthetic multi-viewport model.
- [ ] **ENG-18.06 — 3D publishing.** Add supported AutoCAD/Revit/Navisworks 3D PDF export adapters.
  Verify model structure, properties, units, and documented host-version compatibility.

### ENG-19 · 3D PDF review

- [ ] **ENG-19.01 — Model viewing.** Audit/extend existing 3D support for rotate/spin/pan/zoom,
  cameras, and saved views. Verify bounded parsing and a synthetic 3D PDF on claimed platforms.
- [ ] **ENG-19.02 — IFC/U3D creation.** Import supported model inputs into 3D PDFs with an
  explicit support matrix. Verify units, transforms, unsupported entities, and round-trip output.
- [ ] **ENG-19.03 — Model tree.** Show hierarchy/properties with search and filters. Verify
  selected tree entries select the intended part and malformed cyclic/deep input remains bounded.
- [ ] **ENG-19.04 — Component display.** Isolate/show/hide parts and adjust supported appearance/
  transparency. Verify reset, saved-view behavior, and occluded selections.
- [ ] **ENG-19.05 — Section and display controls.** Implement supported cross-sections/render modes.
  Verify section position/orientation, reset, and exported/saved display policy.
- [ ] **ENG-19.06 — 3D comments.** Anchor annotations to model views and navigate them from the
  register. Verify camera/part association after save/reopen and unsupported-view fallbacks.
- [ ] **ENG-19.07 — Model data export.** Export filtered model-tree properties to CSV. Verify
  hierarchy identifiers, Unicode, escaping, missing properties, and numeric units.

### ENG-20 · Field evidence and inspections

- [ ] **ENG-20.01 — Photo observations.** Place/crop/annotate photographs and associate them with
  drawing observations. Verify image orientation, provenance, metadata policy, and PDF persistence.
- [ ] **ENG-20.02 — Attached media.** Attach photos, supported 360-degree images, and video to
  annotations. Verify size/type limits, playback capability, export retention, and safe opening.
- [ ] **ENG-20.03 — Inspection registers.** Configure symbols, locations, attributes, and statuses
  for observations/deficiencies. Verify a room-based inspection and outstanding-item report.
- [ ] **ENG-20.04 — Evidence summaries.** Export photo/comment reports with drawing references.
  Verify thumbnails, captions, links, pagination, and inaccessible-media errors.

### ENG-21 · Web and mobile workflows

- [ ] **ENG-21.01 — Platform capability matrix.** Define desktop/browser/iOS/Android coverage
  separately for local documents, Sessions, and Projects. Verify unsupported actions are explicit.
- [ ] **ENG-21.02 — Remote document access.** View supported Project/Session/connected files in
  browser and mobile clients. Verify authentication, permissions, large sheets, and touch navigation.
- [ ] **ENG-21.03 — Portable annotations.** Add supported annotations, replies, statuses, and
  reusable tools. Verify cross-client rendering, identity, and concurrent update handling.
- [ ] **ENG-21.04 — Portable measurements.** Support length, polylength, area, and count using
  shared calibration. Verify identical geometry/quantity on desktop, browser, iOS, and Android.
- [ ] **ENG-21.05 — Mobile forms and offline use.** Fill/save forms, sync offline documents, and
  download/export files. Verify reconnect conflicts and no loss when a device goes offline.
- [ ] **ENG-21.06 — Browser page creation.** Create PDF pages from blank layouts or original
  templates. Verify dimensions, saved content, embedded allowed fonts, and desktop interoperability.
- [ ] **ENG-21.07 — Connected save-back.** Open and save through supported document services.
  Verify version conflicts, access revocation, retries, and visible unsynchronized changes.

### ENG-22 · Forms, checklists, and stamps

- [ ] **ENG-22.01 — Form filling.** Fill supported text/check/radio/list/button/signature fields.
  Verify shared widgets, validation, read-only flags, tab order, and regenerated appearances.
- [ ] **ENG-22.02 — Form authoring.** Create/edit fields and supported calculations. Verify field
  naming, dependency order, cycles, defaults, layout, and interoperability after save.
- [ ] **ENG-22.03 — Field detection.** Preview automatically detected fields on static forms.
  Verify false positives are removable and accepted fields match synthetic checklist layouts.
- [ ] **ENG-22.04 — Form data exchange.** Import/export/merge supported form data with validation.
  Verify Unicode, missing fields, duplicate records, and no partial silent overwrite.
- [ ] **ENG-22.05 — Review stamps.** Create reusable original stamps and dynamic/interactive
  fields. Verify dates/user inputs, saved appearances, and bounded scripting where supported.
- [ ] **ENG-22.06 — Batch stamps.** Preview/apply stamps to selected pages/files. Verify rotation,
  placement, permission failures, and no mutations to unselected files.

### ENG-23 · Signatures, seals, and security

- [ ] **ENG-23.01 — Digital signing.** Use digital IDs and signature fields to sign/certify PDFs.
  Verify the signed output with an independent validator and protect private-key material.
- [ ] **ENG-23.02 — Signature inspection.** Show certificates, validity, trust, and post-signing
  modifications accurately. Verify expired/untrusted/changed cases without false valid claims.
- [ ] **ENG-23.03 — Security controls.** Apply passwords/permissions and honor them in all tools.
  Verify encrypted save/reopen, wrong passwords, and restricted export/print behavior.
- [ ] **ENG-23.04 — Seal placement.** Separate professional seal graphics from digital signatures.
  Verify allowed user-provided assets, placement/date controls, and clearly documented semantics.
- [ ] **ENG-23.05 — Batch sign and seal.** Preview and atomically publish supported multi-file
  signing/sealing results. Verify cancellation, already signed inputs, and per-file validation.
- [ ] **ENG-23.06 — Sensitive-output workflow.** Combine redaction/security/signing in a documented
  order. Verify final output contains no seeded secret and signatures validate on the final bytes.

### ENG-24 · Reports and deliverables

- [ ] **ENG-24.01 — PDF comment summaries.** Generate readable reports with annotation data and
  drawing context. Verify pagination, hyperlinks, Unicode, and allowed embedded fonts/assets.
- [ ] **ENG-24.02 — Structured exports.** Export annotations as CSV/XML with stable identifiers
  and typed values where supported. Verify escaping, large exports, and no spreadsheet injection.
- [ ] **ENG-24.03 — Filtered multi-file reports.** Filter by discipline/status/author/custom fields
  and Space across files. Verify exact inclusion and source traceability.
- [ ] **ENG-24.04 — Quantity schedules.** Produce grouped takeoff summaries with unit-separated
  totals. Verify groups against known synthetic quantities and inspect rendered report output.
- [ ] **ENG-24.05 — Review delivery package.** Bundle drawing logs, Session history, outstanding
  comments, and supporting evidence. Verify links, version references, and complete manifests.
- [ ] **ENG-24.06 — Model-data reports.** Include supported model-tree properties and view references
  in deliverables. Verify exported rows correspond to selected components and documented units.

### ENG-25 · Batch processing and scripting

- [ ] **ENG-25.01 — Batch runner.** Share job selection, dry-run previews, progress, cancellation,
  per-file results, and safe publication across operations. Verify interruption and retry behavior.
- [ ] **ENG-25.02 — Drawing batches.** Expose compare, overlay, link generation, and slip-sheet
  tasks through the batch runner. Verify sheet mapping and meaningful partial-failure reports.
- [ ] **ENG-25.03 — Document batches.** Add headers/footers, stamps, crop/resize, splitting, security,
  and signing/sealing jobs. Verify permissions and no replacement of unrelated user files.
- [ ] **ENG-25.04 — Print batches.** Select files, page ranges, scales, paper, and destinations.
  Verify print-to-file geometries and real printer handling on each claimed desktop platform.
- [ ] **ENG-25.05 — Creation/assembly batches.** Combine/convert groups of supported files with
  ordered inputs. Verify source preservation, output manifests, and unsupported-format reporting.
- [ ] **ENG-25.06 — Script manager.** Save/replay versioned automation recipes over the shared
  tool table. Verify validation, bounded execution, root confinement, and actionable failing steps.

### ENG-26 · External document and field integrations

- [ ] **ENG-26.01 — Connector foundation.** Define opt-in authentication, capability discovery,
  secret storage, retries, and version-conflict APIs. Verify logout/revocation and no secret logging.
- [ ] **ENG-26.02 — SharePoint.** Open/version/save supported documents through an explicitly
  configured connector. Verify a real sandbox round trip, permissions, and concurrent updates.
- [ ] **ENG-26.03 — ProjectWise.** Integrate supported file/review workflows via documented APIs.
  Verify checkout/version behavior against a configured sandbox installation.
- [ ] **ENG-26.04 — Office and Outlook.** Provide supported document/email-to-PDF workflows and
  attachment handling. Verify licensed-host execution, attachment selection, and cancellation.
- [ ] **ENG-26.05 — OneDrive.** Add supported browse/open/save-back behavior. Verify real sandbox
  revisions, expired access, and preservation of unsynchronized edits.
- [ ] **ENG-26.06 — Box and Dropbox.** Implement each supported connector and document its actual
  read/write coverage. Verify separate real sandbox round trips and conflicting revisions.
- [ ] **ENG-26.07 — Procore and Egnyte.** Implement documented document access/save-back capabilities
  per service. Verify permissions, document identity, and per-service sandbox evidence.
- [ ] **ENG-26.08 — DocuSign.** Add opt-in document handoff/status retrieval through a supported
  integration. Verify a sandbox signing cycle, returned document identity, and explicit submission.
- [ ] **ENG-26.09 — GoCanvas-style field links.** Link configured field records/tasks to drawing
  annotations through documented APIs. Verify record identity, status refresh, and broken links.
- [ ] **ENG-26.10 — Newforma and Autodesk Forma.** Implement documented partner workflows where
  access is available. Verify each integration independently; do not count a URL shortcut as a connector.
- [ ] **ENG-26.11 — Integration catalog.** Publish actual capabilities, supported versions,
  prerequisites, and known limits for each adapter, including Excel from ENG-08.09. Verify against
  executable integration tests rather than marketing equivalence claims.

### ENG-27 · Team workspace standards

- [ ] **ENG-27.01 — Profiles and layouts.** Save/import/export UI profiles, panel arrangements,
  and shared templates. Verify migration, missing monitors, and corrupt-settings recovery.
- [ ] **ENG-27.02 — Toolbars and shortcuts.** Customize toolbars, quick-access actions, and keyboard
  bindings. Verify conflicts, platform modifiers, persistence, and keyboard-only invocation.
- [ ] **ENG-27.03 — Shared review standards.** Distribute tool sets, annotation defaults, statuses,
  and filters without overwriting personal work. Verify versioned updates and rollback.
- [ ] **ENG-27.04 — Appearance and windows.** Audit dark mode, configurable panels, multiple windows,
  and application-instance behavior. Verify focus, readability, and independent unsaved documents.
- [ ] **ENG-27.05 — Annotation translation.** Add explicitly configured translation with source
  retention and provider disclosure. Verify Unicode, offline/errors, and no automatic document upload.

### ENG-28 · Assisted review and model-connected workflows

These target engineering outcomes comparable to the newer Bluebeam Max tools. Bluebeam lists
Smart Review and Smart Overlay as previews, with Smart Review optimized for US commercial
building work. SimplyPDF must declare its own tested scope and must not treat AI output as
engineering approval. Geometry conversion/offset tasks are ENG-04.04–05.

- [ ] **ENG-28.01 — Drawing health checks.** Detect missing/blank sheets and duplicate sheet IDs
  using configurable rules. Verify seeded failures, false positives, and reviewable findings.
- [ ] **ENG-28.02 — Tag and grid coordination.** Check configured door/fixture tags and gridline
  consistency across disciplines. Verify declared drawing conventions and explain each finding.
- [ ] **ENG-28.03 — Assisted review results.** Present proposed issues as reviewable annotations,
  dashboards, and tracked items. Verify acceptance/rejection and retain source/evidence references.
- [ ] **ENG-28.04 — Assisted set comparison.** Compare drawing sets across phases/scales with
  alignment and discipline reports. Verify known changes and low-confidence results against a corpus.
- [ ] **ENG-28.05 — Sheet stitching.** Preview automatic placement and manual registration into a
  continuous view. Verify scale, overlaps, seams, source links, and bounded resource use.
- [ ] **ENG-28.06 — AI through MCP.** Expose existing document/search/markup operations to supported
  configured clients. Verify metadata queries, actionable tool errors, root limits, and no default
  server registration/listening or outbound model calls.
- [ ] **ENG-28.07 — Revit-connected review.** Map review annotations into corresponding Revit sheets
  and model views using published metadata/adapters. Verify actual supported Revit versions,
  viewport transforms, comments/status updates, and unmapped-markup warnings.
- [ ] **ENG-28.08 — Assisted-tool qualification.** Publish measured false positives/negatives,
  supported disciplines, model/provider requirements, preview status, and platform coverage.
  Verify a reproducible synthetic benchmark and human confirmation before applying proposals.

### ENG-29 · Discipline workflow acceptance

These are final end-to-end acceptance tasks, not additional analysis solvers. Complete each
using the actual implemented tools, with engine/automation and UI evidence plus a saved report.

- [ ] **ENG-29.01 — Civil/site.** Review a synthetic utility/site plan, measure paving and routes,
  overlay a revision, and report drainage/phasing comments. Verify known quantities and change locations.
- [ ] **ENG-29.02 — Structural.** Redline members/connections/reinforcement, quantify concrete,
  coordinate an opening, and close review comments. Verify quantities and the revision audit trail.
- [ ] **ENG-29.03 — Mechanical/HVAC.** Take off duct/pipe/equipment, sketch a clearance, record an
  installation observation, and refresh an estimate. Verify fittings, rise/drop, and surface formulas.
- [ ] **ENG-29.04 — Electrical.** Count devices, measure cable/conduit, link panel references, and
  produce an inspection register. Verify grouped totals and referenced sheets.
- [ ] **ENG-29.05 — Plumbing/fire protection.** Count fixtures, measure routes, coordinate
  penetrations/riser references, and report deficiencies. Verify units and room assignments.
- [ ] **ENG-29.06 — Process/industrial.** Redline a synthetic P&ID, search equipment tags, review
  vendor sheets, and assemble a turnover package. Verify cross-references and issue closure.
- [ ] **ENG-29.07 — Geotechnical/environmental.** Annotate a report/investigation plan, reference
  sampling locations, and attach allowed original photographs. Verify evidence-location linkage.
- [ ] **ENG-29.08 — Manufacturing/product.** Review fabrication dimensions and a 3D PDF, exchange
  supplier comments, and export the review. Verify saved views, part references, and dimensions.
- [ ] **ENG-29.09 — Commissioning/QA.** Fill checklists/test forms, attach evidence, change issue
  statuses, and report outstanding items. Verify form calculations and history persistence.
- [ ] **ENG-29.10 — Engineering management.** Assign review metadata, close comments, issue a
  controlled drawing Set, and sign a delivery package. Verify permissions, versions, and signatures.

### ENG-30 · Release, documentation, and coverage

- [ ] **ENG-30.01 — Supported-platform release.** Package and smoke-test clean Windows, macOS,
  and Linux installations, including desktop takeoff and opt-in CLI/MCP. Record actual platform evidence.
- [ ] **ENG-30.02 — Original release assets.** Complete allowed SimplyPDF artwork/attribution and
  verify coexistence with PdfCraft installations. Verify no restricted upstream trademarks ship.
- [ ] **ENG-30.03 — Capability and limitation guide.** Document invocation, automation tools,
  dependencies, and tested coverage for every completed item. Verify links and runnable examples.
- [ ] **ENG-30.04 — Scope boundaries.** Document configured RFI/submittal/punch/estimate workflows
  separately from dedicated management systems, CAD authoring, engineering solvers, and BIM clash
  detection. Verify all product claims correspond to implemented, tested behavior.
- [ ] **ENG-30.05 — Inventory reconciliation.** Reconcile all 28 reference categories and ten
  discipline workflows against these IDs, existing checkpoints, and evidence. Keep incomplete
  integrations/preview work visible and do not translate competitor subscription tiers into our claims.

Reference documentation used for this backlog:
[Revu manual](https://support.bluebeam.com/user-manual/dashboard.html),
[measurement tools](https://support.bluebeam.com/user-manual/menus/window/measurements-panel.html),
[annotation register](https://support.bluebeam.com/user-manual/menus/window/markups-list-panel.html),
[quantity linking](https://support.bluebeam.com/revu/how-to/tips-and-tricks/improve-takeoffs-with-quantity-link.html),
[drawing Sets](https://support.bluebeam.com/user-manual/menus/window/create-edit-sets.html),
[batch operations](https://support.bluebeam.com/user-manual/menus/batch/batch-menu.html),
[web/mobile](https://support.bluebeam.com/bluebeam-cloud/resources/bluebeam-cloud-features.html),
[subscription scope](https://support.bluebeam.com/revu/subscription/subscription-features.html),
[Max tools](https://support.bluebeam.com/revu/resources/max.html), and
[preview qualifications](https://www.bluebeam.com/bluebeam-max/).

## Build and run

Install Rust 1.95 or later. From the checkout, open a drawing in the desktop app.

```sh
cargo run --locked --release -p simplypdf -- drawing.pdf
```

Build the CLI and start its MCP server with access confined to your drawing folder.

```sh
cargo build --locked --release -p simplypdf-cli
cargo run --locked --release -p simplypdf-cli -- mcp --root ./work
```

See the [MCP guide](docs/mcp.md) for client configuration.

## Export a takeoff workbook

In the desktop app, click **Export takeoff…** in the takeoff bar, choose **Excel workbook (.xlsx)**,
then click **Export** and choose a new workbook path. CSV is available in the same chooser.
The desktop and automation exports use the same workbook format.

Save the example below as `takeoff.json` and place a PDF with measurements at
`work/drawing.pdf`. The second step uses the document ID returned by the first open in a fresh
session.

```json
[
  {"tool": "doc_open", "args": {"path": "drawing.pdf"}},
  {"tool": "takeoff_export_xlsx", "args": {"doc": 1, "out": "takeoff.xlsx"}}
]
```

```sh
cargo run --locked --release -p simplypdf-cli -- run --script takeoff.json --root ./work
```

The export creates `work/takeoff.xlsx` and requires a fresh output path. In a separate
LibreOffice Calc estimate, use **Sheet → External Links** to import the `TakeoffData` named range.
Keep estimate formulas and pricing in that separate workbook. See the
[automation guide](crates/automation/README.md#takeoff-workbook) for export behavior.

## License

SimplyPDF is based on PdfCraft by the ArtCraft team and contributors. The code is available
under the [MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE). See [NOTICE](NOTICE)
and [asset attribution](ATTRIBUTION.md) for third-party terms.
