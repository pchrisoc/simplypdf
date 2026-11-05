# macOS viewer spike

This Rust prototype opens a local PDF, lists its pages, fits the selected page to the window, and reports canonical page coordinates when clicked. It defaults to SimplyPDF's synthetic four-page fixture. Use only trusted PDFs while parsing and rendering run inside the desktop process.

From the repository root on macOS:

```sh
cargo run --locked --manifest-path spikes/viewer/Cargo.toml
cargo run --locked --manifest-path spikes/viewer/Cargo.toml -- /path/to/drawing.pdf
cargo test --locked --manifest-path spikes/viewer/Cargo.toml
```

The first build downloads dependencies from crates.io. This spike has a separate Cargo lockfile, so the headless workspace continues to build offline without GUI dependencies.

## Replacing the renderer

- `src/pdf.rs` owns Quartz document handles and rasterization. The viewer receives RGBA pixels, the pixel dimensions, the drawn portion of the texture, and shared `PageGeometry`.
- `src/viewer.rs` owns the sheet list, canvas layout, and pointer feedback. It converts canvas clicks to canonical page points and uses those same points to place the marker when the window changes size.
- The adapter reads raw box coordinates and UserUnit once. It derives the Quartz drawing affine transform from the shared viewport and flips Y for the Quartz bitmap. The convenience drawing transform produced a different scale on the fixture, so this adapter constructs its own map.
- The raster test opens the actual PDF and samples ink at the independently specified edge midpoint on every rotation. It also maps those positions back to PDF coordinates. Run it after changing the engine.

The spike uses [eframe 0.31.1](https://github.com/emilk/egui/tree/0.31.1/crates/eframe) (MIT or Apache-2.0), Rust objc2 bindings (MIT), and the macOS [Core Graphics framework](https://developer.apple.com/documentation/coregraphics). This is a macOS reference implementation. Cross-platform renderer selection, release dependency auditing, and packaging remain open.

Rendering is synchronous, fit-to-window, and capped at 2048 pixels on the longest raster side. Next, verify a cross-platform PDF adapter against the same fixture, then add file-open UI, zoom and pan, keyboard and accessibility support, and background rendering. Keep those checks separate from the takeoff workflow.
