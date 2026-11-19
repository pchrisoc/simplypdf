# SimplyPDF takeoff row schema

The current native takeoff export contract is **version 1**, defined by
`TAKEOFF_SCHEMA_VERSION` and `TAKEOFF_COLUMNS` in
`crates/measure/src/lib.rs`. CSV and XLSX share these 14 columns in order:

| Column | Heading | Value |
| --- | --- | --- |
| A | Page | 1-based PDF page |
| B | Index | 1-based measurement index |
| C | Type | Measurement kind |
| D | Value | Numeric quantity |
| E | Unit | Quantity unit |
| F | Label | User label |
| G | Author | Annotation author |
| H | Scale | Applied scale ratio |
| I | Annotation ID | Stable annotation identifier |
| J | HVAC profile | Optional profile |
| K | Material | Optional material |
| L | Nominal size | Optional nominal size |
| M | System | Optional system/service |
| N | Schema version | Integer schema version (currently 1) |

## Identity and import

Measurement IDs come from the PDF annotation's identifier. The
`normalize_ids` operation repairs blank and duplicate imported IDs while
preserving the first occurrence, and the resulting identifiers survive PDF
save/reopen. Consumers must key by **Annotation ID**, not page/index, because
reordering and edits can change positions.

## Workbook contract

Native XLSX snapshot exports contain a `Takeoff` sheet, the named range
`TakeoffData` spanning columns A:N including the header, and the defined name
`TakeoffSchemaVersion`. The snapshot writer refuses to overwrite an existing
file; this is **not** the future managed live workbook sync.

The schema version is a compatibility boundary: adding, removing, renaming, or
reordering columns requires a new version and regression tests. CSV fields
with potentially dangerous spreadsheet-formula prefixes are escaped.
