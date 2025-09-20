use core::fmt::Display;
use core::num::NonZero;
use core::ops::Deref;

use serde::{Deserialize, Serialize};
use wutengine_event::EventSubscription;
use wutengine_graphics::buffer::GpuBuffer;
use wutengine_graphics::color::Color;
use wutengine_graphics::shader::constants::ViewportConstants;
use wutengine_graphics::viewport::Viewport;
use wutengine_graphics::wgpu::BufferUsages;
use wutengine_math::Mat4;
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

    #[serde(skip)]
    runtime: CameraRuntimeData,
}

#[derive(Debug)]
struct CameraRuntimeData {
    projection_mat: Mat4,
    view_mat: Mat4,
    viewport: Viewport,
    viewport_dirty: bool,
}

impl Default for CameraRuntimeData {
    fn default() -> Self {
        Self {
            projection_mat: Mat4::IDENTITY,
            view_mat: Mat4::IDENTITY,
            viewport: Viewport::new(None),
            viewport_dirty: true,
        }
    }
}

impl Clone for CameraRuntimeData {
    fn clone(&self) -> Self {
        Self {
            projection_mat: self.projection_mat,
            view_mat: self.view_mat,
            viewport: Viewport::new(Some(&ViewportConstants {
                view_mat: self.view_mat,
                projection_mat: self.projection_mat,
                vp_mat: self.projection_mat * self.view_mat,
            })),
            viewport_dirty: false,
        }
    }
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
    Surface(crate::graphics::wgpu::SurfaceTexture),
}

impl Deref for CameraTargetTexture {
    type Target = crate::graphics::wgpu::Texture;

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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Viewport(offset=({}, {}), dimensions=({}, {}))",
            self.x, self.y, self.w, self.h
        )
    }
}

impl Camera {
    pub(crate) fn get_viewport_bind_group(&self) -> &graphics::wgpu::BindGroup {
        assert!(
            !self.runtime.viewport_dirty,
            "Attempting to get bindings to dirty viewport. Engine error"
        );

        self.runtime.viewport.get_bind_group()
    }

    pub(crate) fn update_viewport_buffer(&mut self) {
        if !self.runtime.viewport_dirty {
            return;
        }

        self.runtime.viewport.update(&ViewportConstants {
            view_mat: self.runtime.view_mat,
            projection_mat: self.runtime.projection_mat,
            vp_mat: self.runtime.view_mat * self.runtime.projection_mat,
        });

        self.runtime.viewport_dirty = false;
    }

    pub(crate) fn remake_view_mat(&mut self) {
        //TODO: Take from transform matrix of GameObject the camera is placed on
        self.runtime.view_mat = Mat4::IDENTITY;
        self.runtime.viewport_dirty = true;
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

        self.runtime.projection_mat = match self.projection {
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

        self.runtime.viewport_dirty = true;
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
            runtime: CameraRuntimeData::default(),
        }
    }

    pub fn set_projection(&mut self, projection: CameraProjection) {
        self.projection = projection;
        self.remake_proj_mat();
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
        self.runtime.view_mat
    }

    #[inline(always)]
    pub fn get_projection_mat(&self) -> Mat4 {
        self.runtime.projection_mat
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
