use core::ops::Deref;

use glam::Mat4;
use serde::{Deserialize, Serialize};
use wutengine_graphics::color::Color;
use wutengine_windowing::window::WindowIdentifier;

use crate::graphics;
use crate::prelude::Component;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    target: Option<CameraTarget>,
    projection: CameraProjection,
    background: CameraBackground,
    clipping_planes: (f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraTarget {
    Window(WindowIdentifier),
    //Texture(TODO)
}
/// The different types of possible camera projections.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CameraProjection {
    /// Perspective-projecting camera.
    Perspective(FieldOfView),

    /// Orthographic-projecting camera. Value defines vertical viewing volume.
    /// Horizontal volume is determined through aspect ratio
    Orthographic(f32),
}

/// Field-of-view definition for a [CameraProjection]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FieldOfView {
    /// Vertical degrees
    Vertical(f32),

    /// Horizontal degrees
    Horizontal(f32),
}

impl FieldOfView {
    pub fn get_vertical(self, aspect_ratio: f32) -> f32 {
        match self {
            FieldOfView::Vertical(vfov) => vfov,
            FieldOfView::Horizontal(hfov) => {
                let h_rad = hfov.to_radians();

                let vfov_rad = 2.0 * f32::atan(f32::tan(h_rad * 0.5) * (1.0 / aspect_ratio));

                vfov_rad.to_degrees()
            }
        }
    }

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CameraBackground {
    None,
    Color(Color),
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub(crate) enum CameraTargetTexture {
    Surface(wgpu::SurfaceTexture),
}

impl Deref for CameraTargetTexture {
    type Target = wgpu::Texture;

    fn deref(&self) -> &Self::Target {
        match self {
            CameraTargetTexture::Surface(surface_texture) => &surface_texture.texture,
        }
    }
}

impl Camera {
    pub fn new() -> Self {
        Self {
            target: None,
            projection: CameraProjection::Perspective(FieldOfView::Vertical(70.0)),
            background: CameraBackground::Color(Color::BLACK),
            clipping_planes: (0.1, 100.0),
        }
    }

    pub fn set_window(&mut self, target: CameraTarget) {
        self.target = Some(target);
    }

    pub fn set_background(&mut self, background: CameraBackground) {
        self.background = background;
    }

    pub fn get_view_mat(&self) -> Mat4 {
        Mat4::IDENTITY
    }

    pub fn get_projection_mat(&self) -> Mat4 {
        todo!();
        // let phys_window_size = window.size;
        // let aspect_ratio: f32 = phys_window_size.0 as f32 / phys_window_size.1 as f32;

        // match self.projection {
        //     CameraProjection::Perspective(fov) => Mat4::perspective_lh(
        //         fov.get_vertical(aspect_ratio).to_radians() as f32,
        //         aspect_ratio as f32,
        //         0.1,
        //         100.0,
        //     ),
        //     CameraProjection::Orthographic(size) => {
        //         let half_size = size / 2.0;
        //         let half_horizontal_size = half_size * aspect_ratio;

        //         Mat4::orthographic_lh(
        //             -half_horizontal_size as f32,
        //             half_horizontal_size as f32,
        //             -half_size as f32,
        //             half_size as f32,
        //             self.clipping_planes.0,
        //             self.clipping_planes.1,
        //         )
        //     }
        // }
    }

    pub(crate) fn get_target_texture(&self) -> Option<CameraTargetTexture> {
        self.target.as_ref().and_then(|target| match target {
            CameraTarget::Window(win_id) => {
                graphics::get_window_surface_texture(win_id).map(CameraTargetTexture::Surface)
            }
        })
    }

    pub fn get_background(&self) -> CameraBackground {
        self.background
    }
}

impl Component for Camera {}

#[cfg(test)]
mod test {
    use crate::prelude::FieldOfView;

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
