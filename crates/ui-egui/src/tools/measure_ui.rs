//! Measure objects: page-space clicks, live readings and persistent drawing scales.
#[cfg(not(target_arch = "wasm32"))]
use crate::file_filter::AddFilterAnyCase as _;
use crate::{LeftPanel, PdfCraftApp, QuickTool, canvas::DocView, page_cursor::Layer, theme::Tokens, widgets};
use egui::{Color32, Pos2, Stroke};
use pdfcraft_engine::{
    Document, Edit, Style,
    measure::{self, Kind, NewMeasurement, PageUnit, Point, Reading, Scale},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Distance,
    Perimeter,
    Area,
    Calibrate,
}
impl Tool {
    pub fn name(self) -> &'static str {
        match self {
            Self::Distance => "distance",
            Self::Perimeter => "perimeter",
            Self::Area => "area",
            Self::Calibrate => "calibrate",
        }
    }
    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "distance" => Some(Self::Distance),
            "perimeter" => Some(Self::Perimeter),
            "area" => Some(Self::Area),
            "calibrate" => Some(Self::Calibrate),
            _ => None,
        }
    }
    fn kind(self) -> Kind {
        match self {
            Self::Area => Kind::Area,
            Self::Perimeter => Kind::Perimeter,
            _ => Kind::Distance,
        }
    }
}
pub struct MeasureView {
    pub points: Vec<Point>,
    locked_angle: Option<f64>,
    pub hvac: Option<measure::hvac::Takeoff>,
    pub page: Option<usize>,
    pub tool: Option<Tool>,
    pub reading: Option<Reading>,
    pub scale: Option<Scale>,
    pub error: Option<String>,
    pub snaps: measure::snap::SnapOptions,
    pub snap_enabled: bool,
    pub sensitivity: f64,
    /// The page side of the drawing scale, in its page unit ([`page_unit_of`]).
    pub drawing_points: f64,
    pub real_distance: f64,
    pub vertical_multiplier: f64,
    half_size_presets: bool,
    preset_family: usize,
    pub unit: String,
    /// The unit picked for the page side of the scale; `None` follows `unit`.
    pub page_unit: Option<PageUnit>,
    /// The "1:N" field of the drawing scale, as typed.
    pub ratio_text: String,
    /// The "1:N" field was edited and not yet applied.
    ratio_dirty: bool,
    pub precision: u8,
    pub name: String,
    pub whole_page: bool,
    pub rect: [f64; 4],
    pub label: String,
    seeded_page: Option<usize>,
    /// Snapping geometry (or why it failed) per (page, edit generation): extraction is not
    /// repeated on every hover frame.
    paths: Option<(usize, u64, Result<measure::snap::Geometry, String>)>,
}
impl Default for MeasureView {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            locked_angle: None,
            hvac: None,
            page: None,
            tool: None,
            reading: None,
            scale: None,
            error: None,
            snaps: Default::default(),
            snap_enabled: true,
            sensitivity: 6.0,
            drawing_points: 1.0,
            real_distance: 1.0,
            vertical_multiplier: 1.0,
            half_size_presets: false,
            preset_family: 0,
            unit: "in".into(),
            page_unit: None,
            ratio_text: String::new(),
            ratio_dirty: false,
            precision: 2,
            name: "Drawing scale".into(),
            whole_page: true,
            rect: [0.0, 0.0, 100.0, 100.0],
            label: String::new(),
            seeded_page: None,
            paths: None,
        }
    }
}
/// The unit of the page side of a drawing scale: the one picked, else the real-world unit when it
/// is pt, in, mm or cm (a plan at 1:50 in millimetres is 1 mm = 50 mm), else the display unit.
pub(crate) fn page_unit_of(picked: Option<PageUnit>, unit: &str, display: crate::units::Unit) -> PageUnit {
    picked.or_else(|| PageUnit::from_label(unit)).or_else(|| PageUnit::from_label(display.label())).unwrap_or(PageUnit::Pt)
}
/// Representative ratios for scale field regression tests.
#[cfg(test)]
pub(crate) const RATIOS: [f64; 10] = [1.0, 5.0, 10.0, 20.0, 25.0, 50.0, 100.0, 200.0, 500.0, 1000.0];
/// "1:200", "1/200", "1 : 200" or a bare "200" (a comma works as the decimal mark), as the N of 1:N.
/// `None` for anything else, for zero, negative or absurd values.
pub(crate) fn parse_ratio(text: &str) -> Option<f64> {
    let text = text.trim().replace(',', ".");
    let n = match text.split_once([':', '/']) {
        Some((one, n)) => n.trim().parse::<f64>().ok()? / one.trim().parse::<f64>().ok()?,
        None => text.parse::<f64>().ok()?,
    };
    RATIO_RANGE.contains(&n).then_some(n)
}
/// The N of 1:N this panel accepts. The lower bound keeps the real distance (N / 1000 m) within the
/// minimum of its field.
const RATIO_RANGE: std::ops::RangeInclusive<f64> = 1e-3..=1e9;
/// "1:200", or "1:2.5" for a ratio that isn't a whole number: the number as it is, so the field shows exactly what applies.
pub(crate) fn format_ratio(n: f64) -> String {
    format!("1:{n}")
}
/// The N of 1:N the scale fields stand for, when they are a plain drawing scale (real distance in m or mm).
fn ratio_of_fields(state: &MeasureView, display: crate::units::Unit) -> Option<f64> {
    let real_mm = match state.unit.trim() {
        "m" => state.real_distance * 1000.0,
        "mm" => state.real_distance,
        _ => return None,
    };
    let paper_mm = state.drawing_points * page_unit_of(state.page_unit, &state.unit, display).points() * 25.4 / 72.0;
    let n = real_mm / paper_mm;
    // Float noise from the unit conversions is rounded away; anything that isn't a clean ratio isn't one.
    let clean = (n * 1e6).round() / 1e6;
    (RATIO_RANGE.contains(&clean) && (clean - n).abs() <= 1e-9 * n).then_some(clean)
}
/// Make the "1:N" field and the common scale picked in its list say what the scale fields now hold: the ratio, or nothing.
fn sync_ratio_text(state: &mut MeasureView, display: crate::units::Unit) {
    state.ratio_text = ratio_of_fields(state, display).map(format_ratio).unwrap_or_default();
    state.ratio_dirty = false;
}
/// What the scale fields hold, to notice a hand edit.
fn scale_fields(state: &MeasureView) -> (f64, f64, String, Option<PageUnit>) {
    (state.drawing_points, state.real_distance, state.unit.clone(), state.page_unit)
}
/// Fill the scale fields for 1:N: 1 mm on the page is N mm, shown in metres. Returns `false`, changing nothing, when `n` is not a usable ratio.
pub(crate) fn set_ratio(state: &mut MeasureView, n: f64) -> bool {
    if !RATIO_RANGE.contains(&n) {
        return false;
    }
    state.unit = "m".into();
    state.page_unit = Some(PageUnit::Mm);
    state.drawing_points = 1.0;
    state.real_distance = n / 1000.0;
    state.ratio_text = format_ratio(n);
    true
}
impl MeasureView {
    pub fn cancel(&mut self) {
        self.points.clear();
        self.locked_angle = None;
        self.page = None;
        self.reading = None;
        self.scale = None;
    }

    fn lock_vertex(&mut self, last: Point, point: Point) -> Point {
        let dx = point[0] - last[0];
        let dy = point[1] - last[1];
        if dx.hypot(dy) < 1e-6 {
            return last;
        }
        let angle = *self.locked_angle.get_or_insert_with(|| {
            let step = std::f64::consts::FRAC_PI_4;
            (dy.atan2(dx) / step).round() * step
        });
        let (sin, cos) = angle.sin_cos();
        let length = (dx * cos + dy * sin).max(0.0);
        [last[0] + length * cos, last[1] + length * sin]
    }
}
/// Snaps `point` to the page's drawn geometry (endpoints, midpoints, intersections, paths),
/// recording in `state` why it couldn't when the content is too complex or unreadable.
fn snap_to_content(
    state: &mut MeasureView,
    doc: &Document,
    page: usize,
    view_xy: [f32; 2],
    pixels_per_unit: f32,
    point: Point,
) -> Option<(Point, measure::snap::SnapKind)> {
    if state.paths.as_ref().is_none_or(|(p, g, _)| *p != page || *g != doc.edit_generation()) {
        state.paths = Some((page, doc.edit_generation(), doc.measurement_paths(page)));
    }
    if let Some((_, _, Err(e))) = &state.paths {
        state.error = Some(e.clone());
    }
    if let Some((_, _, Ok(paths))) = &state.paths {
        if paths.truncated {
            state.error = Some(tl!("This drawing exceeds the snapping limit; only the first paths are used.").into());
        } else if paths.unreadable > 0 {
            state.error = Some(tl!("Some page content couldn't be read; snapping uses the rest.").into());
        }
        let tolerance = doc
            .measurement_to_user(page, [f64::from(view_xy[0]) + state.sensitivity / f64::from(pixels_per_unit), f64::from(view_xy[1])])
            .map(|p| measure::distance(point, p))
            .unwrap_or(state.sensitivity / f64::from(pixels_per_unit));
        if state.snaps.intersections && paths.intersection_limited(point, tolerance) {
            state.error = Some(tl!("This area has too many overlapping paths; intersection snapping is limited.").into());
        }
        if let Ok(Some(hit)) = paths.snap(point, tolerance, state.snaps) {
            return Some((hit.point, hit.kind));
        }
    }
    None
}

/// The Measure tools' own clicks this frame: those of a press nobody holds.
#[derive(Clone, Copy)]
pub(crate) struct Clicks {
    pub single: bool,
    pub double: bool,
}

impl Clicks {
    /// `resp`'s clicks, when the press is the tool's (`free`).
    pub(crate) fn of(resp: &egui::Response, free: bool) -> Self {
        Clicks { single: free && resp.clicked(), double: free && resp.double_clicked() }
    }
}

pub(crate) fn page_input(
    ui: &egui::Ui,
    clicks: Clicks,
    doc: &Document,
    view: &mut DocView,
    cx: &crate::comments::PageCx<'_>,
    tool: Tool,
    units: crate::units::Unit,
) {
    let page = cx.page;
    let xf = cx.xf;
    let author = &cx.prefs.author;
    if !doc.allows_annotation() {
        return;
    }
    let Some(info) = doc.info.pages.get(page) else { return };
    let state = &mut view.measure;
    if state.tool != Some(tool) {
        state.cancel();
        state.tool = Some(tool);
    }
    // Keys belong to a focused text field (the label, unit or viewport name) when there is one.
    let typing = ui.ctx().egui_wants_keyboard_input();
    if !typing && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        state.cancel();
    }
    let screen = |p: Point| {
        let v = info.user_to_view(p[0] as f32, p[1] as f32);
        xf.norm_to_screen(v[0] / xf.pw, v[1] / xf.ph)
    };
    let pos = cx.ptr.within(xf.rect);
    let shift = ui.input(|i| i.modifiers.shift);
    if !shift {
        state.locked_angle = None;
    }
    let mut hover = None;
    let mut snap = None;
    if let Some(pos) = pos {
        view.cursor.claim(Layer::ToolArea, egui::CursorIcon::Crosshair);
        let (vx, vy) = xf.screen_to_view(pos);
        let [x, y] = info.view_to_user(vx, vy);
        let mut point = [f64::from(x), f64::from(y)];
        if state.snap_enabled
            && let Some((snapped_point, kind)) = snap_to_content(state, doc, page, [vx, vy], xf.zoom().max(1e-6), point)
        {
            point = snapped_point;
            snap = Some(kind);
        }
        if shift
            && state.page == Some(page)
            && let Some(last) = state.points.last().copied()
        {
            let constrained = state.lock_vertex(last, point);
            if measure::distance(constrained, point) > 1e-6 {
                snap = None;
            }
            point = constrained;
        }
        hover = Some(point);
        if clicks.single {
            if state.page != Some(page) {
                state.cancel();
                state.page = Some(page);
            }
            if state.points.is_empty() {
                match doc.measurement_scale_or(page, point, units.per_point(), units.label()) {
                    Ok(scale) => {
                        state.scale = Some(scale);
                        state.error = None;
                    }
                    Err(e) => {
                        state.error = Some(e);
                        return;
                    }
                }
            }
            let closes =
                tool == Tool::Area && state.points.len() >= 3 && state.points.first().is_some_and(|first| screen(*first).distance(pos) <= 6.0);
            let repeats = state.points.last().is_some_and(|last| measure::distance(*last, point) < 1e-6);
            if !closes && !repeats && state.points.len() < measure::MAX_POINTS {
                state.points.push(point);
            }
            let finishes = closes || matches!(tool, Tool::Distance | Tool::Calibrate) && state.points.len() == 2;
            if finishes {
                finish(state, &mut view.pending_edit, doc, page, tool, author, units);
            }
        }
    }
    if state.page == Some(page) && !state.points.is_empty() {
        if (clicks.double || !typing && ui.input(|i| i.key_pressed(egui::Key::Enter))) && matches!(tool, Tool::Perimeter | Tool::Area) {
            finish(state, &mut view.pending_edit, doc, page, tool, author, units);
        }
        if !typing && ui.input(|i| i.key_pressed(egui::Key::Backspace)) {
            state.points.pop();
        }
        let mut points = state.points.clone();
        if let Some(p) = hover
            && points.last().is_none_or(|last| measure::distance(*last, p) > 1e-6)
        {
            points.push(p);
        }
        if let Some(scale) = &state.scale {
            match measure::reading(tool.kind(), &points, scale) {
                Ok(reading) => state.reading = Some(reading),
                Err(e) => state.error = Some(e.to_string()),
            }
        }
        let positions: Vec<Pos2> = points.iter().map(|&p| screen(p)).collect();
        let stroke = Stroke::new(1.5, Color32::from_rgb(0, 120, 214));
        for pair in positions.windows(2) {
            if let [a, b] = pair {
                ui.painter().line_segment([*a, *b], stroke);
            }
        }
        if tool == Tool::Area
            && let (Some(a), Some(b)) = (positions.first(), positions.last())
        {
            ui.painter().line_segment([*a, *b], stroke);
        }
        for &p in &positions {
            ui.painter().circle_filled(p, 3.0, stroke.color);
        }
        if let (Some(p), Some(kind)) = (hover, snap) {
            ui.painter().circle_stroke(screen(p), 6.0, Stroke::new(1.0, Color32::from_rgb(46, 158, 92)));
            ui.painter().text(
                screen(p) + egui::vec2(10.0, -10.0),
                egui::Align2::LEFT_BOTTOM,
                tl!(match kind {
                    measure::snap::SnapKind::Endpoint => "Endpoint",
                    measure::snap::SnapKind::Midpoint => "Midpoint",
                    measure::snap::SnapKind::Intersection => "Intersection",
                    measure::snap::SnapKind::Path => "Path",
                }),
                crate::theme::regular(11.0),
                stroke.color,
            );
        }
        if let (Some(p), Some(reading)) = (hover, &state.reading) {
            let label = if tool.kind() == Kind::Perimeter
                && let (Some(last), Some(scale)) = (state.points.last(), &state.scale)
                && let Ok(segment) = measure::reading(Kind::Distance, &[*last, p], scale)
            {
                format!("{}: {}\n{}: {}", tl!("Segment"), segment.label, tl!("Total"), reading.label)
            } else {
                reading.label.clone()
            };
            ui.painter().text(screen(p) + egui::vec2(12.0, 12.0), egui::Align2::LEFT_TOP, label, crate::theme::semibold(12.0), stroke.color);
        }
    }
}
fn finish(state: &mut MeasureView, pending: &mut Option<Edit>, doc: &Document, page: usize, tool: Tool, author: &str, units: crate::units::Unit) {
    if tool == Tool::Calibrate {
        if let [a, b] = state.points.as_slice() {
            let a = doc.measurement_to_view(page, *a);
            let b = doc.measurement_to_view(page, *b);
            if let (Ok(a), Ok(b)) = (a, b) {
                state.drawing_points = measure::distance(a, b) / page_unit_of(state.page_unit, &state.unit, units).points();
                state.vertical_multiplier = 1.0;
            }
        }
        state.cancel();
        return;
    }
    let minimum = if tool == Tool::Area { 3 } else { 2 };
    if state.points.len() < minimum {
        state.error = Some(crate::i18n::fmt(tl!("This measurement needs at least {n} points."), &[("n", &minimum.to_string())]));
        return;
    }
    if let Some(hvac) = &state.hvac
        && tool == Tool::Perimeter
        && let Err(error) = hvac.validate()
    {
        state.error = Some(error.to_string());
        return;
    }
    if let Some(scale) = state.scale.clone() {
        *pending = Some(Edit::AddMeasurement(NewMeasurement {
            hvac: if tool == Tool::Perimeter { state.hvac.clone() } else { None },
            page,
            kind: tool.kind(),
            points: state.points.clone(),
            scale,
            style: Style { color: [0.0, 0.47, 0.84], ..Style::default() },
            label: state.label.clone(),
            author: author.into(),
        }));
        state.cancel();
    }
}
pub(crate) fn panel(app: &mut PdfCraftApp, ui: &mut egui::Ui, t: &Tokens) {
    crate::takeoff_workspace::settings_header(app, ui);
    egui::ScrollArea::vertical().id_salt("measure-panel").show(ui, |ui| panel_body(app, ui, t));
}
fn measurement_row(ui: &mut egui::Ui, t: &Tokens, icon: &str, label: &str, active: bool, command: &str) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(ui.available_width(), 32.0), egui::Sense::click_and_drag());
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, active, label));
    crate::theme::Press::track(ui, &response).wash(ui, rect, 6, active);
    crate::icons::paint(ui, egui::Rect::from_center_size(rect.left_center() + egui::vec2(12.0, 0.0), egui::vec2(18.0, 18.0)), icon, 18.0, t.icon);
    widgets::paint_left(ui, rect.left_center() + egui::vec2(30.0, 0.0), label, crate::theme::regular(13.0), t.text, (rect.width() - 36.0).max(1.0));
    crate::tool_access::source(&response, format!("command:{command}"));
    response.on_hover_cursor(egui::CursorIcon::PointingHand)
}

/// Draw the measurement panel: tool selection, scale calibration, snapping and the current reading.
fn panel_body(app: &mut PdfCraftApp, ui: &mut egui::Ui, t: &Tokens) {
    let Some((index, id)) = app.active_ids() else {
        ui.label(tl!("Open a PDF to measure it."));
        return;
    };
    let Some(doc) = app.session.get(id) else { return };
    let units = app.units;
    let page = app.views[index].current;
    let info = doc.info.pages.get(page).cloned();
    let state = &mut app.views[index].measure;
    if state.seeded_page != Some(page) {
        state.seeded_page = Some(page);
        state.vertical_multiplier = 1.0;
        // One page unit (the one the scale's unit gives) and what it measures.
        if let Ok(a) = doc.measurement_to_user(page, [0.0, 0.0])
            && let Ok(scale) = doc.measurement_scale_or(page, a, units.per_point(), units.label())
            && let Ok(b) = doc.measurement_to_user(page, [page_unit_of(None, &scale.unit, units).points(), 0.0])
            && let Ok(c) = doc.measurement_to_user(page, [0.0, page_unit_of(None, &scale.unit, units).points()])
            && let Ok(reading) = measure::reading(Kind::Distance, &[a, b], &scale)
            && let Ok(vertical) = measure::reading(Kind::Distance, &[a, c], &scale)
            && reading.value > 0.0
        {
            state.drawing_points = 1.0;
            state.real_distance = reading.value;
            state.vertical_multiplier = vertical.value / reading.value;
            state.unit = scale.unit;
            state.page_unit = None;
            state.precision = scale.precision;
        }
        sync_ratio_text(state, units);
    }

    let mut tool = None;
    widgets::section_title(ui, tl!("Takeoff"));
    for (label, icon, value) in
        [("Distance", "ruler", Tool::Distance), ("Continuous length", "spline", Tool::Perimeter), ("Area", "square", Tool::Area)]
    {
        if measurement_row(
            ui,
            t,
            icon,
            tl!(label),
            app.quick_tool == QuickTool::Measure(value) && state.hvac.is_none(),
            &format!("measure.{}", value.name()),
        )
        .clicked()
        {
            state.hvac = None;
            tool = Some(value);
        }
    }
    if measurement_row(ui, t, "wrench", tl!("Pipe takeoff"), state.hvac.as_ref().is_some_and(|h| h.profile == "pipe"), "measure.pipe").clicked() {
        state.hvac = Some(measure::hvac::Takeoff::pipe());
        tool = Some(Tool::Perimeter);
    }
    if measurement_row(ui, t, "columns-2", tl!("Sheet-metal takeoff"), state.hvac.as_ref().is_some_and(|h| h.profile != "pipe"), "measure.duct")
        .clicked()
    {
        state.hvac = Some(measure::hvac::Takeoff::duct());
        tool = Some(Tool::Perimeter);
    }

    widgets::section_title(ui, tl!("Drawing scale"));
    let mut chosen = None;
    let mut preset = None;
    ui.horizontal(|ui| {
        let groups = [
            (tl!("Metric"), measure::presets::METRIC),
            (tl!("Architectural"), measure::presets::ARCHITECTURAL),
            (tl!("Engineering"), measure::presets::ENGINEERING),
        ];
        craft_ui::ComboBox::from_id_salt("measure-common-scales")
            .selected_text(tl!("Common scales"))
            .width(20.0)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .show_ui_with_header(
                ui,
                |ui| {
                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut state.half_size_presets, false, tl!("Full size"));
                        ui.selectable_value(&mut state.half_size_presets, true, tl!("Half size"));
                    });
                    ui.horizontal(|ui| {
                        for (index, (label, _)) in groups.iter().enumerate() {
                            ui.selectable_value(&mut state.preset_family, index, *label);
                        }
                    });
                    groups.get(state.preset_family).map_or(measure::presets::METRIC, |(_, group)| *group)
                },
                |ui, group| {
                    for item in group {
                        if ui.selectable_label(false, item.label).clicked() {
                            preset = Some(*item);
                            ui.close();
                        }
                    }
                },
            );
    });
    if let Some(scale) = &state.scale {
        ui.label(egui::RichText::new(&scale.ratio).color(t.text_muted));
    } else {
        let paper = page_unit_of(state.page_unit, &state.unit, units);
        ui.label(
            egui::RichText::new(format!("{} {} = {} {}", state.drawing_points, paper.label(), state.real_distance, state.unit)).color(t.text_muted),
        );
    }
    if measurement_row(ui, t, "ruler", tl!("Calibrate from two points"), false, "measure.calibrate").clicked() {
        tool = Some(Tool::Calibrate);
    }
    let fields = scale_fields(state);
    egui::CollapsingHeader::new(tl!("Custom scale")).show(ui, |ui| {
        widgets::begin_dialog(ui);
        ui.label(tl!("Scale ratio"));
        let typed = ui.add(widgets::line(&mut state.ratio_text).hint_text("1:100").desired_width(ui.available_width()).char_limit(16));
        typed.widget_info(|| egui::WidgetInfo::labeled(egui::Role::TextInput, true, tl!("Scale")));
        state.ratio_dirty |= typed.changed();
        if typed.lost_focus() && std::mem::take(&mut state.ratio_dirty) {
            match parse_ratio(&state.ratio_text) {
                Some(n) => chosen = Some(n),
                None => state.error = Some(tl!("Enter a scale such as 1:50.").into()),
            }
        }
        ui.label(tl!("On drawing"));
        ui.horizontal_wrapped(|ui| {
            ui.add(egui::DragValue::new(&mut state.drawing_points).range(0.000001..=1e9));
            let paper = page_unit_of(state.page_unit, &state.unit, units);
            craft_ui::ComboBox::from_label(tl!("Paper unit")).selected_text(paper.label()).width(55.0).show_ui(ui, |ui| {
                for unit in PageUnit::ALL {
                    if ui.selectable_label(unit == paper, unit.label()).clicked() {
                        state.page_unit = Some(unit);
                    }
                }
            });
        });
        ui.label(tl!("Real distance"));
        ui.horizontal_wrapped(|ui| {
            ui.add(egui::DragValue::new(&mut state.real_distance).range(0.000001..=1e12));
            let label = ui.label(tl!("Unit"));
            ui.add(widgets::line(&mut state.unit).desired_width(60.0).char_limit(12)).labelled_by(label.id);
        });
        ui.label(tl!("Decimals"));
        ui.add(egui::DragValue::new(&mut state.precision).range(0..=6));
        ui.label(tl!("Vertical scale multiplier"));
        ui.add(egui::DragValue::new(&mut state.vertical_multiplier).speed(0.001));
        ui.add(widgets::checkbox(&mut state.whole_page, tl!("Entire page")));
        if !state.whole_page {
            for (value, name) in state.rect.iter_mut().zip(["X", "Y", "Width", "Height"]) {
                ui.label(tl!(name));
                ui.add(egui::DragValue::new(value).range(0.0..=1e9));
            }
        }
        let label = ui.label(tl!("Viewport"));
        ui.add(widgets::line(&mut state.name).desired_width(ui.available_width()).char_limit(128)).labelled_by(label.id);
    });
    if scale_fields(state) != fields {
        sync_ratio_text(state, units);
    }
    let apply = measurement_row(ui, t, "check", tl!("Apply scale"), false, "measure.scale").clicked() && doc.allows_annotation();

    widgets::section_title(ui, tl!("Snapping"));
    ui.add(widgets::checkbox(&mut state.snap_enabled, tl!("Snap to drawing")));
    egui::CollapsingHeader::new(tl!("Snap options")).show(ui, |ui| {
        ui.add(widgets::checkbox(&mut state.snaps.endpoints, tl!("Endpoints")));
        ui.add(widgets::checkbox(&mut state.snaps.midpoints, tl!("Midpoints")));
        ui.add(widgets::checkbox(&mut state.snaps.intersections, tl!("Intersections")));
        ui.add(widgets::checkbox(&mut state.snaps.paths, tl!("Paths")));
        ui.add(egui::Slider::new(&mut state.sensitivity, 1.0..=20.0).text(tl!("Sensitivity")));
    });

    if let Some(hvac) = &mut state.hvac {
        widgets::section_title(ui, tl!("HVAC specifications"));
        let previous = hvac.profile.clone();
        for (name, value, choices) in [
            ("Profile", &mut hvac.profile, &["pipe", "round duct", "rectangular duct"][..]),
            (
                "Material",
                &mut hvac.material,
                &["Copper", "Carbon steel", "Stainless steel", "PVC", "CPVC", "PEX", "Galvanized steel", "Aluminum"][..],
            ),
            (
                "System",
                &mut hvac.service,
                &["Supply", "Return", "Exhaust", "Outside air", "Hydronic supply", "Hydronic return", "Refrigerant", "Condensate"][..],
            ),
        ] {
            ui.label(egui::RichText::new(tl!(name)).small().color(t.text_muted));
            craft_ui::ComboBox::from_id_salt(("hvac", name)).selected_text(value.as_str()).width(ui.available_width()).show_ui(ui, |ui| {
                for choice in choices {
                    ui.selectable_value(value, (*choice).into(), *choice);
                }
            });
            ui.add_space(5.0);
        }
        if previous != hvac.profile {
            hvac.size = match hvac.profile.as_str() {
                "pipe" => "1 in",
                "round duct" => "12 in",
                _ => "12 x 12 in",
            }
            .into();
        }
        ui.label(egui::RichText::new(tl!("Nominal size")).small().color(t.text_muted));
        let sizes = match hvac.profile.as_str() {
            "pipe" => measure::hvac::PIPE_SIZES,
            "round duct" => measure::hvac::ROUND_DUCT_SIZES,
            _ => measure::hvac::RECTANGULAR_DUCT_SIZES,
        };
        craft_ui::ComboBox::from_id_salt("hvac-nominal-size").selected_text(&hvac.size).width(ui.available_width()).show_ui(ui, |ui| {
            for size in sizes {
                ui.selectable_value(&mut hvac.size, (*size).into(), *size);
            }
        });
        egui::CollapsingHeader::new(tl!("Custom size")).show(ui, |ui| {
            widgets::begin_dialog(ui);
            ui.label(tl!("Diameter or width × height, with units"));
            ui.add(widgets::line(&mut hvac.size).desired_width(ui.available_width()).char_limit(64));
        });
    }
    widgets::section_title(ui, tl!("Markup"));
    let label = ui.label(tl!("Label"));
    widgets::begin_dialog(ui);
    ui.add(widgets::line(&mut state.label).hint_text(tl!("Optional label")).desired_width(ui.available_width()).char_limit(128))
        .labelled_by(label.id);
    if let Some(reading) = &state.reading {
        ui.label(egui::RichText::new(&reading.label).strong());
    }
    if let Some(error) = &state.error {
        ui.colored_label(Color32::from_rgb(190, 50, 50), error);
    }

    if let Some(n) = chosen
        && set_ratio(state, n)
    {
        state.error = None;
    }
    if let Some(preset) = preset {
        let (paper, page_unit) = preset.paper(state.half_size_presets);
        state.drawing_points = paper;
        state.page_unit = Some(page_unit);
        state.real_distance = preset.real;
        state.unit = preset.unit.into();
        sync_ratio_text(state, units);
        state.error = None;
    }
    // Apply scale, or a 1:N typed or picked: the fields are applied to the document at once.
    if (apply || chosen.is_some() || preset.is_some())
        && doc.allows_annotation()
        && let Some(info) = info
    {
        let result =
            doc.measurement_to_user(page, [0.0, 0.0]).and_then(|a| doc.measurement_to_user(page, [1.0, 0.0]).map(|b| (a, b))).and_then(|(a, b)| {
                let page_unit = page_unit_of(state.page_unit, &state.unit, units);
                Scale::ratio(a, b, state.drawing_points, page_unit, state.real_distance, &state.unit, state.precision)
                    .and_then(|scale| scale.with_vertical_multiplier(a, b, state.vertical_multiplier))
                    .map_err(|e| e.to_string())
            });
        match result {
            Ok(scale) => {
                let [x, y, w, h] = if state.whole_page { [0.0, 0.0, f64::from(info.width), f64::from(info.height)] } else { state.rect };
                let a = info.view_to_user(x as f32, y as f32);
                let b = info.view_to_user((x + w) as f32, (y + h) as f32);
                app.views[index].pending_edit = Some(Edit::SetMeasurementScale {
                    page,
                    bbox: [f64::from(a[0].min(b[0])), f64::from(a[1].min(b[1])), f64::from(a[0].max(b[0])), f64::from(a[1].max(b[1]))],
                    name: state.name.clone(),
                    scale,
                });
            }
            Err(e) => state.error = Some(e.to_string()),
        }
    }
    if let Some(tool) = tool {
        app.views[index].measure.cancel();
        app.quick_tool = QuickTool::Measure(tool);
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn export_workbook(app: &mut PdfCraftApp) {
    let Some((_, id)) = app.active_ids() else { return };
    let Some(doc) = app.session.get(id) else { return };
    if !doc.allows_copying() {
        return;
    }
    let listing = match doc.measurements() {
        Ok(listing) => listing,
        Err(error) => {
            app.notify(error);
            return;
        }
    };
    let write = move |app: &mut PdfCraftApp, mut path: std::path::PathBuf| {
        if !path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("xlsx")) {
            path.set_extension("xlsx");
        }
        match measure::xlsx::write_new(&path, &listing.measurements) {
            Ok(()) if listing.truncated || !listing.unsupported.is_empty() => {
                app.notify(tl!("Saved an incomplete takeoff workbook. Some measurements were omitted."));
            }
            Ok(()) => app.notify(tl!("Saved measurements.")),
            Err(error) => app.notify(error),
        }
    };
    if let Some(dir) = &app.export_dir_override {
        write(app, std::path::PathBuf::from(dir).join("takeoff.xlsx"));
    } else {
        let dialog = rfd::AsyncFileDialog::new().set_file_name("takeoff.xlsx").add_filter_any_case("XLSX", &["xlsx"]);
        app.ask_one(crate::pickers::Ask::Save(dialog), None, write);
    }
}

pub(crate) fn command(app: &mut PdfCraftApp, id: &str) {
    app.left = LeftPanel::Tool("measure");
    app.left_open = true;
    if let Some((index, _)) = app.active_ids()
        && let Some(tool) = id.strip_prefix("measure.").and_then(Tool::from_name)
    {
        app.views[index].measure.cancel();
        if matches!(tool, Tool::Distance | Tool::Perimeter | Tool::Area) {
            app.views[index].measure.hvac = None;
        }
        app.quick_tool = QuickTool::Measure(tool);
    }
    if let Some((index, _)) = app.active_ids()
        && matches!(id, "measure.pipe" | "measure.duct")
    {
        app.views[index].measure.cancel();
        app.views[index].measure.hvac = Some(if id == "measure.pipe" { measure::hvac::Takeoff::pipe() } else { measure::hvac::Takeoff::duct() });
        app.quick_tool = QuickTool::Measure(Tool::Perimeter);
    }
    if id == "measure.export" {
        export_csv(app);
    }
}
pub(crate) fn export_csv(app: &mut PdfCraftApp) {
    let Some((_, id)) = app.active_ids() else { return };
    let result = app.session.get(id).ok_or_else(|| "no such document".to_string()).and_then(|d| d.measurements());
    match result {
        Err(e) => app.notify(&e),
        Ok(listing) => {
            let csv = measure::csv(&listing.measurements);
            if !listing.unsupported.is_empty() {
                app.notify(crate::i18n::fmt(
                    tl!("{n} measurements use an unsupported format and are left out."),
                    &[("n", &listing.unsupported.len().to_string())],
                ));
            }
            #[cfg(not(target_arch = "wasm32"))]
            {
                let write = move |app: &mut PdfCraftApp, path: std::path::PathBuf| match crate::editing::write_atomically(
                    &path.to_string_lossy(),
                    csv.as_bytes(),
                ) {
                    Ok(()) => app.notify(tl!("Saved measurements.")),
                    Err(e) => app.notify(e.to_string()),
                };
                match app.export_dir_override.as_ref().map(|d| std::path::PathBuf::from(d).join("measurements.csv")) {
                    Some(path) => write(app, path),
                    None => {
                        let dialog = rfd::AsyncFileDialog::new().set_file_name("measurements.csv").add_filter_any_case("CSV", &["csv"]);
                        app.ask_one(crate::pickers::Ask::Save(dialog), None, write);
                    }
                }
            }
            #[cfg(target_arch = "wasm32")]
            if let Err(e) = crate::editing::download("measurements.csv", csv.as_bytes()) {
                app.notify(&e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn shift_keeps_the_initial_direction_until_released() {
        let mut state = super::MeasureView::default();
        let horizontal = state.lock_vertex([0.0, 0.0], [10.0, 1.0]);
        assert!(horizontal[1].abs() < 1e-9);
        let still_horizontal = state.lock_vertex([10.0, 0.0], [11.0, 10.0]);
        assert!(still_horizontal[1].abs() < 1e-9);
        state.locked_angle = None;
        let vertical = state.lock_vertex([10.0, 0.0], [11.0, 10.0]);
        assert!((vertical[0] - 10.0).abs() < 1e-9);
        state.locked_angle = None;
        let diagonal = state.lock_vertex([0.0, 0.0], [9.0, 10.0]);
        assert!((diagonal[0] - diagonal[1]).abs() < 1e-9);
    }

    use super::*;

    /// What Apply scale builds from the panel's fields, then a distance read with it.
    fn measured(state: &MeasureView, from: Point, to: Point) -> f64 {
        let page_unit = page_unit_of(state.page_unit, &state.unit, crate::units::Unit::Points);
        let scale = Scale::ratio([0.0, 0.0], [1.0, 0.0], state.drawing_points, page_unit, state.real_distance, &state.unit, 4).expect("scale");
        measure::reading(Kind::Distance, &[from, to], &scale).expect("reading").value
    }

    #[test]
    fn typed_ratios_are_read_leniently() {
        assert_eq!(parse_ratio("1:200"), Some(200.0));
        assert_eq!(parse_ratio(" 1 : 50 "), Some(50.0));
        assert_eq!(parse_ratio("1/20"), Some(20.0));
        assert_eq!(parse_ratio("250"), Some(250.0));
        assert_eq!(parse_ratio("1:2,5"), Some(2.5));
        assert_eq!(parse_ratio("2:1"), Some(0.5));
        assert_eq!(parse_ratio("1:0.001"), Some(1e-3));
    }

    #[test]
    fn invalid_ratios_are_refused() {
        for bad in [
            "", " ", ":", "1:", ":50", "1:0", "0", "-5", "1:-5", "0:5", "abc", "1:abc", "1:50:2", "1:inf", "NaN", "1e12", "1:1e12", "1:0.0001",
            "1:1e-6",
        ] {
            assert_eq!(parse_ratio(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn ratios_are_written_as_people_say_them() {
        assert_eq!(format_ratio(50.0), "1:50");
        assert_eq!(format_ratio(2.5), "1:2.5");
        assert_eq!(format_ratio(0.5), "1:0.5");
        // Nothing is rounded away: the field reads the ratio that applies.
        assert_eq!(format_ratio(1.234), "1:1.234");
        assert_eq!(format_ratio(0.001), "1:0.001");
        assert_eq!(parse_ratio(&format_ratio(333.333333)), Some(333.333333));
        for n in RATIOS {
            assert_eq!(parse_ratio(&format_ratio(n)), Some(n));
        }
    }

    #[test]
    fn every_offered_ratio_reads_paper_length_times_n_in_metres_whatever_the_unit_was() {
        for unit in ["mm", "cm", "m", "km", "in", "ft"] {
            for n in RATIOS {
                let mut state = MeasureView { unit: unit.into(), ..Default::default() };
                assert!(set_ratio(&mut state, n));
                assert_eq!(state.unit, "m");
                // 72 pt is one inch of paper: 25.4 mm x N, shown in metres.
                let v = measured(&state, [0.0, 0.0], [0.0, 72.0]);
                let expected = 25.4 * n / 1000.0;
                assert!((v - expected).abs() < 1e-9 * expected.max(1.0), "1:{n} in {unit} gave {v}, wanted {expected}");
            }
        }
    }

    #[test]
    fn any_starting_unit_becomes_metres() {
        let mut state = MeasureView { unit: "yards?".into(), ..Default::default() };
        assert!(set_ratio(&mut state, 50.0));
        assert_eq!(state.unit, "m");
        // 1 mm of paper at 1:50 is 50 mm.
        let v = measured(&state, [0.0, 0.0], [72.0 / 25.4, 0.0]);
        assert!((v - 0.05).abs() < 1e-9, "{v}");
    }

    #[test]
    fn the_smallest_ratio_keeps_the_real_distance_within_its_field() {
        let mut state = MeasureView::default();
        assert!(set_ratio(&mut state, parse_ratio("1:0.001").expect("ratio")));
        // The real-distance field accepts 0.000001 and no less.
        assert!(state.real_distance >= 0.000001, "{}", state.real_distance);
        assert!(!set_ratio(&mut state, 1e-4));
    }

    #[test]
    fn the_ratio_field_follows_the_scale_fields() {
        let display = crate::units::Unit::Points;
        let mut state = MeasureView::default();
        for n in [50.0, 2.5, 1.234, 0.001, 1e9] {
            assert!(set_ratio(&mut state, n));
            sync_ratio_text(&mut state, display);
            assert_eq!(state.ratio_text, format_ratio(n), "1:{n}");
        }
        // A hand edit that is still a drawing scale reads as its ratio, in another page unit too.
        assert!(set_ratio(&mut state, 50.0));
        state.page_unit = Some(PageUnit::Cm);
        sync_ratio_text(&mut state, display);
        assert_eq!(state.ratio_text, "1:5");
        state.page_unit = Some(PageUnit::Mm);
        state.real_distance = 0.1;
        sync_ratio_text(&mut state, display);
        assert_eq!(state.ratio_text, "1:100");
        // One that isn't (another unit, or no ratio at all) empties it.
        state.unit = "ft".into();
        sync_ratio_text(&mut state, display);
        assert_eq!(state.ratio_text, "");
        state.unit = "m".into();
        state.real_distance = 0.1 / 3.0;
        sync_ratio_text(&mut state, display);
        assert_eq!(state.ratio_text, "");
        state.drawing_points = 0.0;
        sync_ratio_text(&mut state, display);
        assert_eq!(state.ratio_text, "");
    }

    #[test]
    fn an_unusable_ratio_changes_nothing() {
        let mut state = MeasureView::default();
        let before = (state.drawing_points, state.real_distance, state.unit.clone(), state.page_unit);
        for n in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(!set_ratio(&mut state, n));
        }
        assert_eq!((state.drawing_points, state.real_distance, state.unit.clone(), state.page_unit), before);
    }
}
