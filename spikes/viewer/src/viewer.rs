//! Desktop shell. PDF engine objects stay inside the adapter.

use std::path::{Path, PathBuf};

use eframe::egui::{self, Color32, ColorImage, CursorIcon, Key, Modifiers, PointerButton, Pos2, Rect, TextureHandle, Vec2};
use simplypdf_core::geometry::{PageGeometry, Point};
use simplypdf_draw::viewport::ZoomWindow;
use simplypdf_draw::zoom_history::ZoomHistory;
use simplypdf_draw::zoom_tool::{DragEffect, DragMove, Mode, Release, ZoomRect};

use crate::mac_menu::NativeMenu;
use crate::menu::{self, Command};
use crate::pdf::{Document, RasterPage};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tool {
    Inspect,
    Pan,
    Zoom,
    Rectangle,
}

pub struct Viewer {
    native_menu: Option<NativeMenu>,
    document: Option<Document>,
    page_index: usize,
    image: Option<(TextureHandle, PageGeometry, Rect)>,
    selected: Option<Point>,
    navigation: Navigation,
    zoom_history: ZoomHistory<Navigation>,
    tool: Tool,
    zoom_start: Option<Pos2>,
    rectangle_start: Option<Point>,
    rectangles: Vec<(usize, [Point; 2])>,
    undone_rectangles: Vec<(usize, [Point; 2])>,
    primary_pan: bool,
    error: Option<String>,
}

impl Viewer {
    pub fn new(context: &egui::Context, path: PathBuf) -> Self {
        let mut viewer = Self {
            native_menu: None,
            document: None,
            page_index: 0,
            image: None,
            selected: None,
            navigation: Navigation::default(),
            zoom_history: ZoomHistory::default(),
            tool: Tool::Inspect,
            zoom_start: None,
            rectangle_start: None,
            rectangles: Vec::new(),
            undone_rectangles: Vec::new(),
            primary_pan: false,
            error: None,
        };
        viewer.open(context, &path);
        viewer
    }

    fn open(&mut self, context: &egui::Context, path: &Path) {
        let result = Document::open(path).and_then(|document| {
            let first_page = document.render(0)?;
            Ok((document, first_page))
        });
        let title = path.file_name().unwrap_or(path.as_os_str()).to_string_lossy();
        match result {
            Ok((document, first_page)) => {
                context.send_viewport_cmd(egui::ViewportCommand::Title(format!("{title} — SimplyPDF")));
                self.document = Some(document);
                self.rectangles.clear();
                self.undone_rectangles.clear();
                self.display_page(context, 0, first_page);
            }
            Err(error) => self.error = Some(format!("Could not open {title}. {error}")),
        }
    }

    fn show_page(&mut self, context: &egui::Context, index: usize) {
        if let Some(document) = &self.document {
            match document.render(index) {
                Ok(page) => self.display_page(context, index, page),
                Err(error) => self.error = Some(error),
            }
        }
    }

    fn display_page(&mut self, context: &egui::Context, index: usize, page: RasterPage) {
        let image = ColorImage::from_rgba_unmultiplied(page.size, &page.rgba);
        let texture = context.load_texture("PDF page", image, egui::TextureOptions::LINEAR);
        let uv = Rect::from_min_max(Pos2::ZERO, Pos2::from(page.uv_max));
        self.image = Some((texture, page.geometry, uv));
        self.page_index = index;
        self.selected = None;
        self.primary_pan = false;
        self.zoom_start = None;
        self.rectangle_start = None;
        self.navigation = Navigation::default();
        self.zoom_history = ZoomHistory::default();
        self.zoom_history.insert(self.navigation);
        self.error = None;
    }

    pub fn install_native_menu(&mut self, context: &egui::Context) {
        self.native_menu = NativeMenu::install(context);
    }

    fn command_enabled(&self, command: Command) -> bool {
        match command {
            Command::Open => true,
            Command::Undo => !self.rectangles.is_empty(),
            Command::Redo => !self.undone_rectangles.is_empty(),
            Command::FirstPage | Command::PreviousPage => self.page_index > 0,
            Command::NextPage | Command::LastPage => self.document.as_ref()
                .is_some_and(|document| self.page_index + 1 < document.page_count()),
            Command::ZoomPrevious => self.zoom_history.can_step_back(),
            Command::ZoomNext => self.zoom_history.can_step_forward(),
            _ => self.image.is_some(),
        }
    }

    fn apply_navigation(&mut self, command: Command) {
        let before = self.navigation;
        match command {
            Command::ZoomPrevious => {
                if let Some(view) = self.zoom_history.step_back() {
                    self.navigation = *view;
                }
                return;
            }
            Command::ZoomNext => {
                if let Some(view) = self.zoom_history.step_forward() {
                    self.navigation = *view;
                }
                return;
            }
            Command::ZoomIn => self.navigation.zoom_at(1.25, Vec2::ZERO),
            Command::ZoomOut => self.navigation.zoom_at(1.0 / 1.25, Vec2::ZERO),
            Command::FitPage => self.navigation = Navigation::default(),
            _ => return,
        }
        self.record_navigation(before);
    }

    fn record_navigation(&mut self, before: Navigation) {
        if self.navigation != before {
            if self.zoom_history.current() != Some(&before) {
                self.zoom_history.insert(before);
            }
            self.zoom_history.insert(self.navigation);
        }
    }

    fn execute(&mut self, context: &egui::Context, command: Command) {
        if !self.command_enabled(command) {
            return;
        }
        match command {
            Command::Open => match crate::file_picker::open_document() {
                Ok(Some(path)) => self.open(context, &path),
                Ok(None) => {}
                Err(error) => self.error = Some(error.into()),
            },
            Command::Undo => {
                if let Some(rectangle) = self.rectangles.pop() {
                    self.undone_rectangles.push(rectangle);
                }
            }
            Command::Redo => {
                if let Some(rectangle) = self.undone_rectangles.pop() {
                    self.rectangles.push(rectangle);
                }
            }
            Command::FirstPage => self.show_page(context, 0),
            Command::PreviousPage => self.show_page(context, self.page_index - 1),
            Command::NextPage => self.show_page(context, self.page_index + 1),
            Command::LastPage => {
                if let Some(document) = &self.document {
                    self.show_page(context, document.page_count() - 1);
                }
            }
            Command::InspectTool => self.tool = Tool::Inspect,
            Command::PanTool => self.tool = Tool::Pan,
            Command::ZoomTool => self.tool = Tool::Zoom,
            Command::Rectangle => self.tool = Tool::Rectangle,
            command => self.apply_navigation(command),
        }
    }

    fn draw(&mut self, context: &egui::Context) {
        let dropped = context.input(|input| input.raw.dropped_files.first().and_then(|file| file.path.clone()));
        if let Some(path) = dropped {
            self.open(context, &path);
        }
        let commands = if let Some(menu) = &self.native_menu {
            menu.take_commands()
        } else {
            let mut commands: Vec<_> = menu::window_menu(context, |command| self.command_enabled(command)).into_iter().collect();
            context.input_mut(|input| {
                let mut redo = Modifiers::COMMAND;
                redo.shift = true;
                if input.consume_key(redo, Key::Z) {
                    commands.push(Command::Redo);
                } else if input.consume_key(Modifiers::COMMAND, Key::Z) {
                    commands.push(Command::Undo);
                }
                for (key, command) in [
                    (Key::O, Command::Open),
                    (Key::Equals, Command::ZoomIn),
                    (Key::Plus, Command::ZoomIn),
                    (Key::Minus, Command::ZoomOut),
                    (Key::Num0, Command::FitPage),
                ] {
                    if input.consume_key(Modifiers::COMMAND, key) {
                        commands.push(command);
                    }
                }
            });
            commands
        };
        for command in commands {
            self.execute(context, command);
        }

        let count = self.document.as_ref().map_or(0, Document::page_count);
        if count > 0 {
            context.input_mut(|input| {
                if input.consume_key(Modifiers::SHIFT, Key::V) {
                    self.tool = Tool::Pan;
                } else if input.consume_key(Modifiers::NONE, Key::V) {
                    self.tool = Tool::Inspect;
                } else if input.consume_key(Modifiers::NONE, Key::Z) {
                    self.tool = Tool::Zoom;
                }
            });
            let next = context.input_mut(|input| {
                if input.consume_key(Modifiers::NONE, Key::PageDown) {
                    self.page_index.saturating_add(1).min(count - 1)
                } else if input.consume_key(Modifiers::NONE, Key::PageUp) {
                    self.page_index.saturating_sub(1)
                } else {
                    self.page_index
                }
            });
            if next != self.page_index {
                self.show_page(context, next);
            }
        }
        egui::TopBottomPanel::top("viewer_toolbar").show(context, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                if ui.add_enabled(self.page_index > 0, egui::Button::new("‹"))
                    .on_hover_text("Previous page (Page Up)").clicked()
                {
                    self.show_page(context, self.page_index - 1);
                }
                ui.label(if count == 0 { "No PDF".into() } else { format!("{} / {count}", self.page_index + 1) });
                if ui.add_enabled(self.page_index + 1 < count, egui::Button::new("›"))
                    .on_hover_text("Next page (Page Down)").clicked()
                {
                    self.show_page(context, self.page_index + 1);
                }
                ui.separator();
                ui.add_enabled_ui(self.image.is_some(), |ui| {
                    if ui.button("−").on_hover_text("Zoom out").clicked() {
                        self.apply_navigation(Command::ZoomOut);
                    }
                    if ui.button("+").on_hover_text("Zoom in").clicked() {
                        self.apply_navigation(Command::ZoomIn);
                    }
                    if ui.button("Fit Page").clicked() {
                        self.apply_navigation(Command::FitPage);
                    }
                    ui.separator();
                    ui.selectable_value(&mut self.tool, Tool::Inspect, "Inspect")
                        .on_hover_text("Inspect a drawing point (V)");
                    ui.selectable_value(&mut self.tool, Tool::Pan, "Pan")
                        .on_hover_text("Pan the drawing (Shift+V)");
                    ui.selectable_value(&mut self.tool, Tool::Zoom, "Zoom")
                        .on_hover_text("Click or drag to zoom (Z)");
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::widgets::global_theme_preference_switch(ui);
                });
            });
        });
        egui::TopBottomPanel::top("drawing_toolbar").show(context, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                ui.strong("Drawing");
                ui.separator();
                if ui.add_enabled(self.image.is_some(), egui::Button::new("▭ Rectangle")
                    .selected(self.tool == Tool::Rectangle)).clicked()
                {
                    self.execute(context, Command::Rectangle);
                }
                ui.separator();
                for (command, label) in [(Command::Undo, "Undo"), (Command::Redo, "Redo")] {
                    if ui.add_enabled(self.command_enabled(command), egui::Button::new(label)).clicked() {
                        self.execute(context, command);
                    }
                }
                ui.label("Session markups");
            });
        });
        egui::SidePanel::left("sheets").default_width(144.0).show(context, |ui| {
            ui.heading("Sheets");
            ui.separator();
            egui::ScrollArea::vertical().show(ui, |ui| {
                for index in 0..count {
                    if ui.selectable_label(index == self.page_index, format!("Page {}", index + 1)).clicked()
                        && index != self.page_index
                    {
                        self.show_page(context, index);
                    }
                }
            });
        });
        if let Some(error) = &self.error {
            let mut dismiss = false;
            egui::TopBottomPanel::top("open_error").show(context, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(ui.visuals().error_fg_color, error);
                    dismiss = ui.button("Dismiss").clicked();
                });
            });
            if dismiss {
                self.error = None;
            }
        }
        egui::TopBottomPanel::bottom("status").show(context, |ui| {
            ui.horizontal(|ui| {
                ui.label(if count == 0 { "No document".into() } else {
                    format!("Page {} of {count} · {:.0}% of fit", self.page_index + 1, self.navigation.zoom * 100.0)
                });
                ui.separator();
                if let Some([x, y]) = self.selected {
                    ui.monospace(format!("Page coordinates  x {x:.2} pt   y {y:.2} pt"));
                } else {
                    ui.label(match self.tool {
                        Tool::Pan => "Drag to pan",
                        Tool::Rectangle => "Drag on the page to add a session rectangle",
                        Tool::Zoom => "Click or drag to zoom · ⌘-click to zoom out",
                        Tool::Inspect => "Click to inspect · Scroll to zoom · Hold Space to pan",
                    });
                }
            });
        });
        egui::CentralPanel::default().show(context, |ui| {
            let Some((texture, geometry, uv)) = self.image.as_ref().map(|(texture, geometry, uv)| (texture.id(), *geometry, *uv)) else {
                ui.label("Open a PDF or drop one into this window.");
                return;
            };
            let (canvas, response) =
                ui.allocate_exact_size(ui.available_size(), egui::Sense::click_and_drag());
            let bounds = canvas.shrink(20.0);
            if !bounds.is_positive() {
                return;
            }
            let Ok(base) = geometry.viewport([0.0, 0.0], 1.0) else { return };
            let [width, height] = base.size();
            let fit = (f64::from(bounds.width()) / width).min(f64::from(bounds.height()) / height);

            if response.double_clicked_by(PointerButton::Middle) {
                self.apply_navigation(Command::FitPage);
            }
            let (space_pan, primary_pressed, primary_down, middle_down) = ui.input(|input| {
                (
                    input.key_down(Key::Space),
                    input.pointer.button_pressed(PointerButton::Primary),
                    input.pointer.button_down(PointerButton::Primary),
                    input.pointer.button_down(PointerButton::Middle),
                )
            });
            let pan_mode = self.tool == Tool::Pan || space_pan;
            if self.tool != Tool::Zoom {
                self.zoom_start = None;
            }
            let canvas_pressed = response.is_pointer_button_down_on();
            if primary_pressed {
                // Keep the press intent through release, even if Space is released first.
                self.primary_pan = canvas_pressed && pan_mode;
            }
            if response.dragged_by(PointerButton::Middle)
                || (self.primary_pan && response.dragged_by(PointerButton::Primary))
            {
                self.navigation.pan += response.drag_delta();
            }
            if let Some(pointer) = response.hover_pos() {
                let zoom = ui.input(|input| {
                    let scroll = input.smooth_scroll_delta.y.clamp(-300.0, 300.0);
                    let gesture = input.zoom_delta();
                    if gesture == 1.0 { (scroll / 240.0).exp2() } else { gesture }
                });
                self.navigation.zoom_at(zoom, pointer - bounds.center());
            }

            let mut zoom_outline = None;
            if self.tool == Tool::Zoom {
                let (pointer, released, zoom_out) = ui.input(|input| {
                    (
                        input.pointer.latest_pos(),
                        input.pointer.button_released(PointerButton::Primary),
                        input.modifiers.command,
                    )
                });
                if primary_pressed && !pan_mode && canvas_pressed {
                    self.zoom_start = pointer;
                }
                if let (Some(start), Some(end)) = (self.zoom_start, pointer) {
                    let selection = zoom_selection(start, end, bounds);
                    if primary_down {
                        if response.dragged_by(PointerButton::Primary) {
                            zoom_outline = Some(Rect::from_two_pos(start, end));
                        }
                    } else if released {
                        let before = self.navigation;
                        self.navigation.apply_draw_zoom(bounds, end, selection, zoom_out);
                        self.record_navigation(before);
                    }
                } else if !pan_mode && !self.primary_pan && response.clicked_by(PointerButton::Primary) {
                    // A click can be delivered as press and release in the same frame.
                    if let Some(end) = response.interact_pointer_pos() {
                        let before = self.navigation;
                        self.navigation.apply_draw_zoom(bounds, end, zoom_selection(end, end, bounds), zoom_out);
                        self.record_navigation(before);
                    }
                }
                if !primary_down {
                    self.zoom_start = None;
                }
            }

            let scale = fit * f64::from(self.navigation.zoom);
            let center = bounds.center() + self.navigation.pan;
            let size = Vec2::new((width * scale) as f32, (height * scale) as f32);
            let rect = Rect::from_center_size(center, size);
            let Ok(view) = geometry.viewport([rect.min.x.into(), rect.min.y.into()], scale) else { return };
            let painter = ui.painter().with_clip_rect(canvas);
            painter.image(texture, rect, uv, Color32::WHITE);
            if let Some(outline) = zoom_outline {
                painter.rect_stroke(
                    outline, 0.0, egui::Stroke::new(1.5_f32, ui.visuals().selection.stroke.color),
                    egui::StrokeKind::Inside,
                );
            }

            if canvas_pressed && (middle_down || (self.primary_pan && primary_down)) {
                context.set_cursor_icon(CursorIcon::Grabbing);
            } else if let Some(pointer) = response.hover_pos() {
                if pan_mode {
                    context.set_cursor_icon(CursorIcon::Grab);
                } else if self.tool == Tool::Zoom {
                    context.set_cursor_icon(if ui.input(|input| input.modifiers.command) {
                        CursorIcon::ZoomOut
                    } else {
                        CursorIcon::ZoomIn
                    });
                } else if rect.contains(pointer) {
                    context.set_cursor_icon(CursorIcon::Crosshair);
                }
            }

            let markup_painter = painter.with_clip_rect(rect.intersect(canvas));
            let stroke = egui::Stroke::new(2.0_f32, Color32::from_rgb(227, 68, 49));
            for &(page, points) in &self.rectangles {
                if page == self.page_index {
                    paint_rectangle(&markup_painter, view, points, stroke);
                }
            }
            if self.tool == Tool::Rectangle && !self.primary_pan {
                if let Some(pointer) = ui.input(|input| input.pointer.latest_pos()) {
                    let end = Pos2::new(
                        pointer.x.clamp(rect.min.x, rect.max.x),
                        pointer.y.clamp(rect.min.y, rect.max.y),
                    );
                    if let Ok(end) = view.canvas_to_page([end.x.into(), end.y.into()]) {
                        if primary_pressed && canvas_pressed && !pan_mode && rect.contains(pointer) {
                            self.rectangle_start = Some(end);
                            self.selected = None;
                        }
                        if let Some(start) = self.rectangle_start {
                            if primary_down {
                                paint_rectangle(&markup_painter, view, [start, end], stroke);
                            } else if response.drag_stopped_by(PointerButton::Primary)
                                && start[0] != end[0] && start[1] != end[1]
                            {
                                self.rectangles.push((self.page_index, [start, end]));
                                self.undone_rectangles.clear();
                            }
                        }
                    }
                }
            }
            if !primary_down || self.tool != Tool::Rectangle {
                self.rectangle_start = None;
            }

            if self.tool == Tool::Inspect && !pan_mode && !self.primary_pan
                && response.clicked_by(PointerButton::Primary)
            {
                if let Some(position) = response.interact_pointer_pos().filter(|p| rect.contains(*p)) {
                    self.selected = view.canvas_to_page([position.x.into(), position.y.into()]).ok();
                }
            }
            if !primary_down {
                self.primary_pan = false;
            }
            if let Some(point) = self.selected {
                if let Ok([x, y]) = view.page_to_canvas(point) {
                    painter.circle_filled(Pos2::new(x as f32, y as f32), 4.0, Color32::from_rgb(28, 106, 235));
                }
            }
        });
        if let Some(menu) = &self.native_menu {
            menu.update_enabled(|command| self.command_enabled(command));
        }
    }
}

impl eframe::App for Viewer {
    fn update(&mut self, context: &egui::Context, _: &mut eframe::Frame) {
        self.draw(context);
    }
}

fn paint_rectangle(
    painter: &egui::Painter,
    view: simplypdf_core::geometry::Viewport,
    [start, end]: [Point; 2],
    stroke: egui::Stroke,
) {
    if let (Ok([x0, y0]), Ok([x1, y1])) = (view.page_to_canvas(start), view.page_to_canvas(end)) {
        let rect = Rect::from_two_pos(Pos2::new(x0 as f32, y0 as f32), Pos2::new(x1 as f32, y1 as f32));
        painter.rect_stroke(rect, 0.0, stroke, egui::StrokeKind::Inside);
    }
}

fn canvas_point(point: Pos2, bounds: Rect) -> [i32; 2] {
    [(point.x - bounds.min.x).round() as i32, (point.y - bounds.min.y).round() as i32]
}

fn zoom_selection(start: Pos2, end: Pos2, bounds: Rect) -> ZoomRect {
    let movement = DragMove {
        mode: Mode::Zoom,
        shift: false,
        outline_visible: false,
        begin: canvas_point(start, bounds),
        end: canvas_point(end, bounds),
        work_size: [0, 0],
        page_size: [0, 0],
    };
    let DragEffect::Outline(selection) = movement.effect() else {
        unreachable!("a zoom drag produces an outline");
    };
    selection
}

/// Zoom relative to fit and pan in logical canvas units.
#[derive(Clone, Copy, PartialEq)]
struct Navigation {
    zoom: f32,
    pan: Vec2,
}

impl Default for Navigation {
    fn default() -> Self {
        Self { zoom: 1.0, pan: Vec2::ZERO }
    }
}

impl Navigation {
    fn apply_draw_zoom(&mut self, bounds: Rect, pointer: Pos2, selection: ZoomRect, zoom_out: bool) {
        let output_size = [bounds.width().round() as u32, bounds.height().round() as u32];
        let release = Release {
            mode: Mode::Zoom,
            shift: false,
            modifier_one: zoom_out,
            point: canvas_point(pointer, bounds),
            output_size,
            selection,
            selection_pixels: selection.size,
            drag_threshold_pixels: 3,
        };
        let Some(rect) = release.zoom_rect() else { return };
        let Some(window) = ZoomWindow::new(output_size, (self.zoom * 100.0).round() as u16, 50) else {
            return;
        };
        let zoom = f32::from(window.zoom_for_rect(rect.size)) / 100.0;
        let center = Vec2::new(
            rect.origin[0] as f32 + rect.size[0] as f32 * 0.5,
            rect.origin[1] as f32 + rect.size[1] as f32 * 0.5,
        );
        let anchor = bounds.min + center - bounds.center();
        self.zoom_at(zoom / self.zoom, anchor);
        self.pan -= anchor;
    }

    /// Keep the point at the anchor stationary as the drawing changes size.
    /// The anchor is relative to the fitted canvas center.
    fn zoom_at(&mut self, factor: f32, anchor: Vec2) {
        let zoom = (self.zoom * factor).clamp(0.5, 8.0);
        self.pan = anchor + (self.pan - anchor) * (zoom / self.zoom);
        self.zoom = zoom;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/page-geometry.pdf")
    }

    fn frame(viewer: &mut Viewer, context: &egui::Context, input: egui::RawInput) -> CursorIcon {
        context.run(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(900.0, 650.0))),
                ..input
            },
            |ctx| viewer.draw(ctx),
        ).platform_output.cursor_icon
    }

    fn pointer_button(pos: Pos2, button: PointerButton, pressed: bool) -> egui::Event {
        egui::Event::PointerButton { pos, button, pressed, modifiers: Modifiers::NONE }
    }

    #[test]
    fn canvas_drag_preserves_selection_with_middle_button_and_pan_tool() {
        for (button, pan_tool) in [(PointerButton::Middle, false), (PointerButton::Primary, true)] {
            let context = egui::Context::default();
            let mut viewer = Viewer::new(&context, fixture());
            viewer.selected = Some([150.0, 120.0]);
            viewer.tool = if pan_tool { Tool::Pan } else { Tool::Inspect };
            frame(&mut viewer, &context, egui::RawInput::default());

            let start = Pos2::new(450.0, 300.0);
            let delta = Vec2::new(40.0, 25.0);
            let end = start + delta;
            for events in [
                vec![egui::Event::PointerMoved(start), pointer_button(start, button, true)],
                vec![egui::Event::PointerMoved(end)],
                vec![pointer_button(end, button, false)],
            ] {
                frame(&mut viewer, &context, egui::RawInput { events, ..Default::default() });
            }

            assert_eq!(viewer.navigation.pan, delta);
            assert_eq!(viewer.selected, Some([150.0, 120.0]));
        }
    }

    #[test]
    fn canvas_cursor_follows_inspect_pan_and_temporary_pan() {
        let context = egui::Context::default();
        let mut viewer = Viewer::new(&context, fixture());
        frame(&mut viewer, &context, egui::RawInput::default());
        let pos = Pos2::new(450.0, 300.0);
        let mut send = |events| {
            frame(&mut viewer, &context, egui::RawInput { events, ..Default::default() })
        };
        let key = |key, pressed, modifiers| egui::Event::Key {
            key, physical_key: None, pressed, repeat: false, modifiers,
        };

        assert_eq!(send(vec![egui::Event::PointerMoved(pos)]), CursorIcon::Crosshair);
        assert_eq!(send(vec![key(Key::V, true, Modifiers::SHIFT)]), CursorIcon::Grab);
        assert_eq!(send(vec![pointer_button(pos, PointerButton::Primary, true)]), CursorIcon::Grabbing);
        assert_eq!(send(vec![pointer_button(pos, PointerButton::Primary, false)]), CursorIcon::Grab);
        assert_eq!(send(vec![key(Key::V, true, Modifiers::NONE)]), CursorIcon::Crosshair);
        assert_eq!(send(vec![pointer_button(pos, PointerButton::Middle, true)]), CursorIcon::Grabbing);
        assert_eq!(send(vec![pointer_button(pos, PointerButton::Middle, false)]), CursorIcon::Crosshair);

        assert_eq!(send(vec![key(Key::Space, true, Modifiers::NONE)]), CursorIcon::Grab);
        assert_eq!(send(vec![pointer_button(pos, PointerButton::Primary, true)]), CursorIcon::Grabbing);
        assert_eq!(send(vec![key(Key::Space, false, Modifiers::NONE)]), CursorIcon::Grabbing);
        assert_eq!(send(vec![pointer_button(pos, PointerButton::Primary, false)]), CursorIcon::Crosshair);
        assert!(viewer.selected.is_none());
    }

    #[test]
    fn zoom_tool_uses_draw_click_drag_and_modifier_zoom() {
        let context = egui::Context::default();
        let mut viewer = Viewer::new(&context, fixture());
        viewer.tool = Tool::Zoom;
        let start = Pos2::new(450.0, 300.0);
        let send = |viewer: &mut Viewer, events, modifiers| {
            frame(viewer, &context, egui::RawInput { events, modifiers, ..Default::default() })
        };
        let button = |point, pressed, modifiers| egui::Event::PointerButton {
            pos: point, button: PointerButton::Primary, pressed, modifiers,
        };
        frame(&mut viewer, &context, egui::RawInput::default());
        assert_eq!(
            send(&mut viewer, vec![egui::Event::PointerMoved(start)], Modifiers::NONE),
            CursorIcon::ZoomIn,
        );
        for pressed in [true, false] {
            send(&mut viewer, vec![button(start, pressed, Modifiers::NONE)], Modifiers::NONE);
        }
        assert!((viewer.navigation.zoom - 2.0).abs() < 0.02);
        assert!(viewer.selected.is_none());
        viewer.apply_navigation(Command::ZoomPrevious);
        assert_eq!(viewer.navigation.zoom, 1.0);

        let end = start + Vec2::new(140.0, 100.0);
        send(&mut viewer, vec![button(start, true, Modifiers::NONE)], Modifiers::NONE);
        send(&mut viewer, vec![egui::Event::PointerMoved(end)], Modifiers::NONE);
        send(&mut viewer, vec![button(end, false, Modifiers::NONE)], Modifiers::NONE);
        let after_drag = viewer.navigation.zoom;
        assert!(after_drag > 3.0);

        assert_eq!(send(&mut viewer, vec![], Modifiers::COMMAND), CursorIcon::ZoomOut);
        for pressed in [true, false] {
            send(&mut viewer, vec![button(end, pressed, Modifiers::COMMAND)], Modifiers::COMMAND);
        }
        assert!((viewer.navigation.zoom - after_drag / 2.0).abs() < 0.02);
    }

    #[test]
    fn rectangle_tool_tracks_page_coordinates_and_supports_undo_and_redo() {
        let context = egui::Context::default();
        let mut viewer = Viewer::new(&context, fixture());
        viewer.execute(&context, Command::Rectangle);
        frame(&mut viewer, &context, egui::RawInput::default());
        let start = Pos2::new(450.0, 280.0);
        let end = Pos2::new(550.0, 370.0);
        for events in [
            vec![egui::Event::PointerMoved(start), pointer_button(start, PointerButton::Primary, true)],
            vec![egui::Event::PointerMoved(end)],
            vec![pointer_button(end, PointerButton::Primary, false)],
        ] {
            frame(&mut viewer, &context, egui::RawInput { events, ..Default::default() });
        }
        assert_eq!(viewer.rectangles.len(), 1);
        let rectangle = viewer.rectangles[0];
        assert_eq!(rectangle.0, 0);
        assert!(rectangle.1[0][0] < rectangle.1[1][0]);
        assert!(rectangle.1[0][1] > rectangle.1[1][1]);

        viewer.execute(&context, Command::NextPage);
        assert_eq!(viewer.page_index, 1);
        assert_eq!(viewer.rectangles, vec![rectangle]);
        viewer.execute(&context, Command::Undo);
        assert!(viewer.rectangles.is_empty());
        viewer.execute(&context, Command::Redo);
        assert_eq!(viewer.rectangles, vec![rectangle]);
        viewer.open(&context, &fixture());
        assert!(!viewer.command_enabled(Command::Undo));
        assert!(!viewer.command_enabled(Command::Redo));
    }

    #[test]
    fn keyboard_navigation_and_file_drop_share_the_document_open_path() {
        let context = egui::Context::default();
        let mut viewer = Viewer::new(&context, fixture());
        frame(
            &mut viewer,
            &context,
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key: Key::PageDown,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::NONE,
                }],
                ..Default::default()
            },
        );
        assert_eq!(viewer.page_index, 1);

        let drop = |path| egui::RawInput {
            dropped_files: vec![egui::DroppedFile { path: Some(path), ..Default::default() }],
            ..Default::default()
        };
        frame(&mut viewer, &context, drop(fixture().with_file_name("missing.pdf")));
        assert_eq!(viewer.page_index, 1);
        assert!(viewer.image.is_some());
        assert!(viewer.error.is_some());

        frame(&mut viewer, &context, drop(fixture()));
        assert_eq!(viewer.page_index, 0);
        assert!(viewer.error.is_none());
    }

    #[test]
    fn zoom_commands_restore_views_and_reset_history_on_page_change() {
        let context = egui::Context::default();
        let mut viewer = Viewer::new(&context, fixture());
        viewer.selected = Some([150.0, 120.0]);
        assert!(!viewer.command_enabled(Command::ZoomPrevious));
        viewer.apply_navigation(Command::ZoomIn);
        viewer.apply_navigation(Command::FitPage);
        viewer.apply_navigation(Command::ZoomPrevious);
        assert_eq!(viewer.navigation.zoom, 1.25);
        assert!(viewer.command_enabled(Command::ZoomNext));
        viewer.apply_navigation(Command::ZoomNext);
        assert_eq!(viewer.navigation.zoom, 1.0);
        assert_eq!(viewer.selected, Some([150.0, 120.0]));

        viewer.show_page(&context, 1);
        assert!(!viewer.command_enabled(Command::ZoomPrevious));
        assert!(!viewer.command_enabled(Command::ZoomNext));
    }

    #[test]
    fn zoom_preserves_anchor_after_pan_and_at_zoom_limits() {
        let mut nav = Navigation { zoom: 1.0, pan: Vec2::new(20.0, -10.0) };
        let anchor = Vec2::new(100.0, 60.0);
        let point = (anchor - nav.pan) / nav.zoom;
        for factor in [2.0, 20.0, 0.001] {
            nav.zoom_at(factor, anchor);
            assert!((nav.pan + point * nav.zoom - anchor).length() < 0.0001);
        }
        assert_eq!(nav.zoom, 0.5);
    }
}
