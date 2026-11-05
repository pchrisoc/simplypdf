# Synthetic PDF fixtures

The first-party generator is written in Rust. It writes a fixed PDF object graph with no vendor files, timestamps, fonts, or external dependencies.

```sh
cargo run --locked --offline -p simplypdf-core --example generate_geometry_fixture -- tests/fixtures/page-geometry.pdf
```

## Manifest

`page-geometry.pdf` contains four pages with clockwise rotation of 0°, 90°, 180°, and 270° in that order. Each page has MediaBox `[-10, 20, 310, 260]`, CropBox `[30, 50, 230, 200]`, and UserUnit 2. A right triangle connects raw PDF vertices (50, 80), (80, 80), and (80, 120).

The raw legs are 30 and 40 units, with hypotenuse 50 from the 3–4–5 ratio. UserUnit doubles these to 60, 80, and 100 canonical points. The first vertex is 60 units right and 60 units up from the media origin, so it becomes (120, 120) in canonical page space.

| Page | Rotation | Visible size at 1 canvas unit per point | First vertex from top left |
| --- | --- | --- | --- |
| 1 | 0° | 400 × 300 | (40, 240) |
| 2 | 90° | 300 × 400 | (60, 40) |
| 3 | 180° | 400 × 300 | (360, 60) |
| 4 | 270° | 300 × 400 | (240, 360) |

The Rust geometry tests use the values above as expectations. The renderer integration test should open the actual PDF and compare metadata, displayed triangle placement, and pointer coordinates with this manifest. Large vector, scanned, and mixed-scale examples belong to later slices.
