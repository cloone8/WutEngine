use core::fmt::Display;

use crate::color::Color;
use crate::component::Component;
use crate::graphics;
use crate::system::Phase;
use crate::window::Window;

/// A camera component. Renders a viewport
#[derive(Debug)]
pub struct Camera {
    // == Configuration ==
    /// The render target for this camera
    target: Option<CameraTarget>,

    /// The projection this camera uses
    projection: CameraProjection,

    /// The background of this camera's viewport
    background: CameraBackground,

    /// The viewport dimensions
    viewport: CameraViewport,

    /// The near/far clipping planes
    clipping_planes: (f32, f32),

    // == Runtime ==
    render_target: Option<wgpu::Texture>,
}

impl Default for Camera {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
/// Public API
impl Camera {
    /// Creates a new default camera component
    pub fn new() -> Self {
        Self {
            target: None,
            projection: CameraProjection::Perspective(FieldOfView::Vertical(70.0)),
            background: CameraBackground::Color(Color::BLACK),
            viewport: CameraViewport::FULL_WINDOW,
            clipping_planes: (0.1, 100.0),
            render_target: None,
        }
    }

    /// Updates the target surface of this camera
    #[inline]
    pub fn set_target(&mut self, target: Option<CameraTarget>) {
        self.target = target;
    }
}

/// System implementations
impl Camera {
    fn update_render_target(&mut self) {
        let Some(camera_target) = self.target else {
            // If the camera has no target configured, free the render target
            if let Some(render_target) = self.render_target.take() {
                render_target.destroy();
            }
            return;
        };

        let target_size = self.viewport.scale_size(camera_target.size());

        if target_size.0 == 0 || target_size.1 == 0 {
            // Target has no size. Usually due to the fact that the window is not yet created,
            // or already destroyed.
            log::debug!(
                "Camera not recreating render target because the camera target size cannot be determined"
            );
            return;
        }

        let needs_recreation = match &self.render_target {
            Some(rt) => {
                let cur_size = rt.size();

                let recreate = target_size != (cur_size.width, cur_size.height);

                if recreate {
                    rt.destroy();
                }

                recreate
            }
            None => true,
        };

        if needs_recreation {
            log::debug!(
                "Recreating render target texture of size {}x{} for camera",
                target_size.0,
                target_size.1
            );
        }

        let render_target_texture = graphics::device().create_texture(&wgpu::TextureDescriptor {
            label: Some("Camera render target texture"),
            size: wgpu::Extent3d {
                width: target_size.0,
                height: target_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        self.render_target = Some(render_target_texture);
    }
}

impl Component for Camera {
    fn insert_default_component_systems(manifest: &mut crate::runtime::SystemManifest)
    where
        Self: Sized,
    {
        manifest.add_system::<&mut Camera>(
            Phase::PreRender,
            Some("Camera update render target"),
            |_, camera| camera.update_render_target(),
        );
    }
}

/// The target surface on which a [Camera] will render its viewport
#[derive(Debug, Clone, Copy)]
pub enum CameraTarget {
    /// This camera renders to the given [Window]
    Window(Window),
}

impl CameraTarget {
    fn size(&self) -> (u32, u32) {
        match self {
            Self::Window(window) => window.get_size(),
        }
    }
}

/// The different types of possible [Camera] projections.
#[derive(Debug, Clone, Copy)]
pub enum CameraProjection {
    /// Perspective-projecting camera.
    Perspective(FieldOfView),

    /// Orthographic-projecting camera. Value defines vertical viewing volume.
    /// Horizontal volume is determined through aspect ratio
    Orthographic(f32),
}

/// Field-of-view definition for a [CameraProjection]
#[derive(Debug, Clone, Copy)]
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

                let vfov_rad = 2.0 * f32::atan(f32::tan(h_rad * 0.5) * (1.0 / aspect_ratio));

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

/// The background of the [Camera] viewport
#[derive(Debug, Clone, Copy)]
pub enum CameraBackground {
    /// No specific background. Probably contains the contents of the previous frame
    None,

    /// A specific background color
    Color(Color),
}

/// The configuration for the viewport of a [Camera]
#[derive(Debug, Clone, Copy)]
pub struct CameraViewport {
    /// Location of the left side of the viewport, as expressed as a fraction of the window. From 0.0-1.0
    pub x: f32,

    /// Location of the bottom of the viewport, as expressed as a fraction of the window. From 0.0-1.0
    pub y: f32,

    /// Width of the viewport, as expressed as a fraction of the window. From 0.0-1.0
    pub w: f32,

    /// Height viewport, as expressed as a fraction of the window. From 0.0-1.0
    pub h: f32,
}

impl CameraViewport {
    /// Camera viewport representing an entire window
    pub const FULL_WINDOW: Self = Self {
        x: 0.0,
        y: 0.0,
        w: 1.0,
        h: 1.0,
    };

    /// Checks that the viewport is configured to valid values
    pub const fn is_valid(&self) -> bool {
        self.x >= 0.0
            && self.x < 1.0
            && self.y >= 0.0
            && self.y < 1.0
            && self.w > 0.0
            && self.w <= 1.0
            && self.h > 0.0
            && self.h <= 1.0
    }

    /// Given a full window size, returns the size that this viewport would take,
    /// not accounting for any viewport areas that are cut off due to viewport positioning
    pub const fn scale_size(self, full_size: (u32, u32)) -> (u32, u32) {
        (
            (self.w * (full_size.0 as f32)) as u32,
            (self.h * (full_size.1 as f32)) as u32,
        )
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
