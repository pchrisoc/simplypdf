# SimplyPDF brand artwork

The project owner's two supplied logos (November 5, 2026) are the SimplyPDF
visual identity. Both are geometric teal (`#266e81`) marks on a transparent
background, faithfully reconstructed as SVG rectangles from the original
2048 × 2048 artwork without changing the visible shape.

- `simplypdf.svg` / `simplypdf-small.svg` — light-mode mark.
- `simplypdf-dark.svg` — dark-mode mark.
- `simplypdf-light-128.png` / `simplypdf-dark-128.png` — the theme-aware
  home-screen and top-chrome marks. The resolved UI theme selects the variant.
- `simplypdf-1024.png`, `simplypdf.ico`, `simplypdf.icns`, and
  `hicolor/` — operating-system packaging and desktop icons (light default).
- `simplypdf-dark-256.png` and `simplypdf-dark-1024.png` — dark-mode
  packaging alternatives. OS launchers normally own their own theme-independent
  icon selection; the in-app logo switches immediately with the theme.

`packaging/icons.sh` regenerates native and theme-variant assets from the
SVG masters (requires resvg; iconutil for macOS). Update `ATTRIBUTION.toml`
SHA-256 entries and run `cargo xtask assets --write` afterward.

The separate PDF document file icon (`simplypdf-document.svg`) is not a
SimplyPDF brand logo and is retained as a file-type icon.

All branded artwork is recorded in `ATTRIBUTION.toml` and distributed under
the repository's dual MIT / Apache-2.0 license by the SimplyPDF owner.
