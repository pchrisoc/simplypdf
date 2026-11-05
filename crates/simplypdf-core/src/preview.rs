//! Read-only distance previews for desktop and MCP callers.

use crate::geometry::Point;
use crate::measurement::{DistanceScale, MeasurementError};

/// Verification state for a calibration that can still be previewed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewStatus {
    Unchecked,
    Checked,
    Failed,
}

/// Outcome from the caller's scale resolver for the entire path.
#[derive(Debug, Clone, Copy)]
pub enum ResolvedScale<'a> {
    Missing,
    Ambiguous,
    Stale,
    Active {
        scale: DistanceScale,
        status: PreviewStatus,
        calibration_id: &'a str,
        calibration_revision: u64,
        region_id: Option<&'a str>,
    },
}

/// Numeric output units. Lengths are unrounded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthUnit {
    Millimeter,
    Meter,
    Inch,
    Foot,
}

impl LengthUnit {
    fn millimeters(self) -> f64 {
        match self {
            Self::Millimeter => 1.0,
            Self::Meter => 1000.0,
            Self::Inch => 25.4,
            Self::Foot => 12.0 * 25.4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DistancePreview<'a> {
    pub length: f64,
    pub unit: LengthUnit,
    pub status: PreviewStatus,
    pub calibration_id: &'a str,
    pub calibration_revision: u64,
    pub region_id: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewError {
    MissingScale,
    AmbiguousScale,
    StaleScale,
    Measurement(MeasurementError),
}

/// Calculate a draft distance using an already-resolved scale.
///
/// Failed and unchecked scales produce a labeled draft preview. Final totals
/// require a separate eligibility check. Resolve page and region boundaries
/// before calling, using the full path in canonical page coordinates.
pub fn preview_distance<'a>(
    points: &[Point],
    resolved: ResolvedScale<'a>,
    unit: LengthUnit,
) -> Result<DistancePreview<'a>, PreviewError> {
    let (scale, status, calibration_id, calibration_revision, region_id) = match resolved {
        ResolvedScale::Missing => return Err(PreviewError::MissingScale),
        ResolvedScale::Ambiguous => return Err(PreviewError::AmbiguousScale),
        ResolvedScale::Stale => return Err(PreviewError::StaleScale),
        ResolvedScale::Active {
            scale,
            status,
            calibration_id,
            calibration_revision,
            region_id,
        } => (scale, status, calibration_id, calibration_revision, region_id),
    };
    let millimeters = scale.length_mm(points).map_err(PreviewError::Measurement)?;
    let length = millimeters / unit.millimeters();
    if millimeters > 0.0 && length == 0.0 {
        return Err(PreviewError::Measurement(
            MeasurementError::UnrepresentableLength,
        ));
    }

    Ok(DistancePreview {
        length,
        unit,
        status,
        calibration_id,
        calibration_revision,
        region_id,
    })
}
