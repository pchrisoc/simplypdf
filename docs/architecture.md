# Architecture and maintenance

SimplyPDF's first-party code is written in Rust. Keep business rules in `simplypdf-core` and application entry points in separate crates as they arrive. Keep GUI types, renderer handles, storage objects, and MCP protocol types at their respective boundaries.

## Coordinate contract

| Space | Origin and axes | Who uses it |
| --- | --- | --- |
| Raw PDF user space | PDF coordinates with positive Y up | PDF adapter at input and output |
| Canonical page space | Unrotated MediaBox lower-left, positive Y up, in points after applying UserUnit | Persisted endpoints and shared measurement logic |
| Canvas | Rotated visible page positioned inside the viewport, positive Y down | Pointer input, raster placement, and overlays |

`PageGeometry::new` takes resolved page metadata. The PDF adapter resolves inherited MediaBox, CropBox, and Rotate and supplies the default UserUnit of 1 when absent. The visible region is CropBox intersected with MediaBox. `pdf_to_page` removes the media origin and applies UserUnit. `viewport` applies the crop offset and clockwise rotation, then places the page at the supplied canvas origin and scale. The origin includes layout and pan. Use logical canvas coordinates for pointers and overlays. Convert between logical units and device pixels at the render boundary.

The core permits points outside the visible crop so overlays can be clipped at the view boundary. It rejects invalid boxes, rotations, scales, and nonfinite conversion results. PDF metadata semantics follow [ISO 32000-1, sections 7.7.3 and 8.3.2](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/PDF32000_2008.pdf).

## Replacing a backend

1. Locate the engine boundary. The only current backend-facing contract is the metadata and coordinate API in `crates/simplypdf-core/src/geometry.rs`. Add a renderer interface once the viewer spike establishes which operations it needs.
2. Map the replacement engine's page metadata to that contract. Record which offsets, rotations, and unit conversions the engine already applies. Verify with the four-page synthetic fixture before wiring pointer events.
3. Keep persisted points and future takeoff calculations in canonical space. Reuse the same transform for overlay placement and pointer input. A renderer swap should preserve existing measurement positions and values.
4. Run the headless tests, then add an adapter-level check that opens the actual fixture and verifies placement against its manifest. Record coverage by OS and engine version in the progress log.

Keep engine-specific code in its adapter crate. Document acquisition, licenses, data conversions, and failure behavior next to each adapter when it lands.

## Shared distance API

The desktop and MCP can call the same `DistanceScale` API in `crates/simplypdf-core/src/measurement.rs`. Calibrate it from two canonical page points and a known length in millimeters. Pass two or more canonical page points to `length_mm` to measure an open path. Round only at the presentation or export boundary.

The calling service will resolve scale scope and verification state before measuring. Keep those decisions in the shared service when it arrives so the desktop, export, and MCP all agree. Keep the distance API independent of storage and transport so it can be exercised with a small input and known result.
