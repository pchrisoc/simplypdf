//! Explicit derived quantities for calibrated area takeoffs.
//!
//! All inputs must already use consistent real-world linear units. The caller
//! supplies an area in squared units and a depth/height in those linear units.

use crate::{MeasureError, Result};

/// Derived volume and exposed wall area for an extruded footprint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtrudedQuantity {
    /// Footprint area in square units.
    pub footprint_area: f64,
    /// Extrusion depth in linear units.
    pub depth: f64,
    /// Volume in cubic units: footprint_area * depth.
    pub volume: f64,
    /// Side-wall area in square units: perimeter * depth.
    pub wall_area: f64,
}

/// Calculate volume and lateral wall area from a calibrated footprint.
///
/// The perimeter must describe the same footprint as the area, including any
/// internal openings whose walls should be counted. Neither top nor bottom
/// face is included in wall_area.
pub fn extrude(footprint_area: f64, perimeter: f64, depth: f64) -> Result<ExtrudedQuantity> {
    for (name, value) in [("footprint area", footprint_area), ("perimeter", perimeter), ("depth", depth)] {
        if !value.is_finite() || value < 0.0 {
            return Err(MeasureError::Invalid(format!("{name} must be finite and nonnegative")));
        }
    }
    let volume = footprint_area * depth;
    let wall_area = perimeter * depth;
    if !volume.is_finite() || !wall_area.is_finite() {
        return Err(MeasureError::Invalid("derived quantity exceeds supported numeric range".into()));
    }
    Ok(ExtrudedQuantity { footprint_area, depth, volume, wall_area })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rectangular_room_uses_explicit_volume_and_lateral_wall_formulas() {
        let result = extrude(20.0, 18.0, 3.0).expect("valid room");
        assert_eq!(result.volume, 60.0);
        assert_eq!(result.wall_area, 54.0);
        assert_eq!(result.footprint_area, 20.0);
        assert_eq!(result.depth, 3.0);
    }

    #[test]
    fn zero_depth_produces_zero_derived_quantities() {
        let result = extrude(20.0, 18.0, 0.0).expect("zero depth");
        assert_eq!(result.volume, 0.0);
        assert_eq!(result.wall_area, 0.0);
    }

    #[test]
    fn rejects_negative_nonfinite_and_overflowing_inputs() {
        for invalid in [-1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(extrude(invalid, 1.0, 1.0).is_err());
            assert!(extrude(1.0, invalid, 1.0).is_err());
            assert!(extrude(1.0, 1.0, invalid).is_err());
        }
        assert!(extrude(f64::MAX, 1.0, 2.0).is_err());
        assert!(extrude(1.0, f64::MAX, 2.0).is_err());
    }
}
