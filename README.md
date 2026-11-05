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
