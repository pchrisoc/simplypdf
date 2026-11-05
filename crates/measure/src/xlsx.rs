//! Native XLSX snapshots shared by the desktop and automation tools.

use crate::Measurement;
use rust_xlsxwriter::Workbook;
use std::path::Path;

/// Write a workbook beside its destination and publish it only if the path is free.
pub fn write_new(path: &Path, measurements: &[Measurement]) -> Result<(), String> {
    let failed = |e: rust_xlsxwriter::XlsxError| e.to_string();
    let mut book = Workbook::new();
    let sheet = book.add_worksheet();
    sheet.set_name("Takeoff").map_err(failed)?;
    sheet
        .write_row(0, 0, ["Page", "Index", "Type", "Value", "Unit", "Label", "Author", "Scale", "Annotation ID"])
        .map_err(failed)?;
    sheet.set_freeze_panes(1, 0).map_err(failed)?;
    for (offset, m) in measurements.iter().enumerate() {
        let row = u32::try_from(offset.saturating_add(1)).map_err(|e| e.to_string())?;
        sheet.write_number(row, 0, m.page.saturating_add(1) as f64).map_err(failed)?;
        sheet.write_number(row, 1, m.index.saturating_add(1) as f64).map_err(failed)?;
        sheet.write_string(row, 2, m.reading.kind.name()).map_err(failed)?;
        sheet.write_number(row, 3, m.reading.value).map_err(failed)?;
        sheet
            .write_row(row, 4, [m.reading.unit.as_str(), &m.label, &m.author, &m.scale.ratio, &m.id])
            .map_err(failed)?;
    }
    let count = measurements.len();
    book.define_name("TakeoffData", &format!("=Takeoff!$A$1:$I${}", count.saturating_add(1))).map_err(failed)?;
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let mut staged = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    book.save_to_writer(staged.as_file_mut()).map_err(failed)?;
    staged.as_file().sync_all().map_err(|e| e.to_string())?;
    staged.persist_noclobber(path).map_err(|e| format!("{}: {}", path.display(), e.error))?;
    Ok(())
}
