//! Pinned tools, the tools open in this session, and mode-bar drop targets.

use crate::{LeftPanel, SimplyPDFApp, icons, theme, widgets};
use simplypdf_engine::{catalog, commands};

#[derive(Clone)]
pub struct DragTool(pub String);

pub fn valid(id: &str) -> bool {
    catalog::group(id).is_some() || id.strip_prefix("command:").is_some_and(|id| commands::command(id).is_some())
}

pub fn pin_quick(app: &mut SimplyPDFApp, id: &str) {
    if valid(id) && !app.quick_tools.iter().any(|item| item == id) && app.quick_tools.len() < 32 {
        app.quick_tools.push(id.into());
    }
}

fn activate(app: &mut SimplyPDFApp, id: &str) {
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

pub fn shortcuts(app: &mut SimplyPDFApp, ui: &mut egui::Ui) {
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

pub fn drop_target(app: &mut SimplyPDFApp, ui: &egui::Ui, response: &egui::Response) {
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

/// Independent outermost tool navigator. The settings pane to its right can close separately.
pub fn show(app: &mut SimplyPDFApp, ui: &mut egui::Ui) {
    if app.mode == crate::Mode::Read {
        return;
    }
    let width = if app.tool_rail_collapsed { 52.0 } else { 200.0 };
    let t = theme::Tokens::get(ui.ctx());
    egui::Panel::left("tool-navigator")
        .exact_size(width)
        .frame(egui::Frame::NONE.fill(t.chrome).inner_margin(6).stroke(egui::Stroke::new(1.0, t.divider)))
        .show(ui, |ui| {
            let icon = if app.tool_rail_collapsed { "chevrons-right" } else { "chevrons-left" };
            let tip = if app.tool_rail_collapsed { "Expand tool navigator" } else { "Collapse tool navigator" };
            if icons::button(ui, icon, 32.0, false, tl!(tip)).clicked() {
                app.tool_rail_collapsed = !app.tool_rail_collapsed;
            }
            if icons::button(ui, "panel-left", 32.0, app.left_open, tl!("Toggle tool settings")).clicked() {
                app.left_open = !app.left_open;
            }
            sidebar(app, ui);
        });
}

fn sidebar(app: &mut SimplyPDFApp, ui: &mut egui::Ui) {
    let current = match app.left {
        LeftPanel::Tool(id) => Some(id),
        _ => None,
    };
    if let Some(id) = current
        && !app.open_tools.iter().any(|tool| tool == id)
    {
        app.open_tools.push(id.into());
    }
    let pinned = app.pinned_tools.clone();
    let open: Vec<_> = app.open_tools.iter().filter(|id| !pinned.contains(id)).cloned().collect();
    let compact = app.tool_rail_collapsed;
    egui::ScrollArea::vertical().id_salt("tool-access-list").max_height((ui.available_height() - 45.0).max(40.0)).show(ui, |ui| {
        if !compact {
            widgets::section_title(ui, tl!("Pinned tools"));
        } else {
            icons::paint(
                ui,
                egui::Rect::from_min_size(ui.cursor().min, egui::vec2(ui.available_width(), 18.0)),
                "pin",
                12.0,
                theme::Tokens::get(ui.ctx()).text_muted,
            );
            ui.add_space(18.0);
        }
        for id in &pinned {
            row(app, ui, id, current == Some(id.as_str()), true);
        }
        ui.separator();
        if !compact {
            widgets::section_title(ui, tl!("Open tools"));
        }
        for id in &open {
            row(app, ui, id, current == Some(id.as_str()), false);
        }
    });
    if compact {
        if icons::button(ui, "layout-grid", 32.0, false, tl!("Browse all tools")).clicked() {
            app.expand_all_tools();
        }
    } else if widgets::ghost_button(ui, "layout-grid", tl!("Browse all tools")).clicked() {
        app.expand_all_tools();
    }
}

fn row(app: &mut SimplyPDFApp, ui: &mut egui::Ui, id: &str, active: bool, pinned: bool) {
    let Some(group) = catalog::group(id) else { return };
    let t = theme::Tokens::get(ui.ctx());
    let compact = app.tool_rail_collapsed;
    ui.push_id(("tool-access", id), |ui| {
        let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 34.0), egui::Sense::click_and_drag());
        response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, active, tl!(group.label)));
        theme::Press::track(ui, &response).wash(ui, rect, 6, active);
        if active {
            ui.painter().circle_filled(rect.left_center() + egui::vec2(2.5, 0.0), 2.0, t.accent_text);
        }
        let center = if compact { rect.center() + egui::vec2(2.0, 0.0) } else { rect.left_center() + egui::vec2(20.0, 0.0) };
        icons::paint(ui, egui::Rect::from_center_size(center, egui::vec2(18.0, 18.0)), group.icon, 18.0, t.icon);
        if !compact {
            widgets::paint_left(
                ui,
                rect.left_center() + egui::vec2(36.0, 0.0),
                tl!(group.label),
                theme::regular(13.0),
                t.text,
                (rect.width() - 40.0).max(1.0),
            );
        }
        source(&response, id.into());
        if response.clicked() {
            app.side_panels.tools_side = crate::side_panels::ToolsSide::Left;
            app.takeoff_workspace.settings_floating = false;
            app.select_tool(group.id);
        }
        response.on_hover_text(tl!(group.label)).context_menu(|ui| {
            if ui.button(tl!("Add to quick access")).clicked() {
                pin_quick(app, id);
                ui.close();
            }
            if ui.button(if pinned { tl!("Unpin tool") } else { tl!("Pin tool") }).clicked() {
                app.toggle_pin_tool(id);
                ui.close();
            }
            if !pinned && ui.button(tl!("Close tool")).clicked() {
                app.open_tools.retain(|tool| tool != id);
                if active {
                    app.left = LeftPanel::AllTools;
                    app.left_open = false;
                }
                ui.close();
            }
        });
    });
}
