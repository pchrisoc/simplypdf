use simplypdf_core::measurement::{DistanceScale, MeasurementError};

const REFERENCE: [[f64; 2]; 2] = [[120.0, 120.0], [180.0, 200.0]];

#[test]
fn measures_straight_and_continuous_paths_at_the_same_scale() {
    let scale = DistanceScale::calibrate(REFERENCE, 2_500.0).unwrap();
    assert_eq!(scale.length_mm(&REFERENCE).unwrap(), 2_500.0);
    let legs = [[120.0, 120.0], [180.0, 120.0], [180.0, 200.0]];
    assert_eq!(scale.length_mm(&legs[..2]).unwrap(), 1_500.0);
    assert_eq!(scale.length_mm(&legs[1..]).unwrap(), 2_000.0);
    assert_eq!(scale.length_mm(&legs).unwrap(), 3_500.0);
    assert_eq!(scale.length_mm(&[[0.0, 0.0]; 2]).unwrap(), 0.0);
}

#[test]
fn rejects_invalid_references_and_paths() {
    for length in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            DistanceScale::calibrate(REFERENCE, length).unwrap_err(),
            MeasurementError::InvalidReference
        );
    }
    assert_eq!(
        DistanceScale::calibrate([[0.0, 0.0]; 2], 1.0).unwrap_err(),
        MeasurementError::InvalidReference
    );
    let scale = DistanceScale::calibrate(REFERENCE, 2_500.0).unwrap();
    assert_eq!(scale.length_mm(&[]), Err(MeasurementError::TooFewPoints));
    assert_eq!(
        scale.length_mm(&[[0.0, 0.0], [f64::NAN, 0.0]]),
        Err(MeasurementError::InvalidGeometry)
    );
}

#[test]
fn reports_numeric_range_errors() {
    let unit_span = [[0.0, 0.0], [1.0, 0.0]];
    let scale = DistanceScale::calibrate(unit_span, f64::MAX).unwrap();
    assert_eq!(
        scale.length_mm(&[[0.0, 0.0], [2.0, 0.0]]),
        Err(MeasurementError::UnrepresentableLength)
    );
    let scale = DistanceScale::calibrate(unit_span, f64::MIN_POSITIVE).unwrap();
    assert_eq!(
        scale.length_mm(&[[0.0, 0.0], [f64::MIN_POSITIVE, 0.0]]),
        Err(MeasurementError::UnrepresentableLength)
    );
}
