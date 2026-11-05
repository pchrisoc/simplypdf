//! Shared menu commands and an in-window presentation for Windows-style chrome.

use eframe::egui;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Open,
    Undo,
    Redo,
    Rectangle,
    FirstPage,
    PreviousPage,
    NextPage,
    LastPage,
    InspectTool,
    PanTool,
    ZoomTool,
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

impl MenuItem {
    const fn new(command: Command, label: &'static str, key: &'static str) -> Self {
        Self { command, label, key }
    }
}

pub const MENUS: &[(&str, &[MenuItem])] = &[
    ("File", &[MenuItem::new(Command::Open, "Open…", "o")]),
    ("Edit", &[
        MenuItem::new(Command::Undo, "Undo", "z"),
        MenuItem::new(Command::Redo, "Redo", "Z"),
    ]),
    ("View", &[
        MenuItem::new(Command::ZoomIn, "Zoom In", "="),
        MenuItem::new(Command::ZoomOut, "Zoom Out", "-"),
        MenuItem::new(Command::FitPage, "Fit Page", "0"),
        MenuItem::new(Command::ZoomPrevious, "Zoom Previous", ""),
        MenuItem::new(Command::ZoomNext, "Zoom Next", ""),
    ]),
    ("Insert", &[MenuItem::new(Command::Rectangle, "Rectangle", "")]),
    ("Page", &[
        MenuItem::new(Command::FirstPage, "First Page", ""),
        MenuItem::new(Command::PreviousPage, "Previous Page", ""),
        MenuItem::new(Command::NextPage, "Next Page", ""),
        MenuItem::new(Command::LastPage, "Last Page", ""),
    ]),
    ("Tools", &[
        MenuItem::new(Command::InspectTool, "Inspect", ""),
        MenuItem::new(Command::PanTool, "Pan", ""),
        MenuItem::new(Command::ZoomTool, "Zoom", ""),
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
                                Command::Undo | Command::Redo => Some(egui::Key::Z),
                                Command::ZoomIn => Some(egui::Key::Equals),
                                Command::ZoomOut => Some(egui::Key::Minus),
                                Command::FitPage => Some(egui::Key::Num0),
                                _ => None,
                            };
                            let mut button = egui::Button::new(item.label);
                            if let Some(key) = key {
                                let mut modifiers = egui::Modifiers::COMMAND;
                                modifiers.shift = item.command == Command::Redo;
                                let shortcut = egui::KeyboardShortcut::new(modifiers, key);
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
