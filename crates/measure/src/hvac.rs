//! Nominal HVAC specifications attached to measured centerline routes.

use serde::{Deserialize, Serialize};

pub const PIPE_SIZES: &[&str] = &["1/4 in", "3/8 in", "1/2 in", "3/4 in", "1 in", "1-1/4 in", "1-1/2 in", "2 in", "2-1/2 in", "3 in", "4 in", "5 in", "6 in", "8 in", "10 in", "12 in"];
pub const ROUND_DUCT_SIZES: &[&str] = &["4 in", "5 in", "6 in", "7 in", "8 in", "9 in", "10 in", "12 in", "14 in", "16 in", "18 in", "20 in", "24 in", "30 in", "36 in"];
pub const RECTANGULAR_DUCT_SIZES: &[&str] = &["8 x 8 in", "10 x 10 in", "12 x 8 in", "12 x 12 in", "16 x 8 in", "16 x 12 in", "18 x 12 in", "20 x 12 in", "24 x 12 in", "24 x 18 in", "24 x 24 in"];

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
        Self { profile: "pipe".into(), material: "Copper".into(), size: "1 in".into(), service: "Hydronic supply".into() }
    }

    pub fn duct() -> Self {
        Self { profile: "round duct".into(), material: "Galvanized steel".into(), size: "12 in".into(), service: "Supply".into() }
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
        Ok(())
    }
}
