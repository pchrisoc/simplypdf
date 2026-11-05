//! Scale-independent path geometry and calibrated physical lengths.

use crate::geometry::Point;

/// Why a path or calibration could not produce a physical length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasurementError {
    TooFewPoints,
    InvalidGeometry,
    InvalidReference,
    UnrepresentableLength,
}

impl std::fmt::Display for MeasurementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::TooFewPoints => "a path requires at least two points",
            Self::InvalidGeometry => "path coordinates and lengths must be finite",
            Self::InvalidReference => "calibration requires a positive reference length and span",
            Self::UnrepresentableLength => "physical length is outside the supported numeric range",
        })
    }
}

impl std::error::Error for MeasurementError {}

/// A uniform scale derived from a known distance, in millimeters per canonical
/// page point. Scope, revision, and verification status belong to the caller.
#[derive(Debug, Clone, Copy)]
pub struct DistanceScale {
    millimeters_per_point: f64,
}

impl DistanceScale {
    /// Calibrate against a known positive length and two canonical page points.
    pub fn calibrate(reference: [Point; 2], millimeters: f64) -> Result<Self, MeasurementError> {
        let span = path_points(&reference)?;
        if span == 0.0 || !millimeters.is_finite() || millimeters <= 0.0 {
            return Err(MeasurementError::InvalidReference);
        }

        let millimeters_per_point = millimeters / span;
        if !millimeters_per_point.is_finite() || millimeters_per_point == 0.0 {
            return Err(MeasurementError::UnrepresentableLength);
        }
        Ok(Self {
            millimeters_per_point,
        })
    }

    /// Measure consecutive segments in canonical page space. The result is
    /// unrounded millimeters. Repeated vertices contribute zero length.
    pub fn length_mm(self, points: &[Point]) -> Result<f64, MeasurementError> {
        let length = path_points(points)?;
        let millimeters = length * self.millimeters_per_point;
        if !millimeters.is_finite() || (length > 0.0 && millimeters == 0.0) {
            return Err(MeasurementError::UnrepresentableLength);
        }
        Ok(millimeters)
    }
}

fn path_points(points: &[Point]) -> Result<f64, MeasurementError> {
    if points.len() < 2 {
        return Err(MeasurementError::TooFewPoints);
    }
    let mut length = 0.0;
    for pair in points.windows(2) {
        let [x0, y0] = pair[0];
        let [x1, y1] = pair[1];
        length += (x1 - x0).hypot(y1 - y0);
        if !length.is_finite() {
            return Err(MeasurementError::InvalidGeometry);
        }
    }
    Ok(length)
}
