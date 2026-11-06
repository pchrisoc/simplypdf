# SimplyPDF — Task checklist

## First-release checkpoints

### Checkpoint 0 · Establish the baseline

- [x] Identify the local Revu 21.11.0 installer with REA.
- [x] Map the documented workflow.
- [x] Verify the existing Rust measurement and MCP baseline on macOS, including numeric XLSX snapshot export and protection against overwriting an existing workbook.
- [ ] Observe Revu on Windows with synthetic drawings.

### Checkpoint 1 · Publish live takeoff quantities to Calc

- [x] Share the [Rust workbook writer](crates/measure/src/xlsx.rs) between desktop export and CLI/MCP.
- [ ] Define persistent measurement IDs and a versioned row schema for saved and imported markups.
- [ ] Add an opt-in managed workbook link to the [Measure panel](crates/ui-egui/src/tools/measure_ui.rs) and matching MCP start, status, refresh, and stop tools.
- [ ] Publish after add, edit, delete, undo, and redo.
- [ ] Link the named range from a separate Calc estimate.

### Checkpoint 2 · Make takeoffs reviewable and repeatable

- [x] Refine measurement settings into icon rows, dropdowns and expandable advanced controls.
- [x] Open takeoffs after the first saved measurement and move CSV/XLSX export into a format chooser.
- [x] Add mode-bar drag-and-drop shortcuts and a pinned/open tools sidebar with active indicators.
- [x] Split the tool navigator and settings into independently collapsible columns.
- [x] Edit takeoff quantities and specifications directly in the table, with undo and save/reopen.
- [x] Show or hide takeoffs individually or by profile, material, and system.
- [x] Constrain HVAC presets to standard materials, systems, and sizes, with material colors and adjacent-size buttons.
- [x] Remove the top-right Discord promotion.
- [x] Add a resizable bottom takeoff table with search, column sorting, unit-separated totals, and selection linked to the drawing.
- [x] Support left/right docking and floating measurement settings and takeoff table windows.
- [x] Add continuous route takeoff with segment/total preview and a held Shift direction lock.
- [x] Add pipe and duct presets, shortcuts, nominal sizes, materials, and system tags saved to PDF measurements and exported to XLSX.
- [ ] Add fitting counts, rise/drop lengths, insulation, sheet-metal surface area, and fabrication allowances with explicit formulas and unit tests.
- [x] Apply separate horizontal and vertical calibration factors to distances and areas.
- [x] Add architectural, engineering, and metric construction presets with full-size and half-size calibration in the desktop app and MCP.
- [ ] Add editable page and viewport scales, with a visible indicator for uncalibrated drawings and the scale applied to each markup.
- [ ] Make vector snaps respect drawing layer visibility.
- [ ] Add named markup layers with visibility, assignment, and persistence.
- [ ] Add point counts with editable groups and visible totals.
- [ ] Extend the takeoff table with grouped totals by sheet, type, subject, and label, plus general markup rows and custom columns.
- [ ] Add reusable takeoff presets, named spaces, and multi-sheet quantity totals.
- [ ] Add area cutouts and depth-based volume after the core quantities and sync pass.

### Checkpoint 3 · Ship the desktop workflow

- [x] Rename app, CLI, crates, application IDs, executable names, and installer metadata to SimplyPDF.
- [ ] Finish original release artwork and verify installed SimplyPDF builds can coexist with PDFCraft on Windows, macOS, and Linux.
- [ ] Package SimplyPDF for Windows, macOS, and Linux.
- [ ] Run a release smoke test on each platform using the same synthetic drawing and linked Calc estimate.

## Engineering feature checklist

### ENG-01 · Drawing viewing and navigation

- [ ] **ENG-01.01 — Engineering document viewing.** Reuse the renderer and document session to open multipage drawings, specifications, calculations, reports, and manuals.
- [ ] **ENG-01.02 — Sheet navigation.** Connect thumbnails, labels, bookmarks, and hyperlinks to one navigation model.
- [ ] **ENG-01.03 — Split views.** Add horizontal/vertical splits for different sheets or two locations on one sheet.
- [ ] **ENG-01.04 — Synchronized views.** Link supported zoom/pan/page actions between splits with an explicit toggle.
- [ ] **ENG-01.05 — Multiple windows and monitors.** Detach documents and restore window layouts.
- [ ] **ENG-01.06 — Viewing modes.** Implement view-only rotation, full-screen/presentation modes, and line-weight/display controls.
- [ ] **ENG-01.07 — File access.** Add recent files and user-organized pinned locations.

### ENG-02 · Engineering redlines and annotations

- [ ] **ENG-02.01 — Text review tools.** Support text boxes, typewriter notes, leader callouts, and insert/delete/replace text comments.
- [ ] **ENG-02.02 — Clouds and pointers.** Implement revision clouds, cloud-plus-callout groups, arrows, lines, and dimension annotations.
- [ ] **ENG-02.03 — Shape tools.** Support rectangles, ellipses, polygons, polylines, and arcs.
- [ ] **ENG-02.04 — Freehand review.** Support pen strokes, erasing supported strokes, and text/area highlighting.
- [ ] **ENG-02.05 — Appearance standards.** Expose colors, opacity, fills, borders, line weights, arrowheads, custom line styles, and hatches.
- [ ] **ENG-02.06 — Geometry editing.** Move, resize, rotate, and edit vertices/curves/control points.
- [ ] **ENG-02.07 — Multi-annotation operations.** Align, distribute, group, ungroup, arrange, copy, duplicate, and lock annotations.
- [ ] **ENG-02.08 — Flattening.** Flatten selected annotations into page content with an explicit recovery option where supported.
- [ ] **ENG-02.09 — Image and stamp placement.** Place images and stamps as review annotations.

### ENG-03 · Reusable tools and engineering symbols

- [ ] **ENG-03.01 — Tool library.** Save named custom tools in discipline-specific sets; import, export, and share a versioned format.
- [ ] **ENG-03.02 — Reuse modes.** Separate appearance-only reuse from full saved geometry; preserve default subjects/comments and grouped assemblies.
- [ ] **ENG-03.03 — Scale-aware tools.** Resize calibrated tool geometry for page and viewport scales.
- [ ] **ENG-03.04 — Numbered symbols.** Add sequential labels with editable prefixes and counters.
- [ ] **ENG-03.05 — Snapshot symbols.** Capture selected PDF regions into reusable annotations while preserving supported vector content.
- [ ] **ENG-03.06 — Legends.** Generate and update legends from selected annotation classes and quantities; save legend presets.

### ENG-04 · Scaled sketches and proposed changes

- [ ] **ENG-04.01 — Sketch to scale.** Add dimension-driven rectangles, ellipses, polygons, and polylines using the measurement calibration model.
- [ ] **ENG-04.02 — Precision construction aids.** Share snapping, alignment, direction locks, and supported centroid display.
- [ ] **ENG-04.03 — Proposed layout sketches.** Combine scaled footprints, routes, openings, and clearance envelopes in editable groups.
- [ ] **ENG-04.04 — Offsets.** Offset supported lines and closed geometry by a specified distance.
- [ ] **ENG-04.05 — Annotation conversions.** Convert or duplicate compatible shapes, routes, and measurements.

### ENG-05 · Calibration and measurement accuracy

- [ ] **ENG-05.01 — Known-distance calibration.** Unify two-point calibration, preset/custom ratios, and saved presets.
- [ ] **ENG-05.02 — Page and viewport scales.** Edit scales per page and named viewport, apply them to page ranges, and show each markup's effective scale.
- [ ] **ENG-05.03 — Independent axes.** Preserve horizontal/vertical calibration and rotated-page transforms.
- [ ] **ENG-05.04 — Units and precision.** Configure metric/imperial units, decimal/fractional captions, and rounding.
- [ ] **ENG-05.05 — Vector snaps and constraints.** Snap to visible endpoints, midpoints, intersections, and paths; constrain directions.
- [ ] **ENG-05.06 — Measurement feedback.** Show placement previews, configurable captions, and uncalibrated indicators.
- [ ] **ENG-05.07 — Scale changes.** Define explicit recalculation for existing measurements and protected/read-only documents.

### ENG-06 · Measurement geometry and derived quantities

- [ ] **ENG-06.01 — Linear tools.** Complete length, continuous polylength, and perimeter with per-segment captions.
- [ ] **ENG-06.02 — Area geometry.** Measure rectangular, irregular, and curved regions.
- [ ] **ENG-06.03 — Circular and angular tools.** Add diameter, center radius, three-point radius, and angle measurements.
- [ ] **ENG-06.04 — Area cutouts.** Subtract polygonal/elliptical openings from measured regions.
- [ ] **ENG-06.05 — Volume and wall area.** Derive quantities from entered depth/height with explicit formulas.
- [ ] **ENG-06.06 — Slope adjustments.** Accept pitch, degrees, and grade with a clear reference plane.
- [ ] **ENG-06.07 — Rise/drop routes.** Add explicit vertical allowances to pipe/conduit routes.
- [ ] **ENG-06.08 — Related values and totals.** Show supported length/area/perimeter/wall/volume values per markup and selected/document totals.
- [ ] **ENG-06.09 — HVAC quantity extensions.** Add fitting counts, insulation, duct surface area, and fabrication allowances to existing pipe/duct presets.

### ENG-07 · Counts and assisted takeoff

- [ ] **ENG-07.01 — Count tools.** Add editable count groups, symbols, component types, and running totals.
- [ ] **ENG-07.02 — Quantity classification.** Classify counts/measurements by subject, label, layer, material, service, and custom fields.
- [ ] **ENG-07.03 — Symbol matching.** Implement bounded visual search with adjustable matching and reviewable results.
- [ ] **ENG-07.04 — Search-result actions.** Convert reviewed matches into counts or annotations as a reversible batch.
- [ ] **ENG-07.05 — Region fill.** Implement assisted bounded-region selection with temporary barriers and preview.
- [ ] **ENG-07.06 — Fill outputs.** Generate selected measurements or named Spaces from approved fills.

### ENG-08 · Calculations and workbook links

- [ ] **ENG-08.01 — Persistent row identity.** Add stable measurement IDs and a versioned export schema.
- [ ] **ENG-08.02 — Quantity formulas.** Add numeric/formula fields and unit-aware material/rate inputs.
- [ ] **ENG-08.03 — Classification lists.** Share editable choice lists for materials, cost codes, rates, and systems.
- [ ] **ENG-08.04 — Snapshot exports.**
- [ ] **ENG-08.05 — Managed workbook publishing.** Add opt-in start/status/refresh/stop controls and tools for a dedicated data workbook.
- [ ] **ENG-08.06 — Live refresh.** Publish after add/edit/delete/undo/redo; retry transient locks and report stale/incomplete scans.
- [ ] **ENG-08.07 — Filtered multi-PDF quantities.** Link totals by subject, label, author, layer, color, and custom fields across source PDFs.
- [ ] **ENG-08.08 — Calc estimate workflow.** Link a separate Calc estimate to `TakeoffData`.
- [ ] **ENG-08.09 — Excel workflow.** Provide and document a supported Excel live-link method equivalent in outcome to quantity-linked cells.

### ENG-09 · Review register and issue tracking

- [ ] **ENG-09.01 — General annotation register.** Extend the takeoff table to all supported annotations with author, page, subject, comments, timestamps, and properties.
- [ ] **ENG-09.02 — Register navigation.** Sort, group, search, and filter rows; save reusable filters.
- [ ] **ENG-09.03 — Custom fields.** Implement text, number, date, checkbox, choice, and formula columns.
- [ ] **ENG-09.04 — Comment threads.** Add replies with stable authorship/thread IDs.
- [ ] **ENG-09.05 — Status models.** Define custom review statuses and record status history.
- [ ] **ENG-09.06 — Review metadata.** Supply configurable discipline, assignee, priority, specification, tag, and package fields.
- [ ] **ENG-09.07 — Annotation exchange.** Import/merge supported PDF/FDF/XFDF annotations with conflict/duplicate handling.

### ENG-10 · Spaces and layers

- [ ] **ENG-10.01 — Named regions.** Create editable Spaces for rooms, zones, and floors, with nesting.
- [ ] **ENG-10.02 — Location association.** Associate annotations with Spaces and report by location.
- [ ] **ENG-10.03 — Room import.** Map supported Revit room metadata to Spaces through an explicitly configured adapter.
- [ ] **ENG-10.04 — Annotation layers.** Create named layers and assign/move annotations.
- [ ] **ENG-10.05 — Discipline isolation.** Show/hide relevant PDF and annotation layers without losing quantities.

### ENG-11 · Comparison and overlays

- [ ] **ENG-11.01 — Revision comparison.** Compare original/revised sheets and create navigable difference annotations.
- [ ] **ENG-11.02 — Comparison options.** Add sensitivity/rendering presets for vector and scanned sources plus annotation inclusion controls.
- [ ] **ENG-11.03 — Alignment.** Offer automatic alignment and editable registration/offsets.
- [ ] **ENG-11.04 — Color overlays.** Overlay two or more sheets using configurable colors and layer appearance.
- [ ] **ENG-11.05 — Batch revision review.** Match multiple sheets and batch compare/overlay them.

### ENG-12 · Sets and drawing revision control

- [ ] **ENG-12.01 — Virtual drawing Sets.** Navigate separate PDFs as one collection without merging sources.
- [ ] **ENG-12.02 — Tags and revision groups.** Categorize sheets, group revisions, and distinguish current/superseded sheets.
- [ ] **ENG-12.03 — Drawing logs.** Export sheet numbers, titles, disciplines, revisions, and source paths.
- [ ] **ENG-12.04 — Slip-sheet matching.** Preview page matches using labels/regions/user rules.
- [ ] **ENG-12.05 — Revision replacement.** Batch insert/replace sheets with configurable markup transfer and superseded-sheet retention.

### ENG-13 · PDF assembly and page management

- [ ] **ENG-13.01 — Assemble packages.** Combine PDFs and insert existing or blank pages.
- [ ] **ENG-13.02 — Page operations.** Extract, delete, reorder, and rotate pages; split by supported rules.
- [ ] **ENG-13.03 — Crop and resize.** Edit page boxes, resize pages, and deskew scans.
- [ ] **ENG-13.04 — Replace page content.** Retain annotations while replacing underlying sheets.
- [ ] **ENG-13.05 — Package identification.** Add/edit headers, footers, numbering, and automatic page labels.
- [ ] **ENG-13.06 — Attachments and layered pages.** Embed files and insert pages as layers.
- [ ] **ENG-13.07 — File-size reduction.** Offer fidelity-aware optimization presets and preview savings.

### ENG-14 · Editing, search, OCR, and conversion

- [ ] **ENG-14.01 — Content editing.** Edit text and erase/cut selected content through existing APIs.
- [ ] **ENG-14.02 — Color processing.** Recolor supported content with scoped previews.
- [ ] **ENG-14.03 — Permanent redaction.** Remove selected visible/hidden content and sanitize affected resources.
- [ ] **ENG-14.04 — OCR.** Make scanned engineering text searchable/selectable with language and region options.
- [ ] **ENG-14.05 — Document text search.** Search pages, open files, folders, and Sets with bounded results.
- [ ] **ENG-14.06 — Search and replace.** Preview text replacements and supported result actions.
- [ ] **ENG-14.07 — Office export.** Export supported Word/Excel/PowerPoint equivalents, including OCR-derived schedules.
- [ ] **ENG-14.08 — PDF creation.** Implement supported Office/file-to-PDF and Windows virtual printer workflows with safe adapters.
- [ ] **ENG-14.09 — Image output.** Export supported raster formats with resolution/page controls.

### ENG-15 · Navigable engineering packages

- [ ] **ENG-15.01 — Bookmarks and labels.** Manually edit and generate them from page content or regions.
- [ ] **ENG-15.02 — Linked contents.** Generate a table of contents from bookmarks with clickable destinations.
- [ ] **ENG-15.03 — Link authoring.** Link regions/annotations to pages, named views, files, and URLs.
- [ ] **ENG-15.04 — Batch links.** Detect sheet/detail references and preview generated hyperlinks.
- [ ] **ENG-15.05 — Link preservation.** Remap links during combine/split/replace and supported CAD publishing.

### ENG-16 · Shared review Sessions

- [ ] **ENG-16.01 — Session model.** Design an opt-in shared review service with authenticated invitations and document identity.
- [ ] **ENG-16.02 — Concurrent markup.** Synchronize simultaneous/asynchronous annotations with stable authorship and conflict handling.
- [ ] **ENG-16.03 — Shared discussions.** Synchronize replies, statuses, and markup alerts.
- [ ] **ENG-16.04 — Roles and permissions.** Enforce host/attendee save, print, markup, invite, and add-document rights on the service.
- [ ] **ENG-16.05 — Activity records.** Record review events and generate final reports.
- [ ] **ENG-16.06 — Offline review.** Queue local work, show sync state, and reconcile reconnects.

### ENG-17 · Shared Projects and file control

- [ ] **ENG-17.01 — Project storage.** Add opt-in storage for PDFs and other project files with folders/uploads.
- [ ] **ENG-17.02 —
- [ ] **ENG-17.03 — Access and sharing.** Enforce user/group permissions and explicit sharing.
- [ ] **ENG-17.04 — Offline files.** Sync selected files with visible freshness/conflict state.
- [ ] **ENG-17.05 — Project-to-review flow.** Start a Session from versioned Project PDFs and return reviewed results with traceable version links.
- [ ] **ENG-17.06 — Cross-platform access.** Expose supported Projects operations on desktop, browser, and mobile.

### ENG-18 · CAD/BIM publishing

- [ ] **ENG-18.01 — CAD publishing contract.** Define versioned adapters, supported applications, and retained metadata.
- [ ] **ENG-18.02 — 2D publishing.** Add supported AutoCAD/Revit/SolidWorks PDF creation paths.
- [ ] **ENG-18.03 — Batch CAD publishing.** Convert selected drawings/sheets with per-file results.
- [ ] **ENG-18.04 — AutoCAD metadata.** Retain supported sheet sets, hyperlinks, and SHX text workflows.
- [ ] **ENG-18.05 — Revit metadata.** Publish room/viewport transforms and sheet identity needed by Spaces and connected review.
- [ ] **ENG-18.06 — 3D publishing.** Add supported AutoCAD/Revit/Navisworks 3D PDF export adapters.

### ENG-19 · 3D PDF review

- [ ] **ENG-19.01 — Model viewing.** Audit/extend existing 3D support for rotate/spin/pan/zoom, cameras, and saved views.
- [ ] **ENG-19.02 — IFC/U3D creation.** Import supported model inputs into 3D PDFs with an explicit support matrix.
- [ ] **ENG-19.03 — Model tree.** Show hierarchy/properties with search and filters.
- [ ] **ENG-19.04 — Component display.** Isolate/show/hide parts and adjust supported appearance/ transparency.
- [ ] **ENG-19.05 — Section and display controls.** Implement supported cross-sections/render modes.
- [ ] **ENG-19.06 — 3D comments.** Anchor annotations to model views and navigate them from the register.
- [ ] **ENG-19.07 — Model data export.** Export filtered model-tree properties to CSV.

### ENG-20 · Field evidence and inspections

- [ ] **ENG-20.01 — Photo observations.** Place/crop/annotate photographs and associate them with drawing observations.
- [ ] **ENG-20.02 — Attached media.** Attach photos, supported 360-degree images, and video to annotations.
- [ ] **ENG-20.03 — Inspection registers.** Configure symbols, locations, attributes, and statuses for observations/deficiencies.
- [ ] **ENG-20.04 — Evidence summaries.** Export photo/comment reports with drawing references.

### ENG-21 · Web and mobile workflows

- [ ] **ENG-21.01 — Platform capability matrix.** Define desktop/browser/iOS/Android coverage separately for local documents, Sessions, and Projects.
- [ ] **ENG-21.02 — Remote document access.** View supported Project/Session/connected files in browser and mobile clients.
- [ ] **ENG-21.03 — Portable annotations.** Add supported annotations, replies, statuses, and reusable tools.
- [ ] **ENG-21.04 — Portable measurements.** Support length, polylength, area, and count using shared calibration.
- [ ] **ENG-21.05 — Mobile forms and offline use.** Fill/save forms, sync offline documents, and download/export files.
- [ ] **ENG-21.06 — Browser page creation.** Create PDF pages from blank layouts or original templates.
- [ ] **ENG-21.07 — Connected save-back.** Open and save through supported document services.

### ENG-22 · Forms, checklists, and stamps

- [ ] **ENG-22.01 — Form filling.** Fill supported text/check/radio/list/button/signature fields.
- [ ] **ENG-22.02 — Form authoring.** Create/edit fields and supported calculations.
- [ ] **ENG-22.03 — Field detection.** Preview automatically detected fields on static forms.
- [ ] **ENG-22.04 — Form data exchange.** Import/export/merge supported form data with validation.
- [ ] **ENG-22.05 — Review stamps.** Create reusable original stamps and dynamic/interactive fields.
- [ ] **ENG-22.06 — Batch stamps.** Preview/apply stamps to selected pages/files.

### ENG-23 · Signatures, seals, and security

- [ ] **ENG-23.01 — Digital signing.** Use digital IDs and signature fields to sign/certify PDFs.
- [ ] **ENG-23.02 — Signature inspection.** Show certificates, validity, trust, and post-signing modifications accurately.
- [ ] **ENG-23.03 — Security controls.** Apply passwords/permissions and honor them in all tools.
- [ ] **ENG-23.04 — Seal placement.** Separate professional seal graphics from digital signatures.
- [ ] **ENG-23.05 — Batch sign and seal.** Preview and atomically publish supported multi-file signing/sealing results.
- [ ] **ENG-23.06 — Sensitive-output workflow.** Combine redaction/security/signing in a documented order.

### ENG-24 · Reports and deliverables

- [ ] **ENG-24.01 — PDF comment summaries.** Generate readable reports with annotation data and drawing context.
- [ ] **ENG-24.02 — Structured exports.** Export annotations as CSV/XML with stable identifiers and typed values where supported.
- [ ] **ENG-24.03 — Filtered multi-file reports.** Filter by discipline/status/author/custom fields and Space across files.
- [ ] **ENG-24.04 — Quantity schedules.** Produce grouped takeoff summaries with unit-separated totals.
- [ ] **ENG-24.05 — Review delivery package.** Bundle drawing logs, Session history, outstanding comments, and supporting evidence.
- [ ] **ENG-24.06 — Model-data reports.** Include supported model-tree properties and view references in deliverables.

### ENG-25 · Batch processing and scripting

- [ ] **ENG-25.01 — Batch runner.** Share job selection, dry-run previews, progress, cancellation, per-file results, and safe publication across operations.
- [ ] **ENG-25.02 — Drawing batches.** Expose compare, overlay, link generation, and slip-sheet tasks through the batch runner.
- [ ] **ENG-25.03 — Document batches.** Add headers/footers, stamps, crop/resize, splitting, security, and signing/sealing jobs.
- [ ] **ENG-25.04 — Print batches.** Select files, page ranges, scales, paper, and destinations.
- [ ] **ENG-25.05 — Creation/assembly batches.** Combine/convert groups of supported files with ordered inputs.
- [ ] **ENG-25.06 — Script manager.** Save/replay versioned automation recipes over the shared tool table.

### ENG-26 · External document and field integrations

- [ ] **ENG-26.01 — Connector foundation.** Define opt-in authentication, capability discovery, secret storage, retries, and version-conflict APIs.
- [ ] **ENG-26.02 — SharePoint.** Open/version/save supported documents through an explicitly configured connector.
- [ ] **ENG-26.03 — ProjectWise.** Integrate supported file/review workflows via documented APIs.
- [ ] **ENG-26.04 — Office and Outlook.** Provide supported document/email-to-PDF workflows and attachment handling.
- [ ] **ENG-26.05 — OneDrive.** Add supported browse/open/save-back behavior.
- [ ] **ENG-26.06 — Box and Dropbox.** Implement each supported connector and document its actual read/write coverage.
- [ ] **ENG-26.07 — Procore and Egnyte.** Implement documented document access/save-back capabilities per service.
- [ ] **ENG-26.08 — DocuSign.** Add opt-in document handoff/status retrieval through a supported integration.
- [ ] **ENG-26.09 — GoCanvas-style field links.** Link configured field records/tasks to drawing annotations through documented APIs.
- [ ] **ENG-26.10 — Newforma and Autodesk Forma.** Implement documented partner workflows where access is available.
- [ ] **ENG-26.11 — Integration catalog.** Publish actual capabilities, supported versions, prerequisites, and known limits for each adapter, including Excel from ENG-08.09.

### ENG-27 · Team workspace standards

- [ ] **ENG-27.01 — Profiles and layouts.** Save/import/export UI profiles, panel arrangements, and shared templates.
- [ ] **ENG-27.02 — Toolbars and shortcuts.** Customize toolbars, quick-access actions, and keyboard bindings.
- [ ] **ENG-27.03 — Shared review standards.** Distribute tool sets, annotation defaults, statuses, and filters without overwriting personal work.
- [ ] **ENG-27.04 — Appearance and windows.** Audit dark mode, configurable panels, multiple windows, and application-instance behavior.
- [ ] **ENG-27.05 — Annotation translation.** Add explicitly configured translation with source retention and provider disclosure.

### ENG-28 · Assisted review and model-connected workflows

- [ ] **ENG-28.01 — Drawing health checks.** Detect missing/blank sheets and duplicate sheet IDs using configurable rules.
- [ ] **ENG-28.02 — Tag and grid coordination.**
- [ ] **ENG-28.03 — Assisted review results.** Present proposed issues as reviewable annotations, dashboards, and tracked items.
- [ ] **ENG-28.04 — Assisted set comparison.** Compare drawing sets across phases/scales with alignment and discipline reports.
- [ ] **ENG-28.05 — Sheet stitching.** Preview automatic placement and manual registration into a continuous view.
- [ ] **ENG-28.06 — AI through MCP.** Expose existing document/search/markup operations to supported configured clients.
- [ ] **ENG-28.07 — Revit-connected review.** Map review annotations into corresponding Revit sheets and model views using published metadata/adapters.
- [ ] **ENG-28.08 — Assisted-tool qualification.** Publish measured false positives/negatives, supported disciplines, model/provider requirements, preview status, and platform coverage.

### ENG-29 · Discipline workflow acceptance

- [ ] **ENG-29.01 — Civil/site.** Review a synthetic utility/site plan, measure paving and routes, overlay a revision, and report drainage/phasing comments.
- [ ] **ENG-29.02 — Structural.** Redline members/connections/reinforcement, quantify concrete, coordinate an opening, and close review comments.
- [ ] **ENG-29.03 — Mechanical/HVAC.** Take off duct/pipe/equipment, sketch a clearance, record an installation observation, and refresh an estimate.
- [ ] **ENG-29.04 — Electrical.** Count devices, measure cable/conduit, link panel references, and produce an inspection register.
- [ ] **ENG-29.05 — Plumbing/fire protection.** Count fixtures, measure routes, coordinate penetrations/riser references, and report deficiencies.
- [ ] **ENG-29.06 — Process/industrial.** Redline a synthetic P&ID, search equipment tags, review vendor sheets, and assemble a turnover package.
- [ ] **ENG-29.07 — Geotechnical/environmental.** Annotate a report/investigation plan, reference sampling locations, and attach allowed original photographs.
- [ ] **ENG-29.08 — Manufacturing/product.** Review fabrication dimensions and a 3D PDF, exchange supplier comments, and export the review.
- [ ] **ENG-29.09 — Commissioning/QA.** Fill checklists/test forms, attach evidence, change issue statuses, and report outstanding items.
- [ ] **ENG-29.10 — Engineering management.** Assign review metadata, close comments, issue a controlled drawing Set, and sign a delivery package.

### ENG-30 · Release, documentation, and coverage

- [ ] **ENG-30.01 — Supported-platform release.** Package and smoke-test clean Windows, macOS, and Linux installations, including desktop takeoff and opt-in CLI/MCP.
- [ ] **ENG-30.02 — Original release assets.** Complete allowed SimplyPDF artwork/attribution and verify coexistence with PdfCraft installations.
- [ ] **ENG-30.03 — Capability and limitation guide.** Document invocation, automation tools, dependencies, and tested coverage for every completed item.
- [ ] **ENG-30.04 — Scope boundaries.** Document configured RFI/submittal/punch/estimate workflows separately from dedicated management systems, CAD authoring, engineering solvers, and BIM clash detection.
- [ ] **ENG-30.05 — Inventory reconciliation.** Reconcile all 28 reference categories and ten discipline workflows against these IDs, existing checkpoints, and evidence.
