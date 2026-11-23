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

/// Quantity breakdown separating external walls and opening returns.
///
/// Both wall areas exclude horizontal top and bottom faces. Opening geometry
/// must already be verified as contained and non-overlapping by the caller.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExtrusionBreakdown {
    pub net: ExtrudedQuantity,
    pub opening_area: f64,
    pub outer_wall_area: f64,
    pub opening_wall_area: f64,
}

/// Calculate auditable net volume and separate exterior/interior wall areas.
pub fn extrude_breakdown(
    gross_area: f64,
    outer_perimeter: f64,
    openings: &[(f64, f64)],
    depth: f64,
) -> Result<ExtrusionBreakdown> {
    let net = extrude_with_openings(gross_area, outer_perimeter, openings, depth)?;
    let opening_area = gross_area - net.footprint_area;
    let outer_wall_area = outer_perimeter * depth;
    let opening_wall_area = net.wall_area - outer_wall_area;
    if !opening_area.is_finite() || !outer_wall_area.is_finite() || !opening_wall_area.is_finite() {
        return Err(MeasureError::Invalid("extrusion breakdown exceeds supported numeric range".into()));
    }
    Ok(ExtrusionBreakdown { net, opening_area, outer_wall_area, opening_wall_area })
}

/// Extrude a footprint with non-overlapping, fully contained through-openings.
///
/// Opening areas are subtracted from the gross footprint. Opening perimeters
/// contribute exposed interior walls. The caller must verify geometric
/// containment and non-overlap before passing measured openings.
pub fn extrude_with_openings(
    gross_area: f64,
    outer_perimeter: f64,
    openings: &[(f64, f64)],
    depth: f64,
) -> Result<ExtrudedQuantity> {
    // Validate the outer geometry and depth even for an empty opening list.
    extrude(gross_area, outer_perimeter, depth)?;
    let mut net_area = gross_area;
    let mut exposed_perimeter = outer_perimeter;
    for &(area, perimeter) in openings {
        if !area.is_finite() || !perimeter.is_finite() || area < 0.0 || perimeter < 0.0 {
            return Err(MeasureError::Invalid("opening area and perimeter must be finite and nonnegative".into()));
        }
        if area > net_area {
            return Err(MeasureError::Invalid("openings exceed the available footprint area".into()));
        }
        net_area -= area;
        exposed_perimeter += perimeter;
        if !exposed_perimeter.is_finite() {
            return Err(MeasureError::Invalid("opening perimeters exceed supported numeric range".into()));
        }
    }
    extrude(net_area, exposed_perimeter, depth)
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
    fn openings_reduce_volume_and_add_interior_wall_area() {
        // 10x10 footprint, 2x2 opening, 3-unit depth.
        let result = extrude_with_openings(100.0, 40.0, &[(4.0, 8.0)], 3.0).expect("valid opening");
        assert_eq!(result.footprint_area, 96.0);
        assert_eq!(result.volume, 288.0);
        assert_eq!(result.wall_area, 144.0);
        assert_eq!(extrude_with_openings(100.0, 40.0, &[], 3.0), extrude(100.0, 40.0, 3.0));
    }

    #[test]
    fn breakdown_separates_opening_returns_from_external_walls() {
        let result = extrude_breakdown(100.0, 40.0, &[(4.0, 8.0)], 3.0).expect("valid room");
        assert_eq!(result.opening_area, 4.0);
        assert_eq!(result.outer_wall_area, 120.0);
        assert_eq!(result.opening_wall_area, 24.0);
        assert_eq!(result.net.volume, 288.0);
        assert_eq!(result.net.wall_area, result.outer_wall_area + result.opening_wall_area);
        let empty = extrude_breakdown(100.0, 40.0, &[], 3.0).expect("no openings");
        assert_eq!(empty.opening_area, 0.0);
        assert_eq!(empty.opening_wall_area, 0.0);
        assert!(extrude_breakdown(10.0, 12.0, &[(11.0, 4.0)], 2.0).is_err());
    }

    #[test]
    fn openings_reject_invalid_and_cumulative_excess() {
        assert!(extrude_with_openings(10.0, 12.0, &[(6.0, 4.0), (5.0, 4.0)], 2.0).is_err());
        assert!(extrude_with_openings(10.0, 12.0, &[(f64::NAN, 4.0)], 2.0).is_err());
        assert!(extrude_with_openings(10.0, 12.0, &[(1.0, f64::INFINITY)], 2.0).is_err());
        assert!(extrude_with_openings(10.0, 12.0, &[(1.0, -1.0)], 2.0).is_err());
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
