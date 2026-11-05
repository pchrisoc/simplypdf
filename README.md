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

## SimplyPDF roadmap

The first release targets a local estimating workflow. Open a plan, calibrate it, take off
quantities, review the underlying markups, and see the estimate update in Calc. The desktop app,
CLI, MCP server, and workbook publisher will share Rust implementations. Calc is a separate
application. Check off each item after its acceptance test passes and push the checkpoint to
the SimplyPDF development branch on GitHub.

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
- [ ] Build a takeoff table with selection synced to the drawing. Add filter, sort, and grouped
  totals by sheet, type, subject, label, and unit. Keep incompatible units separate.
- [ ] Add reusable takeoff presets, named spaces, and multi-sheet quantity totals. Verify a
  fixture with two scales and two trades produces the expected totals in the app, MCP, and Calc.
- [ ] Add area cutouts and depth-based volume after the core quantities and sync pass.

### Checkpoint 3 · Ship the desktop workflow

- [ ] Finish the SimplyPDF release identity, including original icons, application IDs, executable
  names, and installers. Verify a SimplyPDF install can coexist with PDFCraft.
- [ ] Package SimplyPDF for Windows, macOS, and Linux. Test opening a PDF and starting the opt-in
  MCP server from a clean install on each platform.
- [ ] Run a release smoke test on each platform using the same synthetic drawing and linked
  Calc estimate. Publish the verified builds with the test results and known gaps.

## Build and run

Install Rust 1.95 or later. From the checkout, open a drawing in the desktop app.

```sh
cargo run --locked --release -p pdfcraft -- drawing.pdf
```

Build the CLI and start its MCP server with access confined to your drawing folder.

```sh
cargo build --locked --release -p pdfcraft-cli
cargo run --locked --release -p pdfcraft-cli -- mcp --root ./work
```

See the [MCP guide](docs/mcp.md) for client configuration.

## Export a takeoff workbook

In the desktop app, open **Measure → Export takeoff as XLSX** and choose a new workbook path.
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
cargo run --locked --release -p pdfcraft-cli -- run --script takeoff.json --root ./work
```

The export creates `work/takeoff.xlsx` and requires a fresh output path. In a separate
LibreOffice Calc estimate, use **Sheet → External Links** to import the `TakeoffData` named range.
Keep estimate formulas and pricing in that separate workbook. See the
[automation guide](crates/automation/README.md#takeoff-workbook) for export behavior.

## License

SimplyPDF is based on PdfCraft by the ArtCraft team and contributors. The code is available
under the [MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE). See [NOTICE](NOTICE)
and [asset attribution](ATTRIBUTION.md) for third-party terms.
