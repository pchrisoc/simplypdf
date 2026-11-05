use simplypdf_core::measurement::{DistanceScale, MeasurementError};
use simplypdf_core::preview::{LengthUnit, PreviewError, PreviewStatus, ResolvedScale, preview_distance};

const PATH: [[f64; 2]; 2] = [[120.0, 120.0], [180.0, 200.0]];

fn active(scale: DistanceScale, status: PreviewStatus) -> ResolvedScale<'static> {
    ResolvedScale::Active {
        scale,
        status,
        calibration_id: "cal-1",
        calibration_revision: 3,
        region_id: Some("detail-a"),
    }
}

#[test]
fn preview_converts_units_and_preserves_calibration_provenance() {
    let scale = DistanceScale::calibrate(PATH, 304.8).unwrap();
    for (unit, expected) in [
        (LengthUnit::Millimeter, 304.8),
        (LengthUnit::Meter, 0.3048),
        (LengthUnit::Inch, 12.0),
        (LengthUnit::Foot, 1.0),
    ] {
        let preview = preview_distance(&PATH, active(scale, PreviewStatus::Checked), unit).unwrap();
        assert!((preview.length - expected).abs() < 1e-12);
        assert_eq!(preview.unit, unit);
        assert_eq!(preview.status, PreviewStatus::Checked);
        assert_eq!(preview.calibration_id, "cal-1");
        assert_eq!(preview.calibration_revision, 3);
        assert_eq!(preview.region_id, Some("detail-a"));
    }
}

#[test]
fn draft_previews_carry_failed_and_unchecked_status() {
    let scale = DistanceScale::calibrate(PATH, 304.8).unwrap();
    for status in [PreviewStatus::Failed, PreviewStatus::Unchecked] {
        let preview = preview_distance(&PATH, active(scale, status), LengthUnit::Inch).unwrap();
        assert_eq!(preview.status, status);
    }
}

#[test]
fn distinguishes_scale_errors_from_invalid_geometry() {
    for (scale, error) in [
        (ResolvedScale::Missing, PreviewError::MissingScale),
        (ResolvedScale::Ambiguous, PreviewError::AmbiguousScale),
        (ResolvedScale::Stale, PreviewError::StaleScale),
    ] {
        assert_eq!(preview_distance(&PATH, scale, LengthUnit::Inch), Err(error));
    }

    let scale = DistanceScale::calibrate(PATH, 304.8).unwrap();
    assert_eq!(
        preview_distance(&PATH[..1], active(scale, PreviewStatus::Checked), LengthUnit::Inch),
        Err(PreviewError::Measurement(MeasurementError::TooFewPoints))
    );
}

#[test]
fn reports_underflow_during_unit_conversion() {
    let path = [[0.0, 0.0], [1.0, 0.0]];
    let scale = DistanceScale::calibrate(path, f64::from_bits(1)).unwrap();
    assert_eq!(
        preview_distance(&path, active(scale, PreviewStatus::Checked), LengthUnit::Meter),
        Err(PreviewError::Measurement(MeasurementError::UnrepresentableLength))
    );
}
