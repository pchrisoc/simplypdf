//! Native macOS file selection, kept outside the viewer and PDF adapter.

use std::path::PathBuf;

use objc2::MainThreadMarker;
use objc2_app_kit::{NSModalResponseOK, NSOpenPanel};

pub fn open_document() -> Result<Option<PathBuf>, &'static str> {
    let main_thread = MainThreadMarker::new().ok_or("The file picker requires the main thread")?;
    let panel = NSOpenPanel::openPanel(main_thread);
    panel.setCanChooseFiles(true);
    panel.setCanChooseDirectories(false);
    panel.setAllowsMultipleSelection(false);
    if panel.runModal() != NSModalResponseOK {
        return Ok(None);
    }
    panel.URL()
        .and_then(|url| url.to_file_path())
        .map(Some)
        .ok_or("The selected file has no local path")
}
