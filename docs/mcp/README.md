# Read-only MCP contract draft

The future Rust stdio adapter opens one explicitly selected project at startup. Each call reads a consistent saved snapshot and returns its project and document revisions. The adapter accepts project-scoped IDs and has no file path parameters. Tool discovery uses the five definitions in [read-only-tools.json](read-only-tools.json). Input schemas use JSON Schema 2020-12. The adapter will reserve stdout for the protocol and use stderr for diagnostics. [MCP tools specification](https://modelcontextprotocol.io/specification/2025-11-25/server/tools)

## Results

Every successful result includes `project_revision` and `document_revision`. Use MCP `structuredContent` for the result and provide the same JSON in a text content block for clients that consume text. Define output schemas alongside the result serializers when the transport lands.

| Tool | Additional result fields |
| --- | --- |
| `list_sheets` | `sheets` entries with `sheet_id`, `label`, and `scale_status`, plus optional `next_cursor` |
| `get_calibration` | Sheet and optional region scope, scale status, and calibration when present. Calibration includes ID, revision, factor in millimeters per page point, source, display unit, references when applicable, and check result when present. |
| `preview_distance` | `sheet_id`, optional `region_id`, unrounded `length`, `unit`, `scale_status`, `calibration_id`, and `calibration_revision` |
| `list_measurements` | `measurements` entries with measurement ID, sheet, category, computed length and unit, scale status, and calibration ID and revision, plus optional `next_cursor` |
| `get_takeoff_totals` | `unit`, `draft`, subtotals by sheet and category, and measurement IDs and reasons for excluded or acknowledged entries |

A cursor is opaque and tied to the snapshot, tool, and filters. Callers restart pagination if the revision changes. Category filtering uses the saved category value. Omitting the region in `get_calibration` returns the page-level scope. Omitting it in `preview_distance` asks the resolver to select the applicable scope across every segment. A supplied region must cover the entire path. See the [implementation brief](../../SimplyPDF-implementation-plan.md) for region overlap and boundary rules.

Numeric units are `mm`, `m`, `in`, and `ft`. The core preview accepts canonical page points and converts the unrounded result from millimeters. Checked, unchecked, and failed scales carry their status in the result. Previewing a failed scale supports checking the discrepancy. Final totals exclude missing, failed, and stale scales and accept unchecked scales only with recorded acknowledgment. Draft totals identify all unverified contributions and exclusions.

## Errors and verification

Use a tool result with `isError: true` and a machine-readable `code` and human-readable `message` for validation or domain errors. Use JSON-RPC errors for malformed protocol requests or unknown tool names. Planned domain codes include `invalid_arguments`, `unknown_sheet`, `unknown_region`, `invalid_cursor`, `stale_cursor`, `invalid_geometry`, `length_out_of_range`, `missing_scale`, `ambiguous_scale`, `stale_scale`, `unsupported_project_schema`, and `project_unreadable`.

The headless preview tests cover unit conversion, calibration provenance, verification status, missing, ambiguous and stale scales, invalid geometry, and numeric underflow. They pass in a scale resolution outcome supplied by the caller. Integration tests for project loading, full-path scope resolution, pagination, tool discovery, errors, and desktop parity are still required.
