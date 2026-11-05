//! Dockable measurement settings and a searchable takeoff table.

use egui_extras::{Column, TableBuilder};
use serde::{Deserialize, Serialize};
use simplypdf_engine::{
    DocId,
    measure::{Listing, Measurement},
};

use crate::{LeftPanel, QuickTool, SimplyPDFApp, side_panels::ToolsSide};

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
    #[serde(skip)]
    pub export_doc: Option<DocId>,
    #[serde(skip)]
    export_csv: bool,

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
    #[serde(skip)]
    editor: Option<CellEdit>,
}

struct CellEdit {
    page: usize,
    index: usize,
    column: usize,
    text: String,
    focus: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            settings_floating: false,
            table_location: Location::Bottom,
            expanded: true,
            export_doc: None,
            export_csv: false,
            table_height: 230.0,
            table_rect: None,
            query: String::new(),
            sort: 0,
            descending: false,
            cache: None,
            editor: None,
        }
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
    fn refresh(&mut self, doc: &simplypdf_engine::Document) -> Result<(), String> {
        if self.cache.as_ref().is_none_or(|cache| cache.doc != doc.id || cache.generation != doc.edit_generation()) {
            if self.cache.as_ref().is_some_and(|cache| cache.doc != doc.id) {
                self.query.clear();
                self.editor = None;
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
                doc: doc.id,
                generation: doc.edit_generation(),
                listing,
                cells,
                visible: Vec::new(),
                query: None,
                sort: 0,
                descending: false,
                totals: Vec::new(),
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
        cache.visible = cache
            .cells
            .iter()
            .enumerate()
            .filter(|(_, cells)| query.is_empty() || cells.iter().any(|cell| cell.to_lowercase().contains(&query)))
            .map(|(index, _)| index)
            .collect();
        cache.visible.sort_by(|a, b| {
            let left = cache.listing.measurements.get(*a);
            let right = cache.listing.measurements.get(*b);
            let ordering = match (left, right, self.sort) {
                (Some(a), Some(b), 0) => a.page.cmp(&b.page),
                (Some(a), Some(b), 1) => a.index.cmp(&b.index),
                (Some(a), Some(b), 3) => a.reading.value.total_cmp(&b.reading.value),
                _ => cache.cells.get(*a).and_then(|row| row.get(self.sort)).cmp(&cache.cells.get(*b).and_then(|row| row.get(self.sort))),
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
pub fn settings_header(app: &mut SimplyPDFApp, ui: &mut egui::Ui) {
    let current = if app.takeoff_workspace.settings_floating {
        Location::Floating
    } else if app.side_panels.tools_side == ToolsSide::Left {
        Location::Left
    } else {
        Location::Right
    };
    ui.horizontal(|ui| {
        for (location, icon, label) in [
            (Location::Left, "panel-left", "Dock settings left"),
            (Location::Right, "panel-right", "Dock settings right"),
            (Location::Floating, "external-link", "Float settings"),
        ] {
            if crate::icons::button(ui, icon, 26.0, current == location, tl!(label)).clicked() {
                app.takeoff_workspace.settings_floating = location == Location::Floating;
                if location != Location::Floating {
                    app.side_panels.tools_side = if location == Location::Left { ToolsSide::Left } else { ToolsSide::Right };
                }
            }
        }
    });
    ui.add_space(6.0);
}

pub fn settings_window(app: &mut SimplyPDFApp, ctx: &egui::Context) {
    if app.takeoff_workspace.settings_floating && app.left_open && app.active.is_some() && matches!(app.left, LeftPanel::Tool("measure")) {
        let mut open = true;
        egui::Window::new(tl!("Measurement settings"))
            .id(egui::Id::unique("measurement-settings-window"))
            .open(&mut open)
            .default_width(330.0)
            .default_height(620.0)
            .resizable(true)
            .show(ctx, |ui| {
                crate::measure_ui::panel(app, ui, &crate::theme::Tokens::get(ctx));
            });
        if !open {
            app.takeoff_workspace.settings_floating = false;
        }
    }
}

pub fn show(app: &mut SimplyPDFApp, ui: &mut egui::Ui) {
    let Some((_, id)) = app.active_ids() else {
        app.takeoff_workspace.cache = None;
        app.takeoff_workspace.query.clear();
        app.takeoff_workspace.table_rect = None;
        return;
    };
    let Some(doc) = app.session.get(id) else { return };
    if let Err(error) = app.takeoff_workspace.refresh(doc) {
        egui::Panel::bottom("takeoff-table-error").show(ui, |ui| {
            ui.label(error);
        });
        return;
    }
    if app.takeoff_workspace.cache.as_ref().is_none_or(|cache| cache.listing.measurements.is_empty()) {
        app.takeoff_workspace.table_rect = None;
        return;
    }
    if app.takeoff_workspace.table_location == Location::Floating {
        let response = egui::Window::new(tl!("Takeoff table"))
            .id(egui::Id::unique("takeoff-table-window"))
            .default_width(900.0)
            .default_height(260.0)
            .resizable(true)
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

fn table(app: &mut SimplyPDFApp, ui: &mut egui::Ui) {
    let active_view = app.active_ids().map(|(index, _)| index);
    let selected_markup = active_view.and_then(|index| app.views.get(index)).and_then(|view| view.comments.selected);
    let export_doc = app.active_ids().map(|(_, id)| id);
    let may_export = export_doc.and_then(|id| app.session.get(id)).is_some_and(|doc| doc.allows_copying());
    let may_edit = export_doc.and_then(|id| app.session.get(id)).is_some_and(|doc| doc.allows_annotation());
    let mut edits = Vec::new();
    let state = &mut app.takeoff_workspace;
    crate::widgets::begin_dialog(ui);
    ui.horizontal_wrapped(|ui| {
        let icon = if state.expanded { "chevron-down" } else { "chevron-up" };
        if crate::icons::button(ui, icon, 26.0, false, tl!("Expand or collapse takeoffs")).clicked() {
            state.expanded = !state.expanded;
        }
        ui.label(egui::RichText::new(tl!("Takeoffs")).strong());
        let placement = crate::icons::button(ui, "settings", 26.0, false, tl!("Table placement"));
        egui::Popup::menu(&placement).show(|ui| {
            for location in [Location::Bottom, Location::Left, Location::Right, Location::Floating] {
                if ui.selectable_label(state.table_location == location, tl!(location.label())).clicked() {
                    state.table_location = location;
                    ui.close();
                }
            }
        });
        let visibility = crate::widgets::ghost_button(ui, "layers", tl!("Visibility"));
        egui::Popup::menu(&visibility).show(|ui| {
            ui.label(tl!("Show or hide takeoffs by category"));
            if let Some(cache) = &state.cache {
                for category in ["Profile", "Material", "System"] {
                    ui.separator();
                    ui.label(egui::RichText::new(tl!(category)).strong());
                    let mut groups = std::collections::BTreeMap::<&str, Vec<&Measurement>>::new();
                    for measurement in &cache.listing.measurements {
                        groups.entry(category_value(measurement, category)).or_default().push(measurement);
                    }
                    if groups.len() > 200 {
                        ui.label(tl!("Showing the first 200 categories; other takeoffs can be toggled in the table."));
                    }
                    for (name, members) in groups.into_iter().take(200) {
                        let mut visible = members.iter().any(|m| m.visible);
                        if ui.add_enabled(may_edit, crate::widgets::checkbox(&mut visible, name)).changed() {
                            for m in members {
                                edits.push(simplypdf_engine::Edit::UpdateMeasurement {
                                    page: m.page,
                                    index: m.index,
                                    patch: simplypdf_engine::measure::MeasurementPatch { visible: Some(visible), ..Default::default() },
                                });
                            }
                        }
                    }
                }
            }
        });
        let width = (ui.available_width() - 120.0).clamp(100.0, 360.0);
        let search = ui.add(crate::widgets::line(&mut state.query).hint_text(tl!("Search takeoffs")).desired_width(width).char_limit(256));
        search.widget_info(|| egui::WidgetInfo::labeled(egui::Role::TextInput, true, tl!("Search takeoffs")));
        if !state.query.is_empty() && crate::icons::button(ui, "x", 26.0, false, tl!("Clear search")).clicked() {
            state.query.clear();
        }
        let export = ui.add_enabled_ui(may_export, |ui| crate::widgets::ghost_button(ui, "file-down", tl!("Export takeoff…"))).inner;
        if export.clicked() {
            state.export_doc = export_doc;
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
        ui.colored_label(
            egui::Color32::YELLOW,
            format!("{}: {} · {}: {}", tl!("Unsupported"), cache.listing.unsupported.len(), tl!("Truncated"), cache.listing.truncated),
        );
    }
    if !state.expanded {
        apply_edits(app, export_doc, edits);
        return;
    }
    let mut sort = None;
    let mut selected = None;
    egui::ScrollArea::horizontal().auto_shrink([false, false]).id_salt("takeoff-table-horizontal").show(ui, |ui| {
        TableBuilder::new(ui)
            .id_salt("takeoff-rows")
            .min_scrolled_height(24.0)
            .auto_shrink([false, false])
            .striped(true)
            .sense(egui::Sense::click())
            .columns(Column::initial(100.0).at_least(55.0).resizable(true).clip(true), 9)
            .column(Column::initial(180.0).at_least(160.0).resizable(true).clip(true))
            .columns(Column::initial(100.0).at_least(55.0).resizable(true).clip(true), 2)
            .header(34.0, |mut header| {
                for (index, name) in HEADERS.iter().enumerate() {
                    header.col(|ui| {
                        let suffix = if state.sort == index { if state.descending { " ▼" } else { " ▲" } } else { "" };
                        if ui
                            .add(egui::Label::new(egui::RichText::new(format!("{}{suffix}", tl!(name))).strong()).sense(egui::Sense::click()))
                            .clicked()
                        {
                            sort = Some(index);
                        }
                    });
                }
            })
            .body(|body| {
                body.rows(34.0, cache.visible.len(), |mut row| {
                    let Some(index) = cache.visible.get(row.index()).copied() else { return };
                    let Some(cells) = cache.cells.get(index) else { return };
                    let Some(measurement) = cache.listing.measurements.get(index) else { return };
                    row.set_selected(cache.listing.measurements.get(index).is_some_and(|m| selected_markup == Some((m.page, m.index))));
                    for (column, cell) in cells.iter().enumerate() {
                        row.col(|ui| {
                            if column == 0 {
                                ui.horizontal(|ui| {
                                    let mut visible = measurement.visible;
                                    let response = ui.add_enabled(may_edit, crate::widgets::checkbox_without_text(&mut visible));
                                    response.widget_info(|| {
                                        egui::WidgetInfo::labeled(
                                            egui::Role::CheckBox,
                                            may_edit,
                                            format!("Show takeoff {}:{}", measurement.page + 1, measurement.index + 1),
                                        )
                                    });
                                    if response.changed() {
                                        edits.push(simplypdf_engine::Edit::UpdateMeasurement {
                                            page: measurement.page,
                                            index: measurement.index,
                                            patch: simplypdf_engine::measure::MeasurementPatch { visible: Some(visible), ..Default::default() },
                                        });
                                    }
                                    if ui.add(egui::Label::new(cell).sense(egui::Sense::click())).clicked() {
                                        selected = Some(index);
                                    }
                                });
                            } else if let Some(patch) = edit_cell(ui, measurement, column, cell, &mut state.editor, may_edit) {
                                edits.push(simplypdf_engine::Edit::UpdateMeasurement { page: measurement.page, index: measurement.index, patch });
                            } else if ui.ctx().input(|i| i.pointer.primary_clicked()) && ui.rect_contains_pointer(ui.min_rect()) {
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
        && let Some(view) = active_view.and_then(|index| app.views.get_mut(index))
    {
        view.current = m.page;
        view.comments.selected = Some((m.page, m.index));
        view.measure.reading = Some(m.reading.clone());
        view.measure.scale = Some(m.scale.clone());
        app.quick_tool = QuickTool::Select;
    }
    apply_edits(app, export_doc, edits);
}

fn apply_edits(app: &mut SimplyPDFApp, doc: Option<DocId>, edits: Vec<simplypdf_engine::Edit>) {
    if !edits.is_empty()
        && let Some(doc) = doc
        && let Err(error) = app.session.apply(doc, simplypdf_engine::Edit::Batch { label: "Edit takeoffs".into(), edits })
    {
        app.notify(error.to_string());
    }
}

fn category_value<'a>(m: &'a Measurement, category: &str) -> &'a str {
    match (m.hvac.as_ref(), category) {
        (Some(h), "Profile") => &h.profile,
        (Some(h), "Material") => &h.material,
        (Some(h), "System") => &h.service,
        _ => m.reading.kind.name(),
    }
}

fn edit_cell(
    ui: &mut egui::Ui,
    m: &Measurement,
    column: usize,
    text: &str,
    editor: &mut Option<CellEdit>,
    may_edit: bool,
) -> Option<simplypdf_engine::measure::MeasurementPatch> {
    use simplypdf_engine::measure::MeasurementPatch;
    if (7..=10).contains(&column)
        && let Some(hvac) = &m.hvac
    {
        let mut value = hvac.clone();
        ui.push_id((m.page, m.index, column), |ui| {
            ui.add_enabled_ui(may_edit, |ui| {
                if column == 9 {
                    if value.profile == "rectangular duct" {
                        ui.menu_button(value.size.clone(), |ui| crate::hvac_ui::fields(ui, &mut value));
                    } else {
                        let sizes = value.sizes();
                        let descending = value.profile == "pipe";
                        crate::hvac_ui::size(ui, "takeoff-size", &mut value.size, sizes, descending);
                    }
                } else {
                    let choices = match column {
                        7 => &["pipe", "round duct", "rectangular duct"][..],
                        8 => value.materials(),
                        _ => value.systems(),
                    };
                    let field = match column {
                        7 => &mut value.profile,
                        8 => &mut value.material,
                        _ => &mut value.service,
                    };
                    craft_ui::ComboBox::from_id_salt("takeoff-choice").selected_text(field.as_str()).width(ui.available_width()).show_ui(ui, |ui| {
                        for name in choices {
                            ui.selectable_value(field, (*name).into(), *name);
                        }
                    });
                    if value.profile != hvac.profile {
                        let profile = value.profile.clone();
                        value = if profile == "pipe" {
                            simplypdf_engine::measure::hvac::Takeoff::pipe()
                        } else {
                            simplypdf_engine::measure::hvac::Takeoff::duct()
                        };
                        value.profile = profile;
                        if value.profile == "rectangular duct" {
                            value.size = "4 x 4 in".into();
                        }
                    }
                }
            })
        });
        return (value != *hvac).then_some(MeasurementPatch { hvac: Some(value), ..Default::default() });
    }
    if !may_edit {
        *editor = None;
    }
    if let Some(edit) = editor.as_mut().filter(|edit| edit.page == m.page && edit.index == m.index && edit.column == column) {
        let response = ui.add(crate::widgets::line(&mut edit.text).desired_width(ui.available_width()).char_limit(512));
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::Role::TextInput,
                may_edit,
                format!("Edit {} {}:{}", HEADERS.get(column).unwrap_or(&"cell"), m.page + 1, m.index + 1),
            )
        });
        if edit.focus {
            response.request_focus();
            edit.focus = false;
            return None;
        }
        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            *editor = None;
            return None;
        }
        if response.lost_focus() || response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            let text = edit.text.clone();
            *editor = None;
            let mut patch = MeasurementPatch::default();
            match column {
                3 => match text.parse::<f64>() {
                    Ok(value) => patch.value = Some(value),
                    Err(_) => {
                        return Some(MeasurementPatch { value: Some(f64::NAN), ..Default::default() });
                    }
                },
                4 => patch.unit = Some(text),
                5 => patch.label = Some(text),
                6 => patch.author = Some(text),
                _ => return None,
            }
            return Some(patch);
        }
        return None;
    }
    let color = if !m.visible {
        crate::theme::Tokens::get(ui.ctx()).text_muted
    } else {
        m.hvac.as_ref().map_or(crate::theme::Tokens::get(ui.ctx()).text, crate::hvac_ui::color)
    };
    let response = ui.add(egui::Label::new(egui::RichText::new(text).color(color)).truncate().sense(egui::Sense::click()));
    if (3..=6).contains(&column) && may_edit {
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::Role::Button,
                true,
                format!("Edit {} {}:{}", HEADERS.get(column).unwrap_or(&"cell"), m.page + 1, m.index + 1),
            )
        });
        if response.clicked() {
            *editor = Some(CellEdit { page: m.page, index: m.index, column, text: text.into(), focus: true });
        }
        response.on_hover_text(tl!("Click to edit; Enter saves; Escape cancels"));
    } else {
        response.on_hover_text(text);
    }
    None
}

pub fn set_option(app: &mut SimplyPDFApp, key: &str, value: &str) -> Result<(), String> {
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
                "bottom" => Location::Bottom,
                "left" => Location::Left,
                "right" => Location::Right,
                "floating" => Location::Floating,
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

pub fn state(app: &SimplyPDFApp) -> serde_json::Value {
    let workspace = &app.takeoff_workspace;
    serde_json::json!({
        "settings_floating": workspace.settings_floating,
        "table_location": workspace.table_location,
        "search": workspace.query,
        "sort": HEADERS.get(workspace.sort),
        "descending": workspace.descending,
        "table_rect": workspace.table_rect,
        "export_open": workspace.export_doc.is_some(),
        "visible_count": workspace.cache.as_ref().map(|cache| cache.visible.len()),
        "totals": workspace.cache.as_ref().map(|cache| cache.totals.iter().take(200).collect::<Vec<_>>()),
        "visible_rows": workspace.cache.as_ref().map(|cache| cache.visible.iter().take(200).collect::<Vec<_>>()),
        "visibility": workspace.cache.as_ref().map(|cache| cache.listing.measurements.iter().take(200).map(|m| serde_json::json!({"page":m.page+1,"index":m.index+1,"visible":m.visible})).collect::<Vec<_>>()),
        "rows": workspace.cache.as_ref().map(|cache| cache.visible.iter().take(200).filter_map(|index| cache.cells.get(*index)).collect::<Vec<_>>()),
    })
}

/// One format chooser for the table export action.
pub fn export_dialog(app: &mut SimplyPDFApp, ctx: &egui::Context) {
    let Some(source) = app.takeoff_workspace.export_doc else { return };
    let mut confirm = false;
    let mut cancel = false;
    let modal = crate::widgets::show_modal(ctx, "takeoff-export", |ui| {
        ui.set_min_width(340.0);
        ui.heading(tl!("Export takeoff"));
        ui.add_space(12.0);
        ui.label(tl!("Choose a file format"));
        crate::widgets::radio_value(ui, &mut app.takeoff_workspace.export_csv, false, tl!("Excel workbook (.xlsx)"));
        crate::widgets::radio_value(ui, &mut app.takeoff_workspace.export_csv, true, tl!("Comma-separated values (.csv)"));
        ui.add_space(16.0);
        ui.horizontal(|ui| {
            cancel = ui.button(tl!("Cancel")).clicked();
            confirm = ui.button(tl!("Export")).clicked();
        });
    });
    if cancel || modal.should_close() || confirm {
        app.takeoff_workspace.export_doc = None;
    }
    if confirm {
        if app.active_ids().map(|(_, id)| id) != Some(source) {
            app.notify(tl!("Select the source drawing again before exporting."));
        } else if app.takeoff_workspace.export_csv {
            crate::measure_ui::export_csv(app);
        } else {
            #[cfg(not(target_arch = "wasm32"))]
            crate::measure_ui::export_workbook(app);
            #[cfg(target_arch = "wasm32")]
            app.notify(tl!("Use CSV export in the browser."));
        }
    }
}
