use crate::math::Mat4;

/// The different types of possible [super::Camera] projections.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraProjection {
    /// Perspective-projecting camera.
    Perspective(FieldOfView),

    /// Orthographic-projecting camera. Value defines vertical viewing volume.
    /// Horizontal volume is determined through aspect ratio
    Orthographic(f32),
}

impl CameraProjection {
    /// Returns the projection matrix corresponding to this [CameraProjection] and the
    /// render target clipping planes and aspect ratio
    pub fn get_matrix(self, aspect_ratio: f32, clip_near: f32, clip_far: f32) -> Mat4 {
        match self {
            Self::Perspective(fov) => Mat4::perspective_lh(
                fov.get_vertical(aspect_ratio).to_radians(),
                aspect_ratio,
                clip_near,
                clip_far,
            ),
            Self::Orthographic(vertical_size) => {
                let half_vertical_size = vertical_size / 2.0;
                let half_horizontal_size = half_vertical_size * aspect_ratio;

                Mat4::orthographic_lh(
                    -half_horizontal_size,
                    half_horizontal_size,
                    -half_vertical_size,
                    vertical_size,
                    clip_near,
                    clip_far,
                )
            }
        }
    }
}

/// Field-of-view definition for a [CameraProjection]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldOfView {
    /// Vertical degrees
    Vertical(f32),

    /// Horizontal degrees
    Horizontal(f32),
}

impl FieldOfView {
    /// Returns the vertical field of view in degrees
    pub fn get_vertical(self, aspect_ratio: f32) -> f32 {
        match self {
            FieldOfView::Vertical(vfov) => vfov,
            FieldOfView::Horizontal(hfov) => {
                let h_rad = hfov.to_radians();

                let vfov_rad = 2.0 * f32::atan(f32::tan(h_rad * 0.5) / aspect_ratio);

                vfov_rad.to_degrees()
            }
        }
    }

    /// Returns the horizontal field of view in degrees
    pub fn get_horizontal(self, aspect_ratio: f32) -> f32 {
        match self {
            FieldOfView::Vertical(vfov) => {
                let v_rad = vfov.to_radians();

                let hfov_rad = 2.0 * f32::atan(f32::tan(v_rad * 0.5) * aspect_ratio);

                hfov_rad.to_degrees()
            }
            FieldOfView::Horizontal(hfov) => hfov,
        }
    }
}

#[cfg(test)]
mod test_fov {
    use super::FieldOfView;

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
}
