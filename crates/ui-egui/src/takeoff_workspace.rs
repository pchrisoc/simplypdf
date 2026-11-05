//! Dockable measurement settings and a searchable takeoff table.

use egui_extras::{Column, TableBuilder};
use pdfcraft_engine::{DocId, measure::{Listing, Measurement}};
use serde::{Deserialize, Serialize};

use crate::{LeftPanel, PdfCraftApp, QuickTool, side_panels::ToolsSide};

const HEADERS: [&str; 12] = ["Page", "Index", "Type", "Value", "Unit", "Label", "Author", "Profile", "Material", "Size", "System", "Scale"];

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Location {
    #[default]
    Bottom,
    Left,
    Right,
    Floating,
}

impl Location {
    fn label(self) -> &'static str {
        match self {
            Self::Bottom => "Bottom",
            Self::Left => "Left sidebar",
            Self::Right => "Right sidebar",
            Self::Floating => "Floating window",
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct State {
    pub settings_floating: bool,
    pub table_location: Location,
    pub expanded: bool,
    table_height: f32,
    #[serde(skip)]
    table_rect: Option<[f32; 4]>,
    #[serde(skip)]
    pub query: String,
    #[serde(skip)]
    sort: usize,
    #[serde(skip)]
    descending: bool,
    #[serde(skip)]
    cache: Option<Cache>,
}

impl Default for State {
    fn default() -> Self {
        Self { settings_floating: false, table_location: Location::Bottom, expanded: true, table_height: 230.0, table_rect: None, query: String::new(), sort: 0, descending: false, cache: None }
    }
}

struct Cache {
    doc: DocId,
    generation: u64,
    listing: Listing,
    cells: Vec<[String; 12]>,
    visible: Vec<usize>,
    query: Option<String>,
    sort: usize,
    descending: bool,
    totals: Vec<(String, f64)>,
}

fn cells(m: &Measurement) -> [String; 12] {
    let hvac = m.hvac.as_ref();
    [
        m.page.saturating_add(1).to_string(),
        m.index.saturating_add(1).to_string(),
        m.reading.kind.name().into(),
        format!("{:.*}", usize::from(m.scale.precision), m.reading.value),
        m.reading.unit.clone(),
        m.label.clone(),
        m.author.clone(),
        hvac.map_or_else(String::new, |h| h.profile.clone()),
        hvac.map_or_else(String::new, |h| h.material.clone()),
        hvac.map_or_else(String::new, |h| h.size.clone()),
        hvac.map_or_else(String::new, |h| h.service.clone()),
        m.scale.ratio.clone(),
    ]
}

impl State {
    fn refresh(&mut self, doc: &pdfcraft_engine::Document) -> Result<(), String> {
        if self.cache.as_ref().is_none_or(|cache| cache.doc != doc.id || cache.generation != doc.edit_generation()) {
            if self.cache.as_ref().is_some_and(|cache| cache.doc != doc.id) {
                self.query.clear();
            }
            let listing = match doc.measurements() {
                Ok(listing) => listing,
                Err(error) => {
                    self.cache = None;
                    return Err(error);
                }
            };
            let cells = listing.measurements.iter().map(cells).collect();
            self.cache = Some(Cache {
                doc: doc.id, generation: doc.edit_generation(), listing, cells, visible: Vec::new(), query: None, sort: 0, descending: false, totals: Vec::new(),
            });
        }
        Ok(())
    }

    fn filter_and_sort(&mut self) {
        let Some(cache) = &mut self.cache else { return };
        if cache.query.as_ref() == Some(&self.query) && cache.sort == self.sort && cache.descending == self.descending {
            return;
        }
        let query = self.query.trim().to_lowercase();
        cache.visible = cache.cells.iter().enumerate()
            .filter(|(_, cells)| query.is_empty() || cells.iter().any(|cell| cell.to_lowercase().contains(&query)))
            .map(|(index, _)| index).collect();
        cache.visible.sort_by(|a, b| {
            let left = cache.listing.measurements.get(*a);
            let right = cache.listing.measurements.get(*b);
            let ordering = match (left, right, self.sort) {
                (Some(a), Some(b), 0) => a.page.cmp(&b.page),
                (Some(a), Some(b), 1) => a.index.cmp(&b.index),
                (Some(a), Some(b), 3) => a.reading.value.total_cmp(&b.reading.value),
                _ => cache.cells.get(*a).and_then(|row| row.get(self.sort))
                    .cmp(&cache.cells.get(*b).and_then(|row| row.get(self.sort))),
            };
            let ordering = if self.descending { ordering.reverse() } else { ordering };
            ordering.then_with(|| a.cmp(b))
        });
        let mut totals = std::collections::BTreeMap::<String, f64>::new();
        for index in &cache.visible {
            if let Some(m) = cache.listing.measurements.get(*index) {
                *totals.entry(m.reading.unit.clone()).or_default() += m.reading.value;
            }
        }
        cache.totals = totals.into_iter().collect();
        cache.query = Some(self.query.clone());
        cache.sort = self.sort;
        cache.descending = self.descending;
    }
}

/// The same controls are available in the docked settings and floating window.
pub fn settings_header(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    let current = if app.takeoff_workspace.settings_floating { Location::Floating }
        else if app.side_panels.tools_side == ToolsSide::Left { Location::Left } else { Location::Right };
    let mut location = current;
    craft_ui::ComboBox::from_label(tl!("Settings placement"))
        .selected_text(tl!(current.label()))
        .width(110.0)
        .show_ui(ui, |ui| {
            for choice in [Location::Left, Location::Right, Location::Floating] {
                ui.selectable_value(&mut location, choice, tl!(choice.label()));
            }
        });
    if location != current {
        app.takeoff_workspace.settings_floating = location == Location::Floating;
        if location != Location::Floating {
            app.side_panels.tools_side = if location == Location::Left { ToolsSide::Left } else { ToolsSide::Right };
        }
    }
}

pub fn settings_window(app: &mut PdfCraftApp, ctx: &egui::Context) {
    if app.takeoff_workspace.settings_floating && app.left_open && app.active.is_some() && matches!(app.left, LeftPanel::Tool("measure")) {
        let mut open = true;
        egui::Window::new(tl!("Measurement settings")).id(egui::Id::unique("measurement-settings-window"))
            .open(&mut open).default_width(330.0).default_height(620.0).resizable(true).show(ctx, |ui| {
                crate::measure_ui::panel(app, ui, &crate::theme::Tokens::get(ctx));
            });
        if !open {
            app.takeoff_workspace.settings_floating = false;
        }
    }
}

pub fn show(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    let Some((_, id)) = app.active_ids() else {
        app.takeoff_workspace.cache = None;
        app.takeoff_workspace.query.clear();
        app.takeoff_workspace.table_rect = None;
        return;
    };
    let Some(doc) = app.session.get(id) else { return };
    if let Err(error) = app.takeoff_workspace.refresh(doc) {
        egui::Panel::bottom("takeoff-table-error").show(ui, |ui| { ui.label(error); });
        return;
    }
    if app.takeoff_workspace.table_location == Location::Floating {
        let response = egui::Window::new(tl!("Takeoff table")).id(egui::Id::unique("takeoff-table-window"))
            .default_width(900.0).default_height(260.0).resizable(true)
            .show(ui.ctx(), |ui| table(app, ui));
        app.takeoff_workspace.table_rect = response.map(|response| {
            let rect = response.response.rect;
            [rect.left(), rect.top(), rect.width(), rect.height()]
        });
        return;
    }
    let location = app.takeoff_workspace.table_location;
    let panel = match location {
        Location::Left => egui::Panel::left("takeoff-table-left").default_size(480.0),
        Location::Right => egui::Panel::right("takeoff-table-right").default_size(480.0),
        _ => egui::Panel::bottom("takeoff-table-bottom").default_size(app.takeoff_workspace.table_height.clamp(48.0, 900.0)),
    };
    let max = if app.takeoff_workspace.table_location == Location::Bottom { ui.available_height() * 0.7 } else { ui.available_width() * 0.7 };
    let response = panel.resizable(true).size_range(48.0..=max.max(48.0)).show(ui, |ui| {
        ui.set_min_height(ui.available_height());
        table(app, ui);
    });
    let rect = response.response.rect;
    app.takeoff_workspace.table_rect = Some([rect.left(), rect.top(), rect.width(), rect.height()]);
    if location == Location::Bottom {
        app.takeoff_workspace.table_height = rect.height();
    }
}

fn table(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    let active_view = app.active_ids().map(|(index, _)| index);
    let selected_markup = active_view.and_then(|index| app.views.get(index)).and_then(|view| view.comments.selected);
    let state = &mut app.takeoff_workspace;
    crate::widgets::begin_dialog(ui);
    ui.horizontal_wrapped(|ui| {
        ui.checkbox(&mut state.expanded, tl!("Takeoff table"));
        craft_ui::ComboBox::from_label(tl!("Table placement")).selected_text(tl!(state.table_location.label())).show_ui(ui, |ui| {
            for location in [Location::Bottom, Location::Left, Location::Right, Location::Floating] {
                ui.selectable_value(&mut state.table_location, location, tl!(location.label()));
            }
        });
        let search = ui.label(tl!("Search takeoffs"));
        ui.add(crate::widgets::line(&mut state.query).hint_text(tl!("Search takeoffs")).desired_width(180.0).char_limit(256)).labelled_by(search.id);
        if ui.button(tl!("Clear search")).clicked() {
            state.query.clear();
        }
    });
    state.filter_and_sort();
    let Some(cache) = &state.cache else { return };
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("{} / {} {}", cache.visible.len(), cache.listing.measurements.len(), tl!("measurements")));
        for (unit, value) in cache.totals.iter().take(8) {
            if value.is_finite() {
                ui.label(format!("{}: {value:.2} {unit}", tl!("Total")));
            } else {
                ui.label(tl!("Total exceeds numeric limits"));
            }
        }
    });
    if !cache.listing.unsupported.is_empty() || cache.listing.truncated {
        ui.colored_label(egui::Color32::YELLOW, format!("{}: {} · {}: {}", tl!("Unsupported"), cache.listing.unsupported.len(), tl!("Truncated"), cache.listing.truncated));
    }
    if !state.expanded {
        return;
    }
    let mut sort = None;
    let mut selected = None;
    egui::ScrollArea::horizontal().auto_shrink([false, false]).id_salt("takeoff-table-horizontal").show(ui, |ui| {
        TableBuilder::new(ui).id_salt("takeoff-rows").min_scrolled_height(24.0).auto_shrink([false, false]).striped(true).sense(egui::Sense::click())
            .columns(Column::initial(100.0).at_least(55.0).resizable(true).clip(true), HEADERS.len())
            .header(24.0, |mut header| {
                for (index, name) in HEADERS.iter().enumerate() {
                    header.col(|ui| {
                        let suffix = if state.sort == index { if state.descending { " ▼" } else { " ▲" } } else { "" };
                        if ui.button(format!("{}{suffix}", tl!(name))).clicked() {
                            sort = Some(index);
                        }
                    });
                }
            })
            .body(|body| {
                body.rows(24.0, cache.visible.len(), |mut row| {
                    let Some(index) = cache.visible.get(row.index()).copied() else { return };
                    let Some(cells) = cache.cells.get(index) else { return };
                    row.set_selected(cache.listing.measurements.get(index).is_some_and(|m| selected_markup == Some((m.page, m.index))));
                    for cell in cells {
                        row.col(|ui| {
                            if cell.is_empty() {
                                ui.label("");
                            } else if ui.selectable_label(false, cell).on_hover_text(cell).clicked() {
                                selected = Some(index);
                            }
                        });
                    }
                    if row.response().clicked() {
                        selected = Some(index);
                    }
                });
            });
    });
    if let Some(column) = sort {
        state.descending = state.sort == column && !state.descending;
        state.sort = column;
    }
    if let Some(index) = selected
        && let Some(m) = cache.listing.measurements.get(index)
        && let Some(view) = active_view
    {
        let view = &mut app.views[view];
        view.current = m.page;
        view.comments.selected = Some((m.page, m.index));
        view.measure.reading = Some(m.reading.clone());
        view.measure.scale = Some(m.scale.clone());
        app.quick_tool = QuickTool::Select;
    }
}

pub fn set_option(app: &mut PdfCraftApp, key: &str, value: &str) -> Result<(), String> {
    match key {
        "measurement-settings" => {
            let side = match value {
                "left" => Some(ToolsSide::Left),
                "right" => Some(ToolsSide::Right),
                "floating" => None,
                _ => return Err("measurement-settings must be left, right or floating".into()),
            };
            if let Some(side) = side {
                app.side_panels.tools_side = side;
            }
            app.takeoff_workspace.settings_floating = value == "floating";
        }
        "takeoff-table" => {
            app.takeoff_workspace.table_location = match value {
                "bottom" => Location::Bottom, "left" => Location::Left, "right" => Location::Right, "floating" => Location::Floating,
                _ => return Err("takeoff-table must be bottom, left, right or floating".into()),
            };
        }
        "takeoff-search" => {
            if value.len() > 1024 {
                return Err("takeoff-search is limited to 1024 bytes".into());
            }
            app.takeoff_workspace.query = value.into();
        }
        "takeoff-sort" => {
            let column = value.trim_start_matches('-');
            app.takeoff_workspace.sort = HEADERS.iter().position(|name| name.eq_ignore_ascii_case(column)).ok_or("unknown takeoff table column")?;
            app.takeoff_workspace.descending = value.starts_with('-');
        }
        _ => return Err("unknown takeoff workspace option".into()),
    }
    Ok(())
}

pub fn state(app: &PdfCraftApp) -> serde_json::Value {
    let workspace = &app.takeoff_workspace;
    serde_json::json!({
        "settings_floating": workspace.settings_floating,
        "table_location": workspace.table_location,
        "search": workspace.query,
        "sort": HEADERS.get(workspace.sort),
        "descending": workspace.descending,
        "table_rect": workspace.table_rect,
        "visible_count": workspace.cache.as_ref().map(|cache| cache.visible.len()),
        "totals": workspace.cache.as_ref().map(|cache| cache.totals.iter().take(200).collect::<Vec<_>>()),
        "visible_rows": workspace.cache.as_ref().map(|cache| cache.visible.iter().take(200).collect::<Vec<_>>()),
        "rows": workspace.cache.as_ref().map(|cache| cache.visible.iter().take(200).filter_map(|index| cache.cells.get(*index)).collect::<Vec<_>>()),
    })
}
