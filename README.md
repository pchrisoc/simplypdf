# SimplyPDF

SimplyPDF is a Rust desktop PDF workbench for construction takeoff and estimating, based on
[PdfCraft](https://github.com/storytold/pdfcraft). The desktop targets Windows, macOS, and Linux.
Agents can use the same measurement tools through the CLI or an opt-in MCP server.

## What works today

- Calibrate a drawing from two points or a scale ratio, then measure distance, perimeter, and area.
- Save measurements as PDF annotations and export their quantities as numeric XLSX cells.
- Use MCP to open drawings, create and list measurements, and export a takeoff workbook.

The XLSX export is an on-demand snapshot. Live publishing to a Calc-linked workbook and
end-to-end validation on Windows and Linux are pending.

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
