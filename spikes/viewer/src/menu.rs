//! Shared menu commands and an in-window presentation for Windows-style chrome.

use eframe::egui;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Open,
    ZoomIn,
    ZoomOut,
    FitPage,
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
    ]),
];

impl Command {
    pub fn enabled(self, document_open: bool) -> bool {
        self == Self::Open || document_open
    }
}

pub fn window_menu(context: &egui::Context, document_open: bool) -> Option<Command> {
    let mut command = None;
    egui::TopBottomPanel::top("window_menu").show(context, |ui| {
        egui::menu::bar(ui, |ui| {
            for &(label, items) in MENUS {
                ui.menu_button(label, |ui| {
                    for item in items {
                        let shortcut = context.format_shortcut(&egui::KeyboardShortcut::new(
                            egui::Modifiers::COMMAND,
                            match item.command {
                                Command::Open => egui::Key::O,
                                Command::ZoomIn => egui::Key::Equals,
                                Command::ZoomOut => egui::Key::Minus,
                                Command::FitPage => egui::Key::Num0,
                            },
                        ));
                        let button = egui::Button::new(item.label).shortcut_text(shortcut);
                        if ui.add_enabled(item.command.enabled(document_open), button).clicked() {
                            command = Some(item.command);
                            ui.close_menu();
                        }
                    }
                });
            }
        });
    });
    command
}
