use std::fs::{self, OpenOptions};
use std::io;
use std::path::Path;

use simplypdf_draw::document::Document;

pub fn export(document: &Document, path: &Path) -> io::Result<()> {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let temporary = path.with_file_name(format!(".draw-rust-{}-{stamp}.tmp", std::process::id()));
    let mut file = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
    let result = (|| {
        crate::fodg::write(document, &mut file)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

#[cfg(target_os = "macos")]
pub fn choose_path() -> Result<Option<std::path::PathBuf>, &'static str> {
    use objc2::MainThreadMarker;
    use objc2_app_kit::{NSModalResponseOK, NSSavePanel};
    use objc2_foundation::NSString;

    let thread = MainThreadMarker::new().ok_or("The file chooser requires the main thread")?;
    let panel = NSSavePanel::savePanel(thread);
    panel.setNameFieldStringValue(&NSString::from_str("Drawing.fodg"));
    if panel.runModal() != NSModalResponseOK {
        return Ok(None);
    }
    let path = panel.URL().and_then(|url| url.to_file_path()).ok_or("Choose a local file")?;
    if !path.extension().is_some_and(|extension| extension.eq_ignore_ascii_case("fodg")) {
        return Err("Use the .fodg extension for a Flat OpenDocument Drawing");
    }
    Ok(Some(path))
}
