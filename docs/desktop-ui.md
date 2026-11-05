# Desktop UI direction

Build the desktop app in Rust around a generous drawing canvas. Take cues from Thunderbird's clear panes and legible density, and Bluebeam's sheet navigation and visible measurement state. Use original SimplyPDF artwork and styling.

| Region | First viewer slice | Takeoff slice |
| --- | --- | --- |
| Menu bar | File and View in the macOS system menu bar, with an in-window menu above the toolbar on Windows | Add menus as their commands become usable |
| Document toolbar | Page navigation, zoom, Fit Page, and Pan | Document tabs as multi-document workflows arrive |
| Left pane | Collapsible sheet thumbnails and page labels | Sheet search and per-sheet scale status |
| Canvas | Fit page, zoom, pan, rotate, and pointer placement | Calibration line, snapping indicator, and editable endpoints |
| Right pane | Collapsible sheet details | Tool properties and guided calibration |
| Bottom area | Page, zoom, and keyboard hints | Active scale and scope, plus a collapsible measurement table |

Keep open, fit, and page navigation keyboard-accessible with visible focus and labeled controls. Reserve color for emphasis and pair it with text for states such as Scale required, Calibrated, Checked, and Failed. Provide light and dark chrome and keep the document canvas distinct from the surrounding controls. Make panels resizable so the drawing stays usable on a laptop.

The first UI review should show the same synthetic PDF on each target OS. Verify keyboard navigation, window resizing, high-DPI display, and alignment between raster content and the pointer marker at multiple zoom levels. During the takeoff pilot, use the release checklist to evaluate whether an estimator can calibrate, verify, measure, and export without coaching.

The current prototype groups file commands in File and keeps navigation controls in the top toolbar. That follows the Revu layout at the level of command placement. Revu also has a context-sensitive properties toolbar and markup panels, which belong to later slices. [Revu 21 interface guide](https://support.bluebeam.com/revu/subscription/navigate-and-customize-the-interface-21.html)
