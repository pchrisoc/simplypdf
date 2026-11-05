# Draw Rust

Draw Rust is the standalone application for the staged LibreOffice Draw source port. It depends on the Rust Draw crate and eframe. The SimplyPDF viewer is a separate executable.

## Launch on macOS

```sh
open "build/macos/Draw Rust.app"
```

To build the local bundle from the repository root:

```sh
sh apps/draw/packaging/build-macos.sh
```

For a development launch:

```sh
cargo run --locked --manifest-path apps/draw/Cargo.toml
```

The app opens a blank A4 drawing. Choose **Rectangle** or **Ellipse** and drag on the page. Choose a layer at the bottom before drawing. Use **Append Page** for a new page and **Edit** or **⌘Z** and **⇧⌘Z** for shape undo and redo. The toolbar zoom is relative to Fit Page, and **View** provides zoom history.

Use **File → Export Flat OpenDocument Drawing…** to write a `.fodg` file. The supported subset is standard pages, page margins, layer assignments, rectangles, and ellipses. Open the result in LibreOffice Draw. Draw Rust's own import, selection and editing of existing shapes, master pages, text, and the rest of Draw's UI are pending. The initial paper is fixed to A4. Locale and printer-dependent paper settings are pending.

The exporter emits the Flat OpenDocument drawing structure and shape attributes described by the specification. [OpenDocument 1.3 schema](https://docs.oasis-open.org/office/OpenDocument/v1.3/os/part3-schema/OpenDocument-v1.3-os-part3-schema.html)

See the [port ledger](../../docs/libreoffice-draw-port.md) for source mappings, license notices, and parity gates.
