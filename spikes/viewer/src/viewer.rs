//! Desktop shell. PDF engine objects stay inside the adapter.

use std::path::PathBuf;

use eframe::egui::{self, Color32, ColorImage, PointerButton, Pos2, Rect, TextureHandle, Vec2};
use simplypdf_core::geometry::{PageGeometry, Point};

use crate::pdf::Document;

pub struct Viewer {
    title: String,
    document: Result<Document, String>,
    page_index: usize,
    image: Option<(TextureHandle, PageGeometry, Rect)>,
    selected: Option<Point>,
    navigation: Navigation,
    pan_tool: bool,
    error: Option<String>,
}

impl Viewer {
    pub fn new(context: &egui::Context, path: PathBuf) -> Self {
        let mut viewer = Self {
            title: path.file_name().unwrap_or_default().to_string_lossy().into_owned(),
            document: Document::open(&path),
            page_index: 0,
            image: None,
            selected: None,
            navigation: Navigation::default(),
            pan_tool: false,
            error: None,
        };
        viewer.show_page(context, 0);
        viewer
    }

    fn show_page(&mut self, context: &egui::Context, index: usize) {
        self.page_index = index;
        self.selected = None;
        self.navigation = Navigation::default();
        self.image = None;
        self.error = None;
        let result = self.document.as_ref().map_err(Clone::clone).and_then(|doc| doc.render(index));
        match result {
            Ok(page) => {
                let image = ColorImage::from_rgba_unmultiplied(page.size, &page.rgba);
                let texture = context.load_texture("PDF page", image, egui::TextureOptions::LINEAR);
                let uv = Rect::from_min_max(Pos2::ZERO, Pos2::from(page.uv_max));
                self.image = Some((texture, page.geometry, uv));
            }
            Err(error) => self.error = Some(error),
        }
    }
}

impl eframe::App for Viewer {
    fn update(&mut self, context: &egui::Context, _: &mut eframe::Frame) {
        let count = self.document.as_ref().map_or(0, Document::page_count);
        egui::TopBottomPanel::top("document").show(context, |ui| {
            ui.horizontal(|ui| {
                ui.heading("SimplyPDF");
                ui.separator();
                ui.label(&self.title);
                ui.separator();
                ui.add_enabled_ui(self.image.is_some(), |ui| {
                    if ui.button("−").on_hover_text("Zoom out").clicked() {
                        self.navigation.zoom_at(1.0 / 1.25, Vec2::ZERO);
                    }
                    if ui.button("+").on_hover_text("Zoom in").clicked() {
                        self.navigation.zoom_at(1.25, Vec2::ZERO);
                    }
                    if ui.button("Fit page").clicked() {
                        self.navigation = Navigation::default();
                    }
                    ui.toggle_value(&mut self.pan_tool, "Pan");
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::widgets::global_theme_preference_switch(ui);
                });
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
        egui::TopBottomPanel::bottom("status").show(context, |ui| {
            ui.horizontal(|ui| {
                ui.label(if count == 0 { "No document".into() } else {
                    format!("Page {} of {count} · {:.0}% of fit", self.page_index + 1, self.navigation.zoom * 100.0)
                });
                ui.separator();
                if let Some([x, y]) = self.selected {
                    ui.monospace(format!("Page coordinates  x {x:.2} pt   y {y:.2} pt"));
                } else {
                    ui.label(if self.pan_tool {
                        "Drag to pan"
                    } else {
                        "Click to inspect · Scroll to zoom · Middle-drag to pan"
                    });
                }
            });
        });
        egui::CentralPanel::default().show(context, |ui| {
            let Some((texture, geometry, uv)) = &self.image else {
                ui.label(self.error.as_deref().unwrap_or("No page available"));
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
                self.navigation = Navigation::default();
            }
            if response.dragged_by(PointerButton::Middle)
                || (self.pan_tool && response.dragged_by(PointerButton::Primary))
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

            let scale = fit * f64::from(self.navigation.zoom);
            let center = bounds.center() + self.navigation.pan;
            let size = Vec2::new((width * scale) as f32, (height * scale) as f32);
            let rect = Rect::from_center_size(center, size);
            let Ok(view) = geometry.viewport([rect.min.x.into(), rect.min.y.into()], scale) else { return };
            let painter = ui.painter().with_clip_rect(canvas);
            painter.image(texture.id(), rect, *uv, Color32::WHITE);

            if !self.pan_tool && response.clicked_by(PointerButton::Primary) {
                if let Some(position) = response.interact_pointer_pos().filter(|p| rect.contains(*p)) {
                    self.selected = view.canvas_to_page([position.x.into(), position.y.into()]).ok();
                }
            }
            if let Some(point) = self.selected {
                if let Ok([x, y]) = view.page_to_canvas(point) {
                    painter.circle_filled(Pos2::new(x as f32, y as f32), 4.0, Color32::from_rgb(28, 106, 235));
                }
            }
        });
    }
}

/// Zoom relative to fit and pan in logical canvas units.
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
