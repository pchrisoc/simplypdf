//! Standalone Rust Draw port. This executable does not use the SimplyPDF viewer.

mod fodg;
#[cfg(target_os = "macos")]
mod save;

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, StrokeKind, Vec2};
use simplypdf_draw::document::{A4_PORTRAIT, Document, Shape, ShapeKind};
use simplypdf_draw::zoom_history::ZoomHistory;

struct DrawApp {
    document: Document,
    page: usize,
    layer: u8,
    tool: Option<ShapeKind>,
    start: Option<[u32; 2]>,
    undo: Vec<usize>,
    redo: Vec<(usize, Shape)>,
    status: String,
    zoom: u16,
    zoom_history: ZoomHistory<u16>,
}

impl Default for DrawApp {
    fn default() -> Self {
        let mut zoom_history = ZoomHistory::default();
        zoom_history.insert(100);
        Self {
            document: Document::new(A4_PORTRAIT),
            page: 0,
            layer: 0,
            tool: None,
            start: None,
            undo: Vec::new(),
            redo: Vec::new(),
            status: "A4 · Drawing".into(),
            zoom: 100,
            zoom_history,
        }
    }
}

impl DrawApp {
    fn select_page(&mut self, page: usize) {
        self.page = page;
        self.start = None;
    }

    fn append_page(&mut self) {
        if let Some(index) = self.document.add_page_after(self.document.pages().len() - 1) {
            self.select_page(index);
        }
    }

    fn insert(&mut self, shape: Shape) {
        if self.document.add_shape(self.page, shape) {
            self.undo.push(self.page);
            self.redo.clear();
        }
    }

    fn undo(&mut self) {
        if let Some(page) = self.undo.pop() {
            if let Some(shape) = self.document.pop_shape(page) {
                self.redo.push((page, shape));
                self.select_page(page);
            }
        }
    }

    fn redo(&mut self) {
        if let Some((page, shape)) = self.redo.pop() {
            if self.document.add_shape(page, shape) {
                self.undo.push(page);
                self.select_page(page);
            }
        }
    }

    fn set_zoom(&mut self, zoom: u16) {
        let zoom = zoom.clamp(25, 800);
        if self.zoom != zoom {
            self.zoom = zoom;
            self.zoom_history.insert(zoom);
        }
    }

    fn draw(&mut self, context: &egui::Context) {
        let mut redo = egui::Modifiers::COMMAND;
        redo.shift = true;
        if context.input_mut(|input| input.consume_key(redo, egui::Key::Z)) {
            self.redo();
        } else if context.input_mut(|input| input.consume_key(egui::Modifiers::COMMAND, egui::Key::Z)) {
            self.undo();
        }

        egui::TopBottomPanel::top("menu").show(context, |ui| {
            egui::menu::bar(ui, |ui| {
                #[cfg(target_os = "macos")]
                ui.menu_button("File", |ui| {
                    if ui.button("Export Flat OpenDocument Drawing…").clicked() {
                        ui.close_menu();
                        match save::choose_path() {
                            Ok(Some(path)) => {
                                self.status = match save::export(&self.document, &path) {
                                    Ok(()) => format!("Exported {}", path.display()),
                                    Err(error) => format!("Export failed: {error}"),
                                };
                            }
                            Ok(None) => {}
                            Err(error) => self.status = error.into(),
                        }
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.add_enabled(!self.undo.is_empty(), egui::Button::new("Undo Shape")).clicked() {
                        self.undo();
                        ui.close_menu();
                    }
                    if ui.add_enabled(!self.redo.is_empty(), egui::Button::new("Redo Shape")).clicked() {
                        self.redo();
                        ui.close_menu();
                    }
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Fit Page").clicked() {
                        self.set_zoom(100);
                        ui.close_menu();
                    }
                    if ui.add_enabled(self.zoom_history.can_step_back(), egui::Button::new("Zoom Previous")).clicked() {
                        if let Some(&zoom) = self.zoom_history.step_back() {
                            self.zoom = zoom;
                        }
                        ui.close_menu();
                    }
                    if ui.add_enabled(self.zoom_history.can_step_forward(), egui::Button::new("Zoom Next")).clicked() {
                        if let Some(&zoom) = self.zoom_history.step_forward() {
                            self.zoom = zoom;
                        }
                        ui.close_menu();
                    }
                });
                ui.menu_button("Insert", |ui| {
                    for (label, kind) in [("Rectangle", ShapeKind::Rectangle), ("Ellipse", ShapeKind::Ellipse)] {
                        if ui.button(label).clicked() {
                            self.tool = Some(kind);
                            ui.close_menu();
                        }
                    }
                });
                ui.menu_button("Page", |ui| {
                    if ui.button("Append Page").clicked() {
                        self.append_page();
                        ui.close_menu();
                    }
                });
            });
        });

        egui::TopBottomPanel::top("drawing").show(context, |ui| {
            ui.horizontal(|ui| {
                ui.strong("Drawing");
                ui.separator();
                ui.selectable_value(&mut self.tool, None, "Pointer");
                ui.selectable_value(&mut self.tool, Some(ShapeKind::Rectangle), "Rectangle");
                ui.selectable_value(&mut self.tool, Some(ShapeKind::Ellipse), "Ellipse");
                ui.separator();
                if ui.button("Append Page").clicked() {
                    self.append_page();
                }
                ui.separator();
                if ui.button("−").on_hover_text("Zoom out").clicked() {
                    self.set_zoom(self.zoom * 4 / 5);
                }
                ui.label(format!("{}% of fit", self.zoom));
                if ui.button("+").on_hover_text("Zoom in").clicked() {
                    self.set_zoom(self.zoom * 5 / 4);
                }
            });
        });
        egui::SidePanel::left("pages").default_width(152.0).show(context, |ui| {
            ui.heading("Pages");
            egui::ScrollArea::vertical().show(ui, |ui| {
                for page in 0..self.document.pages().len() {
                    if ui.selectable_label(self.page == page, format!("Page {}", page + 1)).clicked() {
                        self.select_page(page);
                    }
                }
            });
        });
        egui::TopBottomPanel::bottom("layers").show(context, |ui| {
            ui.horizontal(|ui| {
                for (id, label) in [(0, "Layout"), (2, "Background Objects"), (3, "Controls"), (4, "Dimension Lines")] {
                    ui.selectable_value(&mut self.layer, id, label);
                }
                ui.separator();
                ui.label(&self.status);
            });
        });

        egui::CentralPanel::default().show(context, |ui| {
            let (canvas, response) = ui.allocate_exact_size(ui.available_size(), egui::Sense::drag());
            let available = canvas.shrink(20.0);
            if !available.is_positive() {
                return;
            }
            let page = &self.document.pages()[self.page];
            let size = Vec2::new(page.size[0] as f32, page.size[1] as f32);
            let scale = (available.width() / size.x).min(available.height() / size.y)
                * f32::from(self.zoom) / 100.0;
            let paper = Rect::from_center_size(available.center(), size * scale);
            let painter = ui.painter().with_clip_rect(paper);
            painter.rect_filled(paper, 0.0, Color32::WHITE);
            let [left, top, right, bottom] = page.margins.map(|margin| margin as f32 * scale);
            painter.rect_stroke(
                Rect::from_min_max(paper.min + Vec2::new(left, top), paper.max - Vec2::new(right, bottom)),
                0.0,
                Stroke::new(1.0_f32, Color32::LIGHT_GRAY),
                StrokeKind::Inside,
            );
            for &shape in &page.shapes {
                paint_shape(&painter, paper.min, scale, shape);
            }

            if let Some(kind) = self.tool {
                if response.hover_pos().is_some_and(|pointer| paper.contains(pointer)) {
                    context.set_cursor_icon(egui::CursorIcon::Crosshair);
                }
                if response.drag_started_by(egui::PointerButton::Primary) {
                    self.start = ui.input(|input| input.pointer.press_origin())
                        .filter(|point| paper.contains(*point))
                        .map(|point| paper_point(point, paper, scale, page.size));
                }
                if let (Some(start), Some(pointer)) = (self.start, response.interact_pointer_pos()) {
                    let end = paper_point(pointer, paper, scale, page.size);
                    if let Some(shape) = Shape::new(kind, self.layer, start, end) {
                        if response.drag_stopped_by(egui::PointerButton::Primary) {
                            paint_shape(&painter, paper.min, scale, shape);
                            self.insert(shape);
                        } else if response.dragged_by(egui::PointerButton::Primary) {
                            paint_shape(&painter, paper.min, scale, shape);
                        }
                    }
                }
            }
            if !ui.input(|input| input.pointer.primary_down()) || self.tool.is_none() {
                self.start = None;
            }
        });
    }
}

fn paper_point(point: Pos2, paper: Rect, scale: f32, size: [u32; 2]) -> [u32; 2] {
    let relative = (point - paper.min) / scale;
    [
        (relative.x.round() as u32).min(size[0]),
        (relative.y.round() as u32).min(size[1]),
    ]
}

fn paint_shape(painter: &egui::Painter, origin: Pos2, scale: f32, shape: Shape) {
    let position = |[x, y]: [u32; 2]| origin + Vec2::new(x as f32, y as f32) * scale;
    let bounds = Rect::from_min_max(position(shape.min), position(shape.max));
    let fill = Color32::from_rgb(191, 220, 247);
    let stroke = Stroke::new(1.5_f32, Color32::from_rgb(30, 79, 125));
    match shape.kind {
        ShapeKind::Rectangle => {
            painter.rect(bounds, 0.0, fill, stroke, StrokeKind::Inside);
        }
        ShapeKind::Ellipse => {
            painter.add(egui::epaint::EllipseShape {
                center: bounds.center(),
                radius: bounds.size() / 2.0,
                fill,
                stroke,
            });
        }
    }
}

impl eframe::App for DrawApp {
    fn update(&mut self, context: &egui::Context, _: &mut eframe::Frame) {
        self.draw(context);
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Draw Rust",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1060.0, 760.0])
                .with_min_inner_size([640.0, 440.0]),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(DrawApp::default()))),
    )
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canvas_drag_inserts_a_shape_and_history_restores_it_across_pages() {
        let context = egui::Context::default();
        let mut app = DrawApp { tool: Some(ShapeKind::Ellipse), ..Default::default() };
        let start = Pos2::new(460.0, 240.0);
        let end = Pos2::new(560.0, 345.0);
        let button = |pos, pressed| egui::Event::PointerButton {
            pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE,
        };
        for events in [
            vec![],
            vec![egui::Event::PointerMoved(start), button(start, true)],
            vec![egui::Event::PointerMoved(end)],
            vec![button(end, false)],
        ] {
            let _ = context.run(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(900.0, 700.0))),
                    events,
                    ..Default::default()
                },
                |ctx| app.draw(ctx),
            );
        }
        let [shape] = app.document.pages()[0].shapes.as_slice() else {
            panic!("one ellipse should be placed by the drag");
        };
        assert_eq!(shape.kind, ShapeKind::Ellipse);
        let shape = *shape;
        app.append_page();
        assert_eq!(app.page, 1);
        app.undo();
        assert_eq!(app.page, 0);
        assert!(app.document.pages()[0].shapes.is_empty());
        app.redo();
        assert_eq!(app.document.pages()[0].shapes, [shape]);
    }
}
