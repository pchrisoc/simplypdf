//! Shared preset controls for new routes and inline takeoff edits.
use crate::icons;
use simplypdf_engine::measure::hvac::{self, Takeoff};

pub fn color(hvac: &Takeoff) -> egui::Color32 {
    let [r, g, b] = hvac.color();
    egui::Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

pub fn fields(ui: &mut egui::Ui, value: &mut Takeoff) {
    ui.label(tl!("Profile"));
    let previous = value.profile.clone();
    craft_ui::ComboBox::from_id_salt("hvac-profile").selected_text(&value.profile).width(ui.available_width()).show_ui(ui, |ui| {
        for name in ["pipe", "round duct", "rectangular duct"] {
            ui.selectable_value(&mut value.profile, name.into(), name);
        }
    });
    if previous != value.profile {
        let profile = value.profile.clone();
        *value = if profile == "pipe" { Takeoff::pipe() } else { Takeoff::duct() };
        value.profile = profile;
        if value.profile == "rectangular duct" {
            value.size = "4 x 4 in".into();
        }
    }
    if value.profile == "rectangular duct" {
        let (width, height) = value.size.split_once('x').map_or(("4", "4"), |(w, h)| (w.trim(), h.trim().trim_end_matches(" in")));
        let mut width = format!("{width} in");
        let mut height = format!("{height} in");
        ui.label(tl!("Width"));
        size(ui, "duct-width", &mut width, hvac::ROUND_DUCT_SIZES, false);
        ui.label(tl!("Height"));
        size(ui, "duct-height", &mut height, hvac::ROUND_DUCT_SIZES, false);
        value.size = format!("{} x {} in", width.trim_end_matches(" in"), height.trim_end_matches(" in"));
    } else {
        ui.label(tl!("Diameter"));
        let pipe = value.profile == "pipe";
        size(ui, "route-size", &mut value.size, if pipe { hvac::PIPE_SIZES } else { hvac::ROUND_DUCT_SIZES }, pipe);
    }
    ui.add_space(6.0);
    ui.colored_label(color(value), tl!("Material"));
    choice(ui, "hvac-material", &mut value.material, if value.profile == "pipe" { hvac::PIPE_MATERIALS } else { hvac::DUCT_MATERIALS });
    ui.add_space(6.0);
    ui.label(tl!("System"));
    choice(ui, "hvac-system", &mut value.service, if value.profile == "pipe" { hvac::PIPE_SYSTEMS } else { hvac::DUCT_SYSTEMS });
    ui.add_space(6.0);
}

fn choice(ui: &mut egui::Ui, id: &str, value: &mut String, choices: &[&str]) {
    craft_ui::ComboBox::from_id_salt(id).selected_text(value.as_str()).width(ui.available_width()).show_ui(ui, |ui| {
        for choice in choices {
            ui.selectable_value(value, (*choice).into(), *choice);
        }
    });
}

/// Adjacent size buttons permit changing size in one click, without opening a menu.
pub fn size(ui: &mut egui::Ui, id: &str, value: &mut String, sizes: &[&str], descending: bool) {
    ui.push_id(id, |ui| {
        ui.horizontal(|ui| {
            let index = sizes.iter().position(|size| *size == value).unwrap_or(0);
            let smaller = if descending { index.checked_add(1).filter(|n| *n < sizes.len()) } else { index.checked_sub(1) };
            let larger = if descending { index.checked_sub(1) } else { index.checked_add(1).filter(|n| *n < sizes.len()) };
            if ui.add_enabled_ui(smaller.is_some(), |ui| icons::button(ui, "minus", 28.0, false, tl!("Smaller size"))).inner.clicked()
                && let Some(size) = smaller.and_then(|n| sizes.get(n))
            {
                *value = (*size).into();
            }
            craft_ui::ComboBox::from_id_salt("size").selected_text(value.as_str()).width((ui.available_width() - 38.0).max(45.0)).show_ui(ui, |ui| {
                for size in sizes {
                    ui.selectable_value(value, (*size).into(), *size);
                }
            });
            if ui.add_enabled_ui(larger.is_some(), |ui| icons::button(ui, "plus", 28.0, false, tl!("Larger size"))).inner.clicked()
                && let Some(size) = larger.and_then(|n| sizes.get(n))
            {
                *value = (*size).into();
            }
        })
    });
}
