use simplypdf_core::geometry::{GeometryError, PageGeometry, Point};

const MEDIA: [Point; 2] = [[-10.0, 20.0], [310.0, 260.0]];
const CROP: [Point; 2] = [[30.0, 50.0], [230.0, 200.0]];

fn assert_point(actual: Point, expected: Point) {
    for (actual, expected) in actual.into_iter().zip(expected) {
        assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
    }
}

#[test]
fn raw_pdf_units_are_normalized_relative_to_media_not_crop() {
    let page = PageGeometry::new(MEDIA, Some(CROP), 2.0, 90).unwrap();
    let start = page.pdf_to_page([50.0, 80.0]).unwrap();
    let end = page.pdf_to_page([80.0, 120.0]).unwrap();
    assert_point(start, [120.0, 120.0]);
    assert_point(end, [180.0, 200.0]);
    assert_eq!((end[0] - start[0]).hypot(end[1] - start[1]), 100.0);
    assert_point(page.page_to_pdf(end).unwrap(), [80.0, 120.0]);
}

#[test]
fn all_rotations_match_independent_canvas_positions() {
    let cases = [
        (0, [40.0, 240.0], [400.0, 300.0]),
        (90, [60.0, 40.0], [300.0, 400.0]),
        (180, [360.0, 60.0], [400.0, 300.0]),
        (-90, [240.0, 360.0], [300.0, 400.0]),
    ];
    for (rotation, expected, size) in cases {
        let page = PageGeometry::new(MEDIA, Some(CROP), 2.0, rotation).unwrap();
        let point = [120.0, 120.0];
        for (origin, factor) in [([0.0, 0.0], 1.0), ([-15.5, 27.0], 2.5)] {
            let view = page.viewport(origin, factor).unwrap();
            let canvas = [origin[0] + expected[0] * factor, origin[1] + expected[1] * factor];
            assert_point(view.size(), [size[0] * factor, size[1] * factor]);
            assert_point(view.page_to_canvas(point).unwrap(), canvas);
            assert_point(view.canvas_to_page(canvas).unwrap(), point);
        }
    }
}

#[test]
fn crop_is_clipped_to_media_and_defaults_to_media() {
    let default = PageGeometry::new(MEDIA, None, 1.0, 360).unwrap();
    assert_point(default.viewport([0.0, 0.0], 1.0).unwrap().size(), [320.0, 240.0]);
    let crop = [[-30.0, 10.0], [100.0, 100.0]];
    let clipped = PageGeometry::new(MEDIA, Some(crop), 1.0, 0).unwrap();
    assert_point(clipped.viewport([0.0, 0.0], 1.0).unwrap().size(), [110.0, 80.0]);
}

#[test]
fn rejects_malformed_geometry_and_points() {
    assert_eq!(
        PageGeometry::new(MEDIA, Some([[400.0, 500.0], [410.0, 510.0]]), 1.0, 0)
            .unwrap_err(),
        GeometryError::InvalidBox
    );
    assert_eq!(
        PageGeometry::new(MEDIA, None, f64::INFINITY, 0).unwrap_err(),
        GeometryError::InvalidUserUnit
    );
    assert_eq!(
        PageGeometry::new(MEDIA, None, 1.0, 45).unwrap_err(),
        GeometryError::InvalidRotation
    );
    let page = PageGeometry::new(MEDIA, None, 1.0, 0).unwrap();
    assert_eq!(page.viewport([0.0, 0.0], 0.0).unwrap_err(), GeometryError::InvalidViewport);
    assert_eq!(page.pdf_to_page([f64::NAN, 0.0]), Err(GeometryError::InvalidPoint));
}
