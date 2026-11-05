# LibreOffice Draw Rust port

SimplyPDF will translate LibreOffice Draw source to Rust in reviewable slices. The reference is LibreOffice 26.8.0.3, commit `bce0998afefdbc355585ca324285661a2170ba77`. Existing SimplyPDF viewer, geometry, and takeoff code provide a runnable integration point during the port. [Pinned LibreOffice source](https://github.com/LibreOffice/core/tree/bce0998afefdbc355585ca324285661a2170ba77)

## Scope and order

The upstream `sd` tree contains 867 files under `source/`, 82 under `inc/`, 256 under `uiconfig/`, and 976 under `qa/` at this revision. Those counts include shared Draw/Impress files and fixtures. They measure inventory size, rather than the number of Draw functions to translate. Follow the Draw targets and shared dependencies before deciding ownership of each slice. [Draw UI target](https://github.com/LibreOffice/core/blob/bce0998afefdbc355585ca324285661a2170ba77/sd/UIConfig_sdraw.mk) · [Shared library manifest](https://github.com/LibreOffice/core/blob/bce0998afefdbc355585ca324285661a2170ba77/sd/Library_sd.mk)

- [x] Pin the upstream revision and inventory the shared Draw/Impress module.
- [x] Translate Draw's zoom history and test insertion, traversal, and capacity.
- [ ] Connect zoom history to the viewer's View commands and verify a page switch starts fresh history.
- [ ] Inventory Draw-owned and shared code using `sd` build targets, command slots, and test targets. Record ownership and dependencies by source unit.
- [ ] Port document and page models, drawing objects, layers, styles, undo, and selection. Map `svx`, `basegfx`, `docmodel`, and `drawinglayer` dependencies to explicit Rust interfaces.
- [ ] Port editing tools, text layout, drawing and hit testing, viewport behavior, and the Draw UI. Map `sfx2`, `framework`, `vcl`, and `editeng` behavior behind the appropriate Rust boundaries.
- [ ] Port Draw's import, export, and printing flows. Review UNO, ODF, PDF, and other supported formats separately, with round-trip tests for each.
- [ ] Port or replace the applicable Draw tests and verify behavior on Windows, macOS, and Linux. Track shared and Impress-only code explicitly when assessing coverage.
- [ ] Audit provenance, attribution, and redistribution requirements for every imported source, asset, fixture, and dependency before packaging.

For each translated slice, record the upstream revision and paths, Rust destinations, dependencies, any deliberate behavioral changes, and verification. Keep source notices with each translated file. The [upstream license page](https://www.libreoffice.org/licenses/) describes the mix of licenses. The initial Rust crate includes the MPL text, Apache license, and Apache OpenOffice attribution. Review each additional file's notices independently. Keep Bluebeam and other proprietary source, binaries, assets, and analysis dumps out of Git.

## First source mapping

| Upstream | Rust destination | Status |
| --- | --- | --- |
| [`sd/source/ui/view/zoomlist.cxx`](https://github.com/LibreOffice/core/blob/bce0998afefdbc355585ca324285661a2170ba77/sd/source/ui/view/zoomlist.cxx) | `crates/simplypdf-draw/src/zoom_history.rs` | Translated and tested |
| [`sd/source/ui/inc/zoomlist.hxx`](https://github.com/LibreOffice/core/blob/bce0998afefdbc355585ca324285661a2170ba77/sd/source/ui/inc/zoomlist.hxx) | Same module | Rust API adapted to generic view state |
| LibreOffice view-shell command invalidation | SimplyPDF viewer command availability | Integration pending |

The Rust translation preserves the ten-entry cap, insertion after the current position while below capacity, forward entries after stepping back, and oldest-entry eviction with cursor reuse at capacity. Rust returns `None` for empty history and lets the caller update command availability. It carries no LibreOffice UI or runtime dependency.

```sh
cargo test -p simplypdf-draw --locked --offline
```
