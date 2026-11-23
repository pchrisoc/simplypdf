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

/// Convert a calibrated footprint and a separately entered height into one linear unit.
///
/// `area_scale` converts one source squared unit to target squared units;
/// `perimeter_scale` and `depth_scale` convert one source linear unit to
/// target linear units. For example, square feet to square metres uses
/// `0.3048 * 0.3048`, while feet to metres uses `0.3048`.
/// Scale factors must be positive and finite to prevent silently mixing units.
pub fn extrude_converted(area: f64, perimeter: f64, depth: f64, area_scale: f64, perimeter_scale: f64, depth_scale: f64) -> Result<ExtrudedQuantity> {
    for (name, scale) in [("area scale", area_scale), ("perimeter scale", perimeter_scale), ("depth scale", depth_scale)] {
        if !scale.is_finite() || scale <= 0.0 {
            return Err(MeasureError::Invalid(format!("{name} must be finite and positive")));
        }
    }
    extrude(area * area_scale, perimeter * perimeter_scale, depth * depth_scale)
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
pub fn extrude_breakdown(gross_area: f64, outer_perimeter: f64, openings: &[(f64, f64)], depth: f64) -> Result<ExtrusionBreakdown> {
    let net = extrude_with_openings(gross_area, outer_perimeter, openings, depth)?;
    let opening_area = gross_area - net.footprint_area;
    let outer_wall_area = outer_perimeter * depth;
    // Sum opening returns directly to avoid cancellation when outer walls dominate.
    let opening_wall_area = openings.iter().try_fold(0.0, |sum, &(_, perimeter)| {
        let area = perimeter * depth;
        let total = sum + area;
        if total.is_finite() { Ok(total) } else { Err(MeasureError::Invalid("opening wall area exceeds supported numeric range".into())) }
    })?;
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
pub fn extrude_with_openings(gross_area: f64, outer_perimeter: f64, openings: &[(f64, f64)], depth: f64) -> Result<ExtrudedQuantity> {
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

/// Derive quantities for a rectangular room with rectangular through-openings.
/// Dimensions are calibrated linear units. Opening containment and non-overlap
/// must be checked by the drawing geometry layer before calling this helper.
pub fn rectangular_room(width: f64, length: f64, height: f64, openings: &[(f64, f64)]) -> Result<ExtrusionBreakdown> {
    for (name, value) in [("width", width), ("length", length), ("height", height)] {
        if !value.is_finite() || value < 0.0 {
            return Err(MeasureError::Invalid(format!("{name} must be finite and nonnegative")));
        }
    }
    let area = width * length;
    let perimeter = 2.0 * width + 2.0 * length;
    if !area.is_finite() || !perimeter.is_finite() {
        return Err(MeasureError::Invalid("room dimensions exceed supported numeric range".into()));
    }
    let mut opening_quantities = Vec::with_capacity(openings.len());
    for &(opening_width, opening_length) in openings {
        if !opening_width.is_finite() || !opening_length.is_finite() || opening_width < 0.0 || opening_length < 0.0 {
            return Err(MeasureError::Invalid("opening dimensions must be finite and nonnegative".into()));
        }
        let opening_area = opening_width * opening_length;
        let opening_perimeter = 2.0 * opening_width + 2.0 * opening_length;
        if !opening_area.is_finite() || !opening_perimeter.is_finite() {
            return Err(MeasureError::Invalid("opening dimensions exceed supported numeric range".into()));
        }
        opening_quantities.push((opening_area, opening_perimeter));
    }
    extrude_breakdown(area, perimeter, &opening_quantities, height)
}


/// A positioned axis-aligned through-opening in a rectangular footprint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectOpening {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub length: f64,
}

/// Derive a rectangular room only after checking opening containment and overlap.
///
/// Coordinates and dimensions are in calibrated linear units relative to the
/// footprint's lower-left corner. Touching boundaries are allowed; overlapping
/// interiors are not. This prevents subtracting invalid or duplicate cutouts.
pub fn rectangular_room_placed(width: f64, length: f64, height: f64, openings: &[RectOpening]) -> Result<ExtrusionBreakdown> {
    let mut dimensions = Vec::with_capacity(openings.len());
    for opening in openings {
        for value in [opening.x, opening.y, opening.width, opening.length] {
            if !value.is_finite() || value < 0.0 {
                return Err(MeasureError::Invalid("opening coordinates and dimensions must be finite and nonnegative".into()));
            }
        }
        if opening.width == 0.0 || opening.length == 0.0 {
            return Err(MeasureError::Invalid("through-openings must have positive width and length".into()));
        }
        if !width.is_finite() || !length.is_finite() || width < 0.0 || length < 0.0 || opening.x > width || opening.y > length || opening.width > width - opening.x || opening.length > length - opening.y {
            return Err(MeasureError::Invalid("opening lies outside the room footprint".into()));
        }
        dimensions.push((opening.width, opening.length));
    }
    for (i, a) in openings.iter().enumerate() {
        for b in &openings[i + 1..] {
            if a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.length && b.y < a.y + a.length {
                return Err(MeasureError::Invalid("through-openings overlap".into()));
            }
        }
    }
    rectangular_room(width, length, height, &dimensions)
}


/// Derive volume and wall area directly from a calibrated polygon measurement.
///
/// The boundary is implicitly closed. The area and distance reading units may
/// differ, so callers explicitly supply conversions to the desired output
/// squared and linear units. Depth is already in the output linear unit.
pub fn extrude_measured_polygon(points: &[crate::Point], scale: &crate::Scale, depth: f64, area_scale: f64, perimeter_scale: f64) -> Result<ExtrudedQuantity> {
    crate::validate_geometry(crate::Kind::Area, points)?;
    let points = crate::open_polygon(points);
    let area = crate::reading(crate::Kind::Area, points, scale)?.area;
    let mut perimeter = 0.0;
    for (a, b) in points.iter().zip(points.iter().cycle().skip(1)).take(points.len()) {
        perimeter += ((b[0] - a[0]) * scale.x).hypot((b[1] - a[1]) * scale.y) * scale.distance_factor;
    }
    extrude_converted(area, perimeter, depth, area_scale, perimeter_scale, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;



    #[test]
    fn measured_polygon_extrudes_closed_boundary_with_explicit_units() {
        let scale = crate::Scale::new(1.0, "ft", 2).expect("scale");
        let square = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
        let result = extrude_measured_polygon(&square, &scale, 3.0, 1.0, 1.0).expect("room");
        assert_eq!(result.volume, 300.0);
        assert_eq!(result.wall_area, 120.0);
        let metric = extrude_measured_polygon(&square, &scale, 3.048, 0.3048 * 0.3048, 0.3048).expect("metric");
        assert!((metric.volume - 300.0 * 0.3048_f64.powi(3)).abs() < 1e-10);
        assert!(extrude_measured_polygon(&square, &scale, -1.0, 1.0, 1.0).is_err());
        assert!(extrude_measured_polygon(&square, &scale, 3.0, 0.0, 1.0).is_err());
        assert!(extrude_measured_polygon(&square[..2], &scale, 3.0, 1.0, 1.0).is_err());
    }

    #[test]
    fn positioned_openings_validate_containment_and_overlap() {
        let a = RectOpening { x: 1.0, y: 1.0, width: 2.0, length: 2.0 };
        let b = RectOpening { x: 3.0, y: 1.0, width: 2.0, length: 2.0 };
        let room = rectangular_room_placed(10.0, 10.0, 3.0, &[a, b]).expect("touching openings");
        assert_eq!(room.net.volume, 276.0);
        assert_eq!(room.opening_wall_area, 48.0);
        assert!(rectangular_room_placed(10.0, 10.0, 3.0, &[a, a]).is_err());
        assert!(rectangular_room_placed(10.0, 10.0, 3.0, &[RectOpening { width: 0.0, ..a }]).is_err());
        assert!(rectangular_room_placed(10.0, 10.0, 3.0, &[RectOpening { length: 0.0, ..a }]).is_err());
        assert!(rectangular_room_placed(10.0, 10.0, 3.0, &[RectOpening { x: 9.0, ..a }]).is_err());
        assert!(rectangular_room_placed(10.0, 10.0, 3.0, &[RectOpening { x: f64::NAN, ..a }]).is_err());
        assert!(rectangular_room_placed(10.0, 10.0, 3.0, &[RectOpening { x: f64::MAX, ..a }]).is_err());
        assert!(rectangular_room_placed(f64::NAN, 10.0, 3.0, &[a]).is_err());
        assert!(rectangular_room_placed(-10.0, 10.0, 3.0, &[a]).is_err());
    }

    #[test]
    fn rectangular_room_dimensions_include_opening_returns() {
        let room = rectangular_room(10.0, 10.0, 3.0, &[(2.0, 2.0)]).expect("valid room");
        assert_eq!(room.net.volume, 288.0);
        assert_eq!(room.outer_wall_area, 120.0);
        assert_eq!(room.opening_wall_area, 24.0);
        assert_eq!(room.net.wall_area, 144.0);
    }

    #[test]
    fn rectangular_room_rejects_invalid_dimensions_and_excess_openings() {
        assert!(rectangular_room(f64::NAN, 10.0, 3.0, &[]).is_err());
        assert!(rectangular_room(10.0, -1.0, 3.0, &[]).is_err());
        assert!(rectangular_room(10.0, 10.0, f64::INFINITY, &[]).is_err());
        assert!(rectangular_room(10.0, 10.0, 3.0, &[(f64::NAN, 2.0)]).is_err());
        assert!(rectangular_room(10.0, 10.0, 3.0, &[(11.0, 10.0)]).is_err());
        assert!(rectangular_room(f64::MAX, 2.0, 3.0, &[]).is_err());
        assert!(rectangular_room(10.0, 10.0, 3.0, &[(f64::MAX, 2.0)]).is_err());
    }

    #[test]
    fn converted_extrusion_matches_metric_equivalent() {
        let converted = extrude_converted(100.0, 40.0, 10.0, 0.3048 * 0.3048, 0.3048, 0.3048).expect("feet to metres");
        let metric = extrude(9.290304, 12.192, 3.048).expect("metric");
        assert!((converted.volume - metric.volume).abs() < 1e-10);
        assert!((converted.wall_area - metric.wall_area).abs() < 1e-10);
    }

    #[test]
    fn converted_extrusion_rejects_invalid_scales_and_overflow() {
        for scale in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(extrude_converted(1.0, 1.0, 1.0, scale, 1.0, 1.0).is_err());
            assert!(extrude_converted(1.0, 1.0, 1.0, 1.0, scale, 1.0).is_err());
            assert!(extrude_converted(1.0, 1.0, 1.0, 1.0, 1.0, scale).is_err());
        }
        assert!(extrude_converted(f64::MAX, 1.0, 1.0, 2.0, 1.0, 1.0).is_err());
    }

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
