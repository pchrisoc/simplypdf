//! Pinned tools, the tools open in this session, and mode-bar drop targets.

use crate::{LeftPanel, PdfCraftApp, icons, theme, widgets};
use pdfcraft_engine::{catalog, commands};

#[derive(Clone)]
pub struct DragTool(pub String);

pub fn valid(id: &str) -> bool {
    catalog::group(id).is_some() || id.strip_prefix("command:").is_some_and(|id| commands::command(id).is_some())
}

pub fn pin_quick(app: &mut PdfCraftApp, id: &str) {
    if valid(id) && !app.quick_tools.iter().any(|item| item == id) && app.quick_tools.len() < 32 {
        app.quick_tools.push(id.into());
    }
}

fn activate(app: &mut PdfCraftApp, id: &str) {
    if let Some(command) = id.strip_prefix("command:").and_then(commands::command) {
        app.run_command(command.id);
    } else if let Some(group) = catalog::group(id) {
        app.select_tool(group.id);
    }
}

/// An icon row that can be dragged onto the top bar.
pub fn source(response: &egui::Response, id: String) {
    response.dnd_set_drag_payload(DragTool(id));
}

pub fn shortcuts(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    let items = app.quick_tools.clone();
    if items.is_empty() {
        return;
    }
    ui.add_space(8.0);
    ui.separator();
    egui::ScrollArea::horizontal().id_salt("quick-tools-scroll").max_width((ui.available_width() - 400.0).max(60.0)).auto_shrink([true, false]).show(
        ui,
        |ui| {
            ui.horizontal(|ui| {
                for id in &items {
                    let (icon, label) = if let Some(command) = id.strip_prefix("command:").and_then(commands::command) {
                        (command.icon, command.label)
                    } else if let Some(group) = catalog::group(id) {
                        (group.icon, group.label)
                    } else {
                        continue;
                    };
                    let response = icons::button(ui, icon, 30.0, false, tl!(label));
                    source(&response, id.clone());
                    if response.clicked() {
                        activate(app, id);
                    }
                    response.context_menu(|ui| {
                        if ui.button(tl!("Remove shortcut")).clicked() {
                            app.quick_tools.retain(|item| item != id);
                            ui.close();
                        }
                    });
                }
            });
        },
    );
}

pub fn drop_target(app: &mut PdfCraftApp, ui: &egui::Ui, response: &egui::Response) {
    if response.dnd_hover_payload::<DragTool>().is_some() {
        ui.painter().rect_stroke(
            response.rect.shrink(2.0),
            4.0,
            egui::Stroke::new(1.0, theme::Tokens::get(ui.ctx()).accent),
            egui::StrokeKind::Inside,
        );
    }
    if let Some(payload) = response.dnd_release_payload::<DragTool>() {
        pin_quick(app, &payload.0);
    }
}

pub fn sidebar(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    let current = match app.left {
        LeftPanel::Tool(id) if app.left_open => Some(id),
        _ => None,
    };
    if let Some(id) = current
        && !app.open_tools.iter().any(|tool| tool == id)
    {
        app.open_tools.push(id.into());
    }
    let pinned = app.pinned_tools.clone();
    let open: Vec<_> = app.open_tools.iter().filter(|id| !pinned.contains(id)).cloned().collect();
    egui::ScrollArea::vertical().id_salt("tool-access-list").max_height((ui.available_height() * 0.25).clamp(110.0, 220.0)).show(ui, |ui| {
        widgets::section_title(ui, tl!("Pinned tools"));
        if pinned.is_empty() {
            ui.label(egui::RichText::new(tl!("Pin tools from Browse all tools.")).small().color(theme::Tokens::get(ui.ctx()).text_muted));
        }
        for id in &pinned {
            row(app, ui, id, current == Some(id.as_str()), true);
        }
        ui.separator();
        widgets::section_title(ui, tl!("Open tools"));
        for id in &open {
            row(app, ui, id, current == Some(id.as_str()), false);
        }
        if open.is_empty() && current.is_none() {
            ui.label(egui::RichText::new(tl!("Choose a tool to get started.")).small());
        }
    });
    if widgets::ghost_button(ui, "layout-grid", tl!("Browse all tools")).clicked() {
        app.expand_all_tools();
    }
    ui.add_space(6.0);
    ui.separator();
}

fn row(app: &mut PdfCraftApp, ui: &mut egui::Ui, id: &str, active: bool, pinned: bool) {
    let Some(group) = catalog::group(id) else { return };
    let t = theme::Tokens::get(ui.ctx());
    ui.push_id(("tool-access", id), |ui| {
        ui.horizontal(|ui| {
            let (dot, _) = ui.allocate_exact_size(egui::vec2(7.0, 28.0), egui::Sense::hover());
            if active {
                ui.painter().circle_filled(dot.center(), 2.5, t.accent_text);
            }
            let width = (ui.available_width() - 28.0).max(30.0);
            let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 30.0), egui::Sense::click_and_drag());
            response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, active, tl!(group.label)));
            theme::Press::track(ui, &response).wash(ui, rect, 6, active);
            icons::paint(
                ui,
                egui::Rect::from_center_size(rect.left_center() + egui::vec2(10.0, 0.0), egui::vec2(16.0, 16.0)),
                group.icon,
                16.0,
                t.icon,
            );
            widgets::paint_left(
                ui,
                rect.left_center() + egui::vec2(26.0, 0.0),
                tl!(group.label),
                theme::regular(13.0),
                t.text,
                (width - 28.0).max(1.0),
            );
            source(&response, id.into());
            if response.clicked() {
                app.select_tool(group.id);
            }
            let (icon, tip) = if pinned { ("pin", tl!("Unpin tool")) } else { ("x", tl!("Close tool")) };
            if icons::button(ui, icon, 24.0, false, tip).clicked() {
                if pinned {
                    app.unpin_tool(id);
                } else {
                    app.open_tools.retain(|tool| tool != id);
                    if active {
                        app.left = LeftPanel::AllTools;
                    }
                }
            }
            response.context_menu(|ui| {
                if ui.button(tl!("Add to quick access")).clicked() {
                    pin_quick(app, id);
                    ui.close();
                }
                if ui.button(if pinned { tl!("Unpin tool") } else { tl!("Pin tool") }).clicked() {
                    app.toggle_pin_tool(id);
                    ui.close();
                }
            });
        });
    });
}
