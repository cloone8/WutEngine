use core::fmt::Display;
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
    viewport: CameraViewport,
    clipping_planes: (f32, f32),

    // Cached values
    #[serde(skip)]
    projection_mat: Mat4,

    #[serde(skip)]
    view_mat: Mat4,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CameraViewport {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl CameraViewport {
    pub const FULL_WINDOW: Self = Self {
        x: 0.0,
        y: 0.0,
        w: 1.0,
        h: 1.0,
    };

    pub fn is_valid(&self) -> bool {
        self.x >= 0.0
            && self.x < 1.0
            && self.y >= 0.0
            && self.y < 1.0
            && self.w > 0.0
            && self.w <= 1.0
            && self.h > 0.0
            && self.h <= 1.0
    }
}

impl Default for CameraViewport {
    fn default() -> Self {
        Self::FULL_WINDOW
    }
}

impl Display for CameraViewport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Viewport(offset=({}, {}), dimensions=({}, {}))",
            self.x, self.y, self.w, self.h
        )
    }
}

impl Camera {
    pub(crate) fn remake_view_mat(&mut self) {
        //TODO: Take from transform matrix of GameObject the camera is placed on
        self.view_mat = Mat4::IDENTITY;
    }

    fn remake_proj_mat(&mut self) {
        let phys_target_size = match self.target.as_ref() {
            Some(CameraTarget::Window(window_id)) => {
                let Some(phys_target_size) = crate::window::window_size(window_id) else {
                    log::warn!(
                        "Not recalculating projection matrix because the camera has an unknown window target: {window_id}"
                    );
                    return;
                };

                phys_target_size
            }
            None => {
                log::warn!(
                    "Not recalculating projection matrix because the camera does not have a target"
                );
                return;
            }
        };

        let aspect_ratio: f32 = phys_target_size.0 as f32 / phys_target_size.1 as f32;

        self.projection_mat = match self.projection {
            CameraProjection::Perspective(fov) => Mat4::perspective_lh(
                fov.get_vertical(aspect_ratio).to_radians() as f32,
                aspect_ratio as f32,
                self.clipping_planes.0,
                self.clipping_planes.1,
            ),
            CameraProjection::Orthographic(size) => {
                let half_size = size / 2.0;
                let half_horizontal_size = half_size * aspect_ratio;

                Mat4::orthographic_lh(
                    -half_horizontal_size,
                    half_horizontal_size,
                    -half_size,
                    half_size,
                    self.clipping_planes.0,
                    self.clipping_planes.1,
                )
            }
        };
    }
}

/// Public API for [Camera]
impl Camera {
    pub fn new() -> Self {
        Self {
            target: None,
            projection: CameraProjection::Perspective(FieldOfView::Vertical(70.0)),
            background: CameraBackground::Color(Color::BLACK),
            viewport: CameraViewport::FULL_WINDOW,
            clipping_planes: (0.1, 100.0),
            projection_mat: Mat4::ZERO,
            view_mat: Mat4::ZERO,
        }
    }

    pub fn set_window(&mut self, target: CameraTarget) {
        self.target = Some(target);
        self.remake_proj_mat();
    }

    pub fn set_background(&mut self, background: CameraBackground) {
        self.background = background;
    }

    pub fn set_viewport(&mut self, viewport: CameraViewport) {
        if !viewport.is_valid() {
            log::error!("Cannot set invalid viewport {viewport}");
            return;
        }

        self.viewport = viewport;
    }

    pub fn set_clipping_planes(&mut self, near: f32, far: f32) {
        if !near.is_normal() {
            log::error!("Cannot set near plane to invalid value: {near}");
            return;
        }

        if !far.is_normal() {
            log::error!("Cannot set far plane to invalid value: {far}");
            return;
        }

        if far <= near {
            log::error!("Far plane ({far}) closer than near plane ({near})");
            return;
        }

        self.clipping_planes = (near, far);
        self.remake_proj_mat();
    }

    #[inline(always)]
    pub fn get_view_mat(&self) -> Mat4 {
        self.view_mat
    }

    #[inline(always)]
    pub fn get_projection_mat(&self) -> Mat4 {
        self.projection_mat
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

impl Default for Camera {
    fn default() -> Self {
        Self::new()
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
