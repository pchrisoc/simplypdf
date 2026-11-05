# LibreOffice Draw Rust port

SimplyPDF will translate LibreOffice Draw source to Rust in reviewable slices. The reference is LibreOffice 26.8.0.3, commit `bce0998afefdbc355585ca324285661a2170ba77`. The Draw crate remains independently testable. The combined SimplyPDF Rust app uses the translated Draw zoom and history code on its PDF canvas. Remaining Draw features follow the staged source port. [Pinned LibreOffice source](https://github.com/LibreOffice/core/tree/bce0998afefdbc355585ca324285661a2170ba77)

## Local upstream checkout

The complete pinned LibreOffice Core tree is checked out at `references/libreoffice/core`. It contains 149,176 tracked paths, including 2,269 under `sd/`. The checkout is shallow and includes every tracked file at the pinned revision. The parent repository ignores it. The dictionaries, help, and translations submodules are checked out at their pinned revisions.

```sh
mkdir -p references/libreoffice
git clone --depth 1 --single-branch --branch libreoffice-26.8.0.3 \
  https://github.com/LibreOffice/core.git references/libreoffice/core
git -C references/libreoffice/core submodule update --init --depth 1 --jobs 3
git -C references/libreoffice/core rev-parse HEAD
git -C references/libreoffice/core submodule status
git -C references/libreoffice/core status --short
```

Expected revision is `bce0998afefdbc355585ca324285661a2170ba77`. Use the complete source tree when following a Draw dependency out of `sd/`. The upstream source tree and other local reference material stay outside the parent Git history.

## Scope and order

The upstream `sd` tree contains 867 files under `source/`, 82 under `inc/`, 256 under `uiconfig/`, and 976 under `qa/` at this revision. Those counts include shared Draw/Impress files and fixtures. They measure inventory size, rather than the number of Draw functions to translate. Follow the Draw targets and shared dependencies before deciding ownership of each slice. [Draw UI target](https://github.com/LibreOffice/core/blob/bce0998afefdbc355585ca324285661a2170ba77/sd/UIConfig_sdraw.mk) · [Shared library manifest](https://github.com/LibreOffice/core/blob/bce0998afefdbc355585ca324285661a2170ba77/sd/Library_sd.mk)

- [x] Pull the complete pinned upstream tree and inventory the shared Draw/Impress module.
- [x] Translate Draw's zoom history and test insertion, traversal, and capacity.
- [x] Translate Draw's viewport fit and integral zoom calculations and compare them to the C++ methods.
- [x] Translate Draw's zoom tool motion and release decisions for drag outlines, pan, click zoom, and modifier zoom-out.
- [x] Connect Draw zoom geometry, click zoom, modifier zoom-out, drag outlines, and history to the SimplyPDF canvas.
- [ ] Port Draw's edge auto-scroll and remaining native zoom-tool behavior.
- [x] Preserve the complete classic menu bar and 48 toolbar definitions as pinned, licensed port references.
- [x] Connect zoom history to the viewer's View commands and verify a page switch starts fresh history.
- [ ] Map the classic menu and toolbar command IDs to upstream handlers, Rust handlers, and parity tests. Port standard File/Edit actions, view and navigation, drawing and formatting, then context-sensitive toolbars. Review Notebookbar, context menus, icons, and localization separately.
- [ ] Inventory Draw-owned and shared code using `sd` build targets, command slots, and test targets. Record ownership and dependencies by source unit.
- [ ] Port document and page models, drawing objects, layers, styles, undo, and selection. Map `svx`, `basegfx`, `docmodel`, and `drawinglayer` dependencies to explicit Rust interfaces.
- [ ] Port editing tools, text layout, drawing and hit testing, viewport behavior, and the Draw UI. Map `sfx2`, `framework`, `vcl`, and `editeng` behavior behind the appropriate Rust boundaries.
- [ ] Port Draw's import, export, and printing flows. Review UNO, ODF, PDF, and other supported formats separately, with round-trip tests for each.
- [ ] Port or replace the applicable Draw tests and verify behavior on Windows, macOS, and Linux. Track shared and Impress-only code explicitly when assessing coverage.
- [ ] Audit provenance, attribution, and redistribution requirements for every imported source, asset, fixture, and dependency before packaging.

For each translated slice, record the upstream revision and paths, Rust destinations, dependencies, any deliberate behavioral changes, and verification. Keep source notices with each translated file. The [upstream license page](https://www.libreoffice.org/licenses/) describes the mix of licenses. The initial Rust crate includes the MPL text, Apache license, and Apache OpenOffice attribution. Review each additional file's notices independently. Keep Bluebeam and other proprietary source, binaries, assets, and analysis dumps out of Git.

## Source areas to trace

| Area | Pinned source and build anchors | What the Rust port must account for |
| --- | --- | --- |
| Draw entry points | `sd/sdi/sdraw.sdi`, `sd/sdi/grdocsh.sdi`, `sd/sdi/grviewsh.sdi`, `sd/UIConfig_sdraw.mk` | Draw-specific command routing, menu configuration, toolbars, and document and view shell behavior |
| Shared document model | `sd/source/core/`, `sd/source/ui/docshell/`, `sd/Library_sd.mk` | Draw pages and shared model behavior across 32 core files and six document-shell files |
| View and tools | `sd/source/ui/view/`, `sd/source/ui/func/`, `sd/source/ui/inc/` | Zoom and gesture work starts here. Classify the rest of the 67 view, 59 tool, and 214 UI include files against Draw command reachability. |
| Formats and integration | `sd/source/filter/`, `sd/source/ui/unoidl/`, `sd/qa/unit/` | 49 filter paths and 26 UNO model paths to classify against the supported Draw formats and their tests |
| Shared and Impress paths | `sd/UIConfig_simpress.mk`, `sd/source/ui/slideshow/`, `sd/source/console/`, `sd/source/ui/slidesorter/` | Trace calls from Draw before marking these paths shared or Impress-specific. Directory names alone do not establish ownership. |

## Source mapping

| Upstream | Rust destination | Status |
| --- | --- | --- |
| [`sd/source/ui/view/zoomlist.cxx`](https://github.com/LibreOffice/core/blob/bce0998afefdbc355585ca324285661a2170ba77/sd/source/ui/view/zoomlist.cxx) | `crates/simplypdf-draw/src/zoom_history.rs` | Translated and tested |
| [`sd/source/ui/inc/zoomlist.hxx`](https://github.com/LibreOffice/core/blob/bce0998afefdbc355585ca324285661a2170ba77/sd/source/ui/inc/zoomlist.hxx) | Same module | Rust API adapted to generic view state |
| [`sd/source/ui/view/sdwindow.cxx`](https://github.com/LibreOffice/core/blob/bce0998afefdbc355585ca324285661a2170ba77/sd/source/ui/view/sdwindow.cxx) | `crates/simplypdf-draw/src/viewport.rs` | Translated `GetZoomForRect`, `SetZoomRect` arithmetic, and `SetZoomIntegral` arithmetic |
| [`sd/source/ui/func/fuzoom.cxx`](https://github.com/LibreOffice/core/blob/bce0998afefdbc355585ca324285661a2170ba77/sd/source/ui/func/fuzoom.cxx) | `crates/simplypdf-draw/src/zoom_tool.rs` | Translated the geometry decisions in `MouseMove` and `MouseButtonUp`. SimplyPDF canvas integration verified by input tests. |
| LibreOffice view-shell command invalidation | SimplyPDF viewer command availability | Integrated for Zoom Previous and Zoom Next |
| [`sd/uiconfig/sdraw/menubar` and `toolbar`](https://github.com/LibreOffice/core/tree/bce0998afefdbc355585ca324285661a2170ba77/sd/uiconfig/sdraw) | [Pinned XML references](../crates/simplypdf-draw/upstream/README.md) | Layout copied. Rust command parity pending. |

The Rust translation preserves the ten-entry cap, insertion after the current position while below capacity, forward entries after stepping back, and oldest-entry eviction with cursor reuse at capacity. Rust returns `None` for empty history and lets the caller update command availability. It carries no LibreOffice UI or runtime dependency.

The viewport port takes logical dimensions directly. Native windows must supply the pixel-to-logical conversion, apply the resulting map mode and repaint effects, and record history. The Rust APIs saturate values outside their representable coordinate ranges. A local comparison compiled the three pinned C++ methods against lightweight window and geometry stubs. The Rust results matched all 1,024 seeded inputs, including degenerate geometry, clamped zoom, and centering. This comparison covers the calculations and excludes VCL effects.

The existing SimplyPDF viewer exposes **View → Zoom → Zoom Previous / Zoom Next**. History starts at Fit Page for each loaded sheet and tracks discrete toolbar or menu zoom changes. Wheel zoom and pan remain interactive; their last view becomes a history entry when the next discrete zoom changes it. This is the viewer's integration policy for the translated history.

```sh
cargo test -p simplypdf-draw --locked --offline
```
