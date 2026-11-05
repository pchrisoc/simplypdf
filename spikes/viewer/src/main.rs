#[cfg(target_os = "macos")]
mod file_picker;
#[cfg(target_os = "macos")]
mod pdf;
#[cfg(target_os = "macos")]
mod viewer;

#[cfg(target_os = "macos")]
fn main() -> eframe::Result {
    let path = std::env::args_os().nth(1).map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/page-geometry.pdf"));
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1080.0, 760.0])
            .with_min_inner_size([640.0, 440.0]),
        ..Default::default()
    };
    eframe::run_native("SimplyPDF", options, Box::new(|context| {
        Ok(Box::new(viewer::Viewer::new(&context.egui_ctx, path)))
    }))
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("The current viewer spike requires macOS.");
    std::process::exit(1);
}
