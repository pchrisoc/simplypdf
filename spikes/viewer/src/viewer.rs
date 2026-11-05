//! Desktop shell. PDF engine objects stay inside the adapter.

use std::path::PathBuf;

use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, TextureHandle, Vec2};
use simplypdf_core::geometry::{PageGeometry, Point};

use crate::pdf::Document;

pub struct Viewer {
    title: String,
    document: Result<Document, String>,
    page_index: usize,
    image: Option<(TextureHandle, PageGeometry, Rect)>,
    selected: Option<Point>,
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
            error: None,
        };
        viewer.show_page(context, 0);
        viewer
    }

    fn show_page(&mut self, context: &egui::Context, index: usize) {
        self.page_index = index;
        self.selected = None;
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
                    format!("Page {} of {count} · Fit page", self.page_index + 1)
                });
                ui.separator();
                if let Some([x, y]) = self.selected {
                    ui.monospace(format!("Page coordinates  x {x:.2} pt   y {y:.2} pt"));
                } else {
                    ui.label("Click the drawing to inspect page coordinates");
                }
            });
        });
        egui::CentralPanel::default().show(context, |ui| {
            let Some((texture, geometry, uv)) = &self.image else {
                ui.label(self.error.as_deref().unwrap_or("No page available"));
                return;
            };
            let bounds = ui.available_rect_before_wrap().shrink(20.0);
            let Ok(base) = geometry.viewport([0.0, 0.0], 1.0) else { return };
            let [width, height] = base.size();
            let scale = (f64::from(bounds.width()) / width).min(f64::from(bounds.height()) / height);
            let rect = Rect::from_center_size(bounds.center(), Vec2::new((width * scale) as f32, (height * scale) as f32));
            let Ok(view) = geometry.viewport([rect.min.x.into(), rect.min.y.into()], scale) else { return };
            let response = ui.put(rect, egui::Image::new((texture.id(), rect.size())).uv(*uv).sense(egui::Sense::click()));
            if response.clicked() {
                if let Some(position) = response.interact_pointer_pos() {
                    self.selected = view.canvas_to_page([position.x.into(), position.y.into()]).ok();
                }
            }
            if let Some(point) = self.selected {
                if let Ok([x, y]) = view.page_to_canvas(point) {
                    ui.painter().circle_filled(Pos2::new(x as f32, y as f32), 4.0, Color32::from_rgb(28, 106, 235));
                }
            }
        });
    }
}
