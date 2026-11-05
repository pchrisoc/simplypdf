# Revu 21.11 navigation study

## Static evidence

REA identified the locally extracted Windows Revu managed assembly as version 21.11.0.23287 targeting .NET Framework 4.8. Its SHA-256 is `df4461f56d17eebce17cfcd7b1af0c29119770f4449d4f0a3d5db16681e43c68`. REA identity evidence is `ev_6580aa3c1631dfa2d95027448e7efd605ca72334702ec4ff9610d85d320a6bb4`.

A focused string search in that assembly found separate mouse-wheel settings for single-page and continuous layouts, plus a reverse-wheel setting. The names support the existence of separate preferences. REA's member analysis hit its representation limit on the main assembly, so handler flow and zoom formulas remain unresolved. Partial member evidence is `ev_8ca4bfab424ab0f583b9b1e6d0a1b32a55abed6bd0e1e27a331ebbd1bd021b31`. This pass used static inspection. Live Revu behavior remains untested.

## Documented behavior and Rust design

Bluebeam documents wheel zoom in single-page mode and scrolling in continuous mode by default. It also documents middle-button drag to pan and middle-button double-click to fit the page. [Wheel settings](https://support.bluebeam.com/revu/how-to/change-mouse-wheel-behavior.html) · [Revu navigation](https://support.bluebeam.com/user-manual/navigate.html)

SimplyPDF currently displays one sheet at a time. Its Rust viewer uses wheel or gesture zoom around the pointer, middle-button drag, a Pan toggle for primary-button dragging, and a Fit page button. Zoom limits of 50% to 800% of fit and the anchor calculation are SimplyPDF design choices. The selected point stays in canonical page coordinates as the canvas moves. Continuous mode and configurable wheel behavior are future comparison items.

Keep the installer, assemblies, extracted code, and analysis dumps outside Git. This note records evidence identifiers and behavior summaries.


## Cursor and temporary pan

Bluebeam documents V for Select, Shift+V for Pan, middle-button dragging, and holding Space to temporarily pan. SimplyPDF uses V for Inspect, Shift+V for Pan, and supports both temporary pan gestures. The viewer uses a system crosshair for point inspection, an open hand when Pan is ready, and a closed hand while the canvas is held. These are SimplyPDF cursor choices. Bluebeam cursor assets and exact runtime cursor transitions were outside this comparison. [Revu mouse navigation and shortcuts](https://support.bluebeam.com/revu/resources/keyboard-shortcuts.html)
