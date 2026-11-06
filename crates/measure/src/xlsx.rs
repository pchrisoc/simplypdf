//! Native XLSX snapshots shared by the desktop and automation tools.

use crate::{Measurement, TAKEOFF_COLUMNS, TAKEOFF_SCHEMA_VERSION};
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

/// Write a workbook beside its destination and publish it only if the path is free.
pub fn write_new(path: &Path, measurements: &[Measurement]) -> Result<(), String> {
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
