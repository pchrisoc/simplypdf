//! Native XLSX snapshots shared by the desktop and automation tools.

use crate::{Listing, Measurement, TAKEOFF_COLUMNS, TAKEOFF_SCHEMA_VERSION};
use rust_xlsxwriter::Workbook;
use std::path::Path;

// Excel supports 1,048,576 rows, including the header at zero-based row 0.
const MAX_DATA_ROWS: usize = 1_048_575;

fn data_row(offset: usize) -> Result<u32, String> {
    if offset >= MAX_DATA_ROWS {
        return Err("takeoff exceeds Excel's 1,048,575 measurement-row limit".into());
    }
    u32::try_from(offset + 1).map_err(|e| e.to_string())
}

/// Build an Excel Power Query (M) source for a published SimplyPDF takeoff workbook.
/// Excel users paste this into Data > Get Data > Blank Query > Advanced Editor.
/// Refresh All then reloads the named TakeoffData range without editing the estimate.
pub fn excel_power_query_source(workbook_path: &Path) -> Result<String, String> {
    if !workbook_path.is_absolute() {
        return Err("Excel Power Query source must be an absolute workbook path".into());
    }
    let path = workbook_path.to_str().ok_or("Excel Power Query source path is not UTF-8")?;
    if !path.to_ascii_lowercase().ends_with(".xlsx") {
        return Err("Excel Power Query source must be an .xlsx workbook".into());
    }
    // M text literals escape quotes by doubling them. Reject control characters.
    if path.chars().any(|c| c.is_control()) {
        return Err("Excel Power Query source path contains control characters".into());
    }
    // Power Query M interprets #(...) escape sequences inside text literals.
    // Escape # first so filenames such as #(lf).xlsx remain literal paths.
    let escaped = path.replace('#', "#(#)").replace('"', "\"\"");
    let columns = TAKEOFF_COLUMNS.iter().map(|name| format!(r#""{}""#, name.replace('"', "\"\""))).collect::<Vec<_>>().join(", ");
    Ok(format!(
        r#"let
    Source = Excel.Workbook(File.Contents("{escaped}"), null, true),
    Takeoff = Source{{[Item="TakeoffData", Kind="DefinedName"]}}[Data],
    Headers = Table.PromoteHeaders(Takeoff, [PromoteAllScalars=true]),
    ExpectedColumns = {{{columns}}},
    SchemaChecked = if Table.ColumnNames(Headers) <> ExpectedColumns then error "SimplyPDF takeoff schema columns have changed" else Headers,
    VersionChecked = if List.AnyTrue(List.Transform(Table.Column(SchemaChecked, "Schema version"), each if _ is number then Number.IsNaN(_) or Number.Abs(_) = #infinity or _ <> {TAKEOFF_SCHEMA_VERSION} else true)) then error "Unsupported SimplyPDF takeoff schema version" else SchemaChecked,
    Ids = List.Transform(Table.Column(VersionChecked, "Annotation ID"), each if _ is text then Text.Trim(_) else null),
    IdentityChecked = if List.Contains(Ids, null) or List.Contains(Ids, "") or List.Count(Ids) <> List.Count(List.Distinct(Ids)) then error "SimplyPDF takeoff has missing or duplicate annotation IDs" else VersionChecked,
    Positions = Table.SelectColumns(IdentityChecked, {{"Page", "Index"}}),
    PositionsChecked = if List.AnyTrue(List.Transform(Table.ToRows(Positions), each List.AnyTrue(List.Transform(_, (n) => if n is number then Number.IsNaN(n) or Number.Abs(n) = #infinity or n < 1 or n <> Number.RoundDown(n) else true)))) then error "SimplyPDF takeoff has invalid page or index positions" else IdentityChecked,
    Quantities = Table.Column(PositionsChecked, "Value"),
    Units = Table.Column(IdentityChecked, "Unit"),
    QuantityChecked = if List.AnyTrue(List.Transform(Quantities, each if _ is number then Number.IsNaN(_) or Number.Abs(_) = #infinity else true)) or List.AnyTrue(List.Transform(Units, each not (_ is text) or Text.Trim(_) = "")) then error "SimplyPDF takeoff has invalid quantities or missing units" else IdentityChecked,
    Scales = Table.Column(QuantityChecked, "Scale"),
    ScaleChecked = if List.AnyTrue(List.Transform(Scales, each not (_ is text) or Text.Trim(_) = "")) then error "SimplyPDF takeoff has missing scale metadata" else QuantityChecked,
    Types = Table.Column(ScaleChecked, "Type"),
    TypeChecked = if List.AnyTrue(List.Transform(Types, each not (_ is text) or Text.Trim(_) = "")) then error "SimplyPDF takeoff has missing measurement types" else QuantityChecked,
    Typed = Table.TransformColumnTypes(TypeChecked, {{
        {{"Page", Int64.Type}}, {{"Index", Int64.Type}}, {{"Type", type text}},
        {{"Value", type number}}, {{"Unit", type text}}, {{"Label", type text}},
        {{"Author", type text}}, {{"Scale", type text}},
        {{"Annotation ID", type text}}, {{"HVAC profile", type text}},
        {{"Material", type text}}, {{"Nominal size", type text}},
        {{"System", type text}}, {{"Schema version", Int64.Type}}
    }})
in
    Typed
"#
    ))
}

/// Export a ready-to-paste Power Query script without overwriting an existing file.
/// The source workbook need not exist yet, allowing estimate setup before publication.
pub fn write_excel_power_query_new(query_path: &Path, workbook_path: &Path) -> Result<(), String> {
    let script = excel_power_query_source(workbook_path)?;
    if !query_path.is_absolute() || !query_path.to_string_lossy().to_ascii_lowercase().ends_with(".pq") {
        return Err("Excel Power Query output must be an absolute .pq path".into());
    }
    if query_path == workbook_path {
        return Err("Excel Power Query output cannot replace its source workbook".into());
    }
    let parent = query_path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut staged = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    use std::io::Write;
    staged.write_all(script.as_bytes()).map_err(|e| e.to_string())?;
    staged.as_file().sync_all().map_err(|e| e.to_string())?;
    staged.persist_noclobber(query_path).map_err(|e| format!("{}: {}", query_path.display(), e.error))?;
    Ok(())
}

/// Publish a validated, versioned measurement snapshot without incomplete row identity.
pub fn write_new_checked(path: &Path, listing: &Listing) -> Result<(), String> {
    let fingerprint = crate::snapshot_fingerprint(listing).map_err(|e| e.to_string())?;
    write_snapshot(path, &listing.measurements, Some(&fingerprint))
}

/// Write a workbook beside its destination and publish it only if the path is free.
pub fn write_new(path: &Path, measurements: &[Measurement]) -> Result<(), String> {
    write_snapshot(path, measurements, None)
}

fn write_snapshot(path: &Path, measurements: &[Measurement], fingerprint: Option<&str>) -> Result<(), String> {
    // Refuse impossible worksheets before allocating or staging any output.
    if measurements.len() > MAX_DATA_ROWS {
        return Err("takeoff exceeds Excel's 1,048,575 measurement-row limit".into());
    }
    let failed = |e: rust_xlsxwriter::XlsxError| e.to_string();
    let mut book = Workbook::new();
    let sheet = book.add_worksheet();
    sheet.set_name("Takeoff").map_err(failed)?;
    sheet.write_row(0, 0, TAKEOFF_COLUMNS).map_err(failed)?;
    sheet.set_freeze_panes(1, 0).map_err(failed)?;
    for (offset, m) in measurements.iter().enumerate() {
        let row = data_row(offset)?;
        sheet.write_number(row, 0, m.page.saturating_add(1) as f64).map_err(failed)?;
        sheet.write_number(row, 1, m.index.saturating_add(1) as f64).map_err(failed)?;
        sheet.write_string(row, 2, m.reading.kind.name()).map_err(failed)?;
        sheet.write_number(row, 3, m.reading.value).map_err(failed)?;
        sheet.write_row(row, 4, [m.reading.unit.as_str(), &m.label, &m.author, &m.scale.ratio, &m.id]).map_err(failed)?;
        sheet.write_number(row, 13, f64::from(TAKEOFF_SCHEMA_VERSION)).map_err(failed)?;
        if let Some(hvac) = &m.hvac {
            sheet.write_row(row, 9, [&hvac.profile, &hvac.material, &hvac.size, &hvac.service]).map_err(failed)?;
        }
    }
    let count = measurements.len();
    book.define_name("TakeoffData", &format!("=Takeoff!$A$1:$N${}", count.saturating_add(1))).map_err(failed)?;
    book.define_name("TakeoffSchemaVersion", &TAKEOFF_SCHEMA_VERSION.to_string()).map_err(failed)?;
    if let Some(fingerprint) = fingerprint {
        // Store the validated publication token as a workbook-level defined name.
        // Consumers can compare it with snapshot_fingerprint before refreshing.
        book.define_name("TakeoffFingerprint", &format!("=\"{fingerprint}\"")).map_err(failed)?;
    }
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut staged = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    book.save_to_writer(staged.as_file_mut()).map_err(failed)?;
    staged.as_file().sync_all().map_err(|e| e.to_string())?;
    staged.persist_noclobber(path).map_err(|e| format!("{}: {}", path.display(), e.error))?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excel_query_uses_named_range_and_escapes_path_quotes() {
        let path = if cfg!(windows) { Path::new(r#"C:\takeoff\job "A".xlsx"#) } else { Path::new("/tmp/job \"A\".xlsx") };
        let m = excel_power_query_source(path).expect("absolute workbook path");
        assert!(m.contains("Item=\"TakeoffData\", Kind=\"DefinedName\""));
        assert!(m.contains("job \"\"A\"\".xlsx"));
        assert!(m.contains("Table.PromoteHeaders"));
        assert!(m.contains("Annotation ID"));
        assert!(m.contains("SchemaChecked"));
        assert!(m.contains("VersionChecked"));
        assert!(m.contains("List.Count(List.Distinct(Ids))"));
        assert!(m.contains("missing or duplicate annotation IDs"));
        assert!(m.contains("Text.Trim(_)"));
        assert!(m.contains("invalid quantities or missing units"));
        assert!(m.contains("Table.TransformColumnTypes(TypeChecked"));

        assert!(m.contains("Unsupported SimplyPDF takeoff schema version"));
    }

    #[test]
    fn excel_query_escapes_m_escape_sequences_in_workbook_paths() {
        let path = if cfg!(windows) { Path::new(r"C:\\takeoff\\job #(lf).xlsx") } else { Path::new("/tmp/job #(lf).xlsx") };
        let m = excel_power_query_source(path).expect("absolute workbook path");
        assert!(m.contains("job #(#)(lf).xlsx"));
        assert!(!m.contains("job #(lf).xlsx"));
    }

    #[test]
    fn excel_query_validates_measurement_types_and_format_braces() {
        let m = excel_power_query_source(Path::new("/tmp/takeoff.xlsx")).expect("valid query");
        assert!(m.contains("Positions = Table.SelectColumns(IdentityChecked, {\"Page\", \"Index\"})"));
        assert!(m.contains("TypeChecked = if List.AnyTrue"));
        assert!(m.contains("missing measurement types"));
        assert!(m.contains("Table.TransformColumnTypes(TypeChecked"));
    }

    #[test]
    fn excel_query_rejects_invalid_page_and_index_positions() {
        let m = excel_power_query_source(Path::new("/tmp/takeoff.xlsx")).expect("valid path");
        assert!(m.contains("Positions = Table.SelectColumns(IdentityChecked"));
        assert!(m.contains("Number.IsNaN(n) or Number.Abs(n) = #infinity or n < 1 or n <> Number.RoundDown(n) else true"));
        assert!(m.contains("Quantities = Table.Column(PositionsChecked"));
        assert!(m.contains("invalid page or index positions"));
    }

    #[test]
    fn excel_query_rejects_missing_schema_versions_and_non_finite_positions() {
        let m = excel_power_query_source(Path::new("/tmp/takeoff.xlsx")).expect("valid query");
        assert!(m.contains("each if _ is number then Number.IsNaN(_) or Number.Abs(_) = #infinity or _ <> 1 else true"));
        assert!(m.contains("Number.IsNaN(n) or Number.Abs(n) = #infinity"));
        assert!(m.contains("n < 1 or n <> Number.RoundDown(n)"));
    }

    #[test]
    fn excel_query_rejects_non_finite_schema_versions() {
        let m = excel_power_query_source(Path::new("/tmp/takeoff.xlsx")).expect("valid query");
        assert!(m.contains("Number.IsNaN(_) or Number.Abs(_) = #infinity or _ <> 1"));
        assert!(m.contains("Unsupported SimplyPDF takeoff schema version"));
    }

    #[test]
    fn excel_query_rejects_non_finite_quantities() {
        let m = excel_power_query_source(Path::new("/tmp/takeoff.xlsx")).expect("valid query");
        assert!(m.contains("Number.IsNaN(_) or Number.Abs(_) = #infinity"));
        assert!(m.contains("if _ is number then"));
        assert!(m.contains("invalid quantities or missing units"));
    }

    #[test]
    fn excel_query_rejects_relative_or_non_xlsx_paths() {
        assert!(excel_power_query_source(Path::new("relative.xlsx")).is_err());
        assert!(excel_power_query_source(Path::new("/tmp/source.csv")).is_err());
        assert!(excel_power_query_source(Path::new("/tmp/bad\nname.xlsx")).is_err());
    }

    #[test]
    fn power_query_file_is_written_once_and_preserves_existing_content() {
        let dir = tempfile::tempdir().expect("temporary query directory");
        let query = dir.path().join("takeoff.pq");
        let workbook = dir.path().join("takeoff.xlsx");
        write_excel_power_query_new(&query, &workbook).expect("create query");
        let original = std::fs::read_to_string(&query).expect("read query");
        assert!(original.contains("TakeoffData"));
        assert!(original.contains("Table.TransformColumnTypes"));
        assert!(write_excel_power_query_new(&query, &workbook).is_err());
        assert_eq!(std::fs::read_to_string(&query).expect("read original query"), original);
    }

    #[test]
    fn power_query_file_rejects_bad_source_without_creating_destination() {
        let dir = tempfile::tempdir().expect("temporary query directory");
        let query = dir.path().join("takeoff.pq");
        assert!(write_excel_power_query_new(&query, Path::new("relative.xlsx")).is_err());
        assert!(!query.exists());
        assert!(write_excel_power_query_new(&query, &dir.path().join("source.csv")).is_err());
        assert!(!query.exists());
    }

    #[test]
    fn power_query_file_requires_absolute_pq_destination() {
        let dir = tempfile::tempdir().expect("temporary query directory");
        let workbook = dir.path().join("source.xlsx");
        assert!(write_excel_power_query_new(Path::new("relative.pq"), &workbook).is_err());
        assert!(write_excel_power_query_new(&dir.path().join("wrong.xlsx"), &workbook).is_err());
        assert!(!dir.path().join("wrong.xlsx").exists());
        assert!(write_excel_power_query_new(&dir.path().join("valid.PQ"), &workbook).is_ok());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn power_query_export_escapes_m_literals_and_never_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let workbook = dir.path().join("takeoff#(lf)\"quoted.xlsx");
        let out = dir.path().join("estimate.pq");
        write_excel_power_query_new(&out, &workbook).unwrap();
        let script = std::fs::read_to_string(&out).unwrap();
        assert!(script.contains("takeoff#(#)(lf)\"\"quoted.xlsx"));
        assert!(script.contains("TakeoffData"));
        assert!(script.contains("Unsupported takeoff column schema"));
        assert!(write_excel_power_query_new(&out, &workbook).is_err());
        assert_eq!(std::fs::read_to_string(&out).unwrap(), script);
        assert!(write_excel_power_query_new(&dir.path().join("invalid.txt"), &workbook).is_err());
    }

    #[test]
    fn power_query_rejects_missing_scale_metadata_before_type_conversion() {
        let script = excel_power_query_source(&std::env::current_dir().unwrap().join("takeoff.xlsx")).unwrap();
        assert!(script.contains("Table.Column(QuantityChecked, \"Scale\")"));
        assert!(script.contains("SimplyPDF takeoff has missing scale metadata"));
        assert!(script.contains("Types = Table.Column(ScaleChecked, \"Type\")"));
    }

    #[test]
    fn excel_row_bounds_allow_last_measurement_and_reject_overflow() {
        assert_eq!(data_row(0).unwrap(), 1);
        assert_eq!(data_row(MAX_DATA_ROWS - 1).unwrap(), 1_048_575);
        assert!(data_row(MAX_DATA_ROWS).is_err());
        assert!(data_row(usize::MAX).is_err());
    }

    #[test]
    fn empty_snapshot_is_a_valid_xlsx_and_never_replaces_an_existing_file() {
        let dir = tempfile::tempdir().expect("temporary export directory");
        let path = dir.path().join("takeoff.xlsx");

        write_new(&path, &[]).expect("initial empty workbook export");
        let original = std::fs::read(&path).expect("read exported workbook");
        assert!(original.starts_with(b"PK"), "XLSX must be a ZIP container");
        assert!(original.len() > 100, "empty export must contain workbook metadata");

        let error = write_new(&path, &[]).expect_err("existing workbook must not be replaced");
        assert!(error.contains("takeoff.xlsx"), "{error}");
        assert_eq!(std::fs::read(&path).expect("read existing workbook"), original);
    }

    #[test]
    fn checked_snapshot_rejects_incomplete_rows_before_creating_file() {
        let dir = tempfile::tempdir().expect("temporary export directory");
        let path = dir.path().join("takeoff.xlsx");
        let mut listing = Listing { truncated: true, ..Listing::default() };
        assert!(write_new_checked(&path, &listing).is_err());
        assert!(!path.exists());
        listing.truncated = false;
        write_new_checked(&path, &listing).expect("complete empty snapshot");
        assert!(path.exists());
    }

    #[test]
    fn checked_workbook_carries_publication_metadata_distinct_from_legacy_export() {
        let dir = tempfile::tempdir().expect("temporary export directory");
        let checked = dir.path().join("checked.xlsx");
        let legacy = dir.path().join("legacy.xlsx");
        let listing = Listing::default();
        let fingerprint = crate::snapshot_fingerprint(&listing).expect("valid empty fingerprint");
        assert!(fingerprint.starts_with(&format!("v{TAKEOFF_SCHEMA_VERSION}-")));
        write_new_checked(&checked, &listing).expect("checked snapshot");
        write_new(&legacy, &listing.measurements).expect("legacy snapshot");
        let checked_bytes = std::fs::read(&checked).expect("checked workbook");
        let legacy_bytes = std::fs::read(&legacy).expect("legacy workbook");
        assert!(checked_bytes.starts_with(b"PK"));
        assert!(legacy_bytes.starts_with(b"PK"));
        assert_ne!(checked_bytes, legacy_bytes, "checked workbook must carry its publication fingerprint");
    }

    #[cfg(unix)]
    #[test]
    fn snapshot_refuses_symlink_destinations_without_changing_the_target() {
        use std::os::unix::fs::symlink;

        let dir = tempfile::tempdir().expect("temporary export directory");
        let target = dir.path().join("protected.xlsx");
        std::fs::write(&target, b"keep existing file").expect("write protected file");
        let link = dir.path().join("takeoff.xlsx");
        symlink(&target, &link).expect("create output symlink");

        write_new(&link, &[]).expect_err("symlink destination must not be overwritten");
        assert_eq!(std::fs::read(&target).expect("read target"), b"keep existing file");
        assert!(std::fs::symlink_metadata(&link).expect("symlink metadata").file_type().is_symlink());
    }
}
