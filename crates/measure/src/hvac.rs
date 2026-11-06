//! Nominal HVAC specifications attached to measured centerline routes.

use serde::{Deserialize, Serialize};

pub const PIPE_SIZES: &[&str] = &["3 in", "2.5 in", "2 in", "1.75 in", "1.5 in", "1.25 in", "1 in", "0.75 in", "0.5 in", "0.25 in"];
pub const ROUND_DUCT_SIZES: &[&str] = &[
    "4 in", "6 in", "8 in", "10 in", "12 in", "14 in", "16 in", "18 in", "20 in", "22 in", "24 in", "26 in", "28 in", "30 in", "32 in", "34 in",
    "36 in", "38 in", "40 in", "42 in", "44 in", "46 in", "48 in",
];
pub const RECTANGULAR_DUCT_SIZES: &[&str] = ROUND_DUCT_SIZES;
pub const PIPE_SYSTEMS: &[&str] = &["HHW Supply", "HHW Return", "CHW Supply", "CHW Return"];
pub const DUCT_SYSTEMS: &[&str] = &["SA", "R", "EA"];
pub const PIPE_MATERIALS: &[&str] = &["Copper"];
pub const DUCT_MATERIALS: &[&str] = &["Galvanized", "Stainless steel", "Aluminum"];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Takeoff {
    pub profile: String,
    pub material: String,
    pub size: String,
    pub service: String,
}

impl Takeoff {
    pub fn pipe() -> Self {
        Self { profile: "pipe".into(), material: "Copper".into(), size: "3 in".into(), service: "HHW Supply".into() }
    }

    pub fn duct() -> Self {
        Self { profile: "round duct".into(), material: "Galvanized".into(), size: "4 in".into(), service: "SA".into() }
    }

    pub fn materials(&self) -> &'static [&'static str] {
        if self.profile == "pipe" { PIPE_MATERIALS } else { DUCT_MATERIALS }
    }
    pub fn systems(&self) -> &'static [&'static str] {
        if self.profile == "pipe" { PIPE_SYSTEMS } else { DUCT_SYSTEMS }
    }
    pub fn sizes(&self) -> &'static [&'static str] {
        if self.profile == "pipe" { PIPE_SIZES } else { ROUND_DUCT_SIZES }
    }
    /// Material colors are identical in the drawing, table and preset controls.
    pub fn color(&self) -> [f64; 3] {
        match self.material.as_str() {
            "Copper" => [0.82, 0.36, 0.13],
            "Stainless steel" => [0.58, 0.36, 0.85],
            "Aluminum" => [0.10, 0.68, 0.48],
            _ => [0.0, 0.47, 0.84],
        }
    }

    pub fn validate(&self) -> crate::Result<()> {
        if !["pipe", "round duct", "rectangular duct"].contains(&self.profile.as_str()) {
            return Err(crate::invalid("HVAC profile must be pipe, round duct or rectangular duct"));
        }
        for value in [&self.material, &self.size, &self.service] {
            if value.trim().is_empty() || value.len() > 64 || value.chars().any(char::is_control) {
                return Err(crate::invalid("HVAC material, size and service must each contain 1 to 64 printable bytes"));
            }
        }
        if !self.materials().contains(&self.material.as_str()) {
            return Err(crate::invalid("choose Copper for pipe, or Galvanized, Stainless steel or Aluminum for sheet metal"));
        }
        if !self.systems().contains(&self.service.as_str()) {
            return Err(crate::invalid("pipe system must be HHW Supply, HHW Return, CHW Supply or CHW Return; duct system must be SA, R or EA"));
        }
        let valid_size = if self.profile == "rectangular duct" {
            self.size.split_once('x').is_some_and(|(width, height)| {
                let width = width.trim().strip_suffix(" in").unwrap_or(width.trim());
                let height = height.trim().strip_suffix(" in").unwrap_or(height.trim());
                let width = format!("{width} in");
                let height = format!("{height} in");
                ROUND_DUCT_SIZES.contains(&width.as_str()) && ROUND_DUCT_SIZES.contains(&height.as_str())
            })
        } else {
            self.sizes().contains(&self.size.as_str())
        };
        if !valid_size {
            return Err(crate::invalid(
                "choose a standard pipe diameter (3, 2.5, or 2 down to 0.25 in in quarter-inch steps), or duct dimensions from 4 to 48 in in two-inch steps",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hvac_presets_validate_and_have_distinct_material_colors() {
        for mut value in [Takeoff::pipe(), Takeoff::duct()] {
            for material in value.materials() {
                value.material = (*material).into();
                for service in value.systems() {
                    value.service = (*service).into();
                    for size in value.sizes() {
                        value.size = (*size).into();
                        value.validate().unwrap();
                    }
                }
            }
        }
        for width in ROUND_DUCT_SIZES {
            for height in ROUND_DUCT_SIZES {
                let mut value = Takeoff::duct();
                value.profile = "rectangular duct".into();
                value.size = format!("{} x {} in", width.trim_end_matches(" in"), height.trim_end_matches(" in"));
                value.validate().unwrap();
            }
        }
        let colors: Vec<_> = DUCT_MATERIALS
            .iter()
            .map(|material| {
                let mut value = Takeoff::duct();
                value.material = (*material).into();
                value.color()
            })
            .collect();
        for (index, color) in colors.iter().enumerate() {
            assert!(!colors.iter().skip(index + 1).any(|other| other == color));
        }
    }

    #[test]
    fn rectangular_dimensions_reject_repeated_unit_suffixes() {
        for size in ["4 in x 6 in in", "4 in x 6 in in in", "4 in in x 6 in", "4 x 6 in in"] {
            let mut value = Takeoff::duct();
            value.profile = "rectangular duct".into();
            value.size = size.into();
            assert!(value.validate().is_err(), "{size}");
        }
        let mut valid = Takeoff::duct();
        valid.profile = "rectangular duct".into();
        valid.size = "4 in x 6 in".into();
        valid.validate().unwrap();
    }

    #[test]
    fn hvac_rejects_unsupported_material_system_and_size() {
        for (field, text) in [("material", "PVC"), ("service", "Steam"), ("size", "2.75 in"), ("size", "0 in")] {
            let mut value = Takeoff::pipe();
            match field {
                "material" => value.material = text.into(),
                "service" => value.service = text.into(),
                _ => value.size = text.into(),
            }
            assert!(value.validate().is_err(), "{value:?}");
        }
        for size in ["3 in", "50 in", "4 x 5 in", "4 x 48 x 6 in"] {
            let mut value = Takeoff::duct();
            if size.contains('x') {
                value.profile = "rectangular duct".into();
            }
            value.size = size.into();
            assert!(value.validate().is_err(), "{value:?}");
        }
    }
}
