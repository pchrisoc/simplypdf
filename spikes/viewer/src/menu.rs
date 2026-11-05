//! Shared menu commands and an in-window presentation for Windows-style chrome.

use eframe::egui;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Open,
    ZoomIn,
    ZoomOut,
    FitPage,
    ZoomPrevious,
    ZoomNext,
}

pub struct MenuItem {
    pub command: Command,
    pub label: &'static str,
    pub key: &'static str,
}

pub const MENUS: &[(&str, &[MenuItem])] = &[
    ("File", &[MenuItem { command: Command::Open, label: "Open…", key: "o" }]),
    ("View", &[
        MenuItem { command: Command::ZoomIn, label: "Zoom In", key: "=" },
        MenuItem { command: Command::ZoomOut, label: "Zoom Out", key: "-" },
        MenuItem { command: Command::FitPage, label: "Fit Page", key: "0" },
        MenuItem { command: Command::ZoomPrevious, label: "Zoom Previous", key: "" },
        MenuItem { command: Command::ZoomNext, label: "Zoom Next", key: "" },
    ]),
];

pub fn window_menu(
    context: &egui::Context,
    enabled: impl Fn(Command) -> bool,
) -> Option<Command> {
    let mut command = None;
    egui::TopBottomPanel::top("window_menu").show(context, |ui| {
        egui::menu::bar(ui, |ui| {
            for &(label, items) in MENUS {
                ui.menu_button(label, |ui| {
                    let mut entries = |ui: &mut egui::Ui| {
                        for item in items {
                            let key = match item.command {
                                Command::Open => Some(egui::Key::O),
                                Command::ZoomIn => Some(egui::Key::Equals),
                                Command::ZoomOut => Some(egui::Key::Minus),
                                Command::FitPage => Some(egui::Key::Num0),
                                Command::ZoomPrevious | Command::ZoomNext => None,
                            };
                            let mut button = egui::Button::new(item.label);
                            if let Some(key) = key {
                                let shortcut = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, key);
                                button = button.shortcut_text(context.format_shortcut(&shortcut));
                            }
                            if ui.add_enabled(enabled(item.command), button).clicked() {
                                command = Some(item.command);
                                ui.close_menu();
                            }
                        }
                    };
                    if label == "View" {
                        ui.menu_button("Zoom", entries);
                    } else {
                        entries(ui);
                    }
                });
            }
        });
    });
    command
}
