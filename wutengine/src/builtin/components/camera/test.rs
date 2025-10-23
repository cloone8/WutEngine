use super::Camera;
use crate::prelude::{CameraViewport, FieldOfView};

#[test]
fn test_fov_conversion_v_to_h() {
    let aspect_ratio = 1920.0 / 1080.0;

    assert_eq!(
        66_f32,
        FieldOfView::Vertical(40.0)
            .get_horizontal(aspect_ratio)
            .round()
    );
    assert_eq!(
        75_f32,
        FieldOfView::Vertical(47.0)
            .get_horizontal(aspect_ratio)
            .round()
    );
    assert_eq!(
        82_f32,
        FieldOfView::Vertical(52.0)
            .get_horizontal(aspect_ratio)
            .round()
    );
    assert_eq!(
        90_f32,
        FieldOfView::Vertical(59.0)
            .get_horizontal(aspect_ratio)
            .round()
    );
    assert_eq!(
        106_f32,
        FieldOfView::Vertical(73.0)
            .get_horizontal(aspect_ratio)
            .round()
    );
}

#[test]
fn test_fov_conversion_h_to_v() {
    let aspect_ratio = 1920.0 / 1080.0;

    assert_eq!(
        40_f32,
        FieldOfView::Horizontal(66.0)
            .get_vertical(aspect_ratio)
            .round()
    );
    assert_eq!(
        47_f32,
        FieldOfView::Horizontal(75.0)
            .get_vertical(aspect_ratio)
            .round()
    );
    assert_eq!(
        52_f32,
        FieldOfView::Horizontal(82.0)
            .get_vertical(aspect_ratio)
            .round()
    );
    assert_eq!(
        59_f32,
        FieldOfView::Horizontal(90.0)
            .get_vertical(aspect_ratio)
            .round()
    );
    assert_eq!(
        73_f32,
        FieldOfView::Horizontal(106.0)
            .get_vertical(aspect_ratio)
            .round()
    );
}
