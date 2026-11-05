# macOS viewer spike

This Rust prototype opens a local PDF, lists its pages, fits the selected page to the window, and reports canonical page coordinates when clicked. Use +/− or scroll over the drawing to zoom. Middle-drag or enable Pan and drag with the primary button. Use Fit page to reset the view. It defaults to SimplyPDF's synthetic four-page fixture. Use only trusted PDFs while parsing and rendering run inside the desktop process.

From the repository root on macOS:

```sh
cargo run --locked --manifest-path spikes/viewer/Cargo.toml
cargo run --locked --manifest-path spikes/viewer/Cargo.toml -- /path/to/drawing.pdf
cargo test --locked --manifest-path spikes/viewer/Cargo.toml
```

Choose **File → Open…** from the macOS system menu bar, press **⌘O**, or drop a PDF into the window. The viewer only replaces the current document once the replacement's first page renders. Page Up and Page Down change sheets. **⌘+** and **⌘−** zoom, and **⌘0** fits the page.

The document toolbar sits at the top of the window and groups sheet navigation, zoom, Fit Page, and Pan. File and View use native AppKit menus on macOS. The shared egui presentation puts those menus inside the window above the toolbar for Windows-style placement. The viewer currently launches on macOS. Windows integration and verification are pending.

The first build downloads dependencies from crates.io. This spike has a separate Cargo lockfile, so the headless workspace continues to build offline without GUI dependencies.

## Replacing the renderer

- `src/pdf.rs` owns Quartz document handles and rasterization. The viewer receives RGBA pixels, the pixel dimensions, the drawn portion of the texture, and shared `PageGeometry`.
- `src/menu.rs` defines the commands and their in-window presentation. `src/mac_menu.rs` presents the same commands through AppKit and queues them back to the viewer.
- `src/file_picker.rs` wraps AppKit file selection. The selected path goes through the same opening method as launch arguments and dropped files.
- `src/viewer.rs` owns the sheet list, canvas layout, and pointer feedback. It converts canvas clicks to canonical page points and uses those same points to place the marker when the window changes size.
- The adapter reads raw box coordinates and UserUnit once. It derives the Quartz drawing affine transform from the shared viewport and flips Y for the Quartz bitmap. The convenience drawing transform produced a different scale on the fixture, so this adapter constructs its own map.
- The raster test opens the actual PDF and samples ink at the independently specified edge midpoint on every rotation. It also maps those positions back to PDF coordinates. Run it after changing the engine.

The spike uses [eframe 0.31.1](https://github.com/emilk/egui/tree/0.31.1/crates/eframe) (MIT or Apache-2.0), Rust objc2 bindings (MIT), and the macOS [Core Graphics framework](https://developer.apple.com/documentation/coregraphics). This is a macOS reference implementation. Cross-platform renderer selection, release dependency auditing, and packaging remain open.

Rendering is synchronous and capped at 2048 pixels on the longest raster side. Zoom scales the existing raster, which can appear soft at high magnification. Next, add background rendering and accessibility coverage while verifying the cross-platform PDF adapter. Keep those checks separate from the takeoff workflow.

## PDFium comparison

The standalone Rust probe checks PDFium against the same four-page fixture. Obtain the trusted library for the host architecture from the pinned [PDFium chromium/8138 release](https://github.com/bblanchon/pdfium-binaries/releases/tag/chromium%2F8138) and verify its archive checksum before unpacking. The tested macOS arm64 archive `pdfium-mac-arm64.tgz` had SHA-256 `21a1567b397b571f4f77f21ca91eac7b892774219c3c715a5127a20950ccb18b`. Keep the library outside the checkout.

```sh
cargo run --locked --manifest-path spikes/viewer/Cargo.toml --example probe_pdfium -- /absolute/path/to/libpdfium.dylib
```

The probe expects a full path to the appropriate `.dylib`, `.dll`, or `.so`. It checks PDFium's rendered pixels and its raw-PDF-to-device mapping against the core transform and verifies that the chosen pixel maps back to the original PDF point. It reads metadata from the same PDF bytes that PDFium renders. The fixture manifest supplies independent expected dimensions and positions.

On macOS arm64, PDFium reported sizes of 200×150 and 150×200 for this fixture. The canonical sizes are 400×300 and 300×400 after applying UserUnit 2. Rendering into a viewport sized from the canonical geometry aligned all four pages. The tested public PDFium size functions alone are insufficient for canonical geometry.

The probe uses [lopdf 0.42.0](https://github.com/J-F-Liu/lopdf/tree/v0.42.0) (MIT) with default features disabled. The metadata reader lives in `examples/probe_pdfium/metadata.rs`. It resolves inherited MediaBox, CropBox, and Rotate entries and reads UserUnit directly from each page. The rest of the probe receives only core `PageGeometry` values. To swap parsers, preserve that return type, page order, inheritance and defaults, and rerun the metadata tests and raster probe.

This comparison covers the synthetic fixture on macOS arm64. Broader parser agreement, encrypted and malformed document handling across both engines, and Windows and Linux verification remain open. The desktop viewer still uses Quartz.
