use core::fmt::Display;

use wutengine_util_macro::unique_id_type32;

use crate::color::Color;
use crate::component::Component;
use crate::graphics::material::NativeMaterial;
use crate::system::Phase;
use crate::window::Window;
use crate::{builtins, graphics};

unique_id_type32! {
    /// The ID of a [Camera]. Used for filtering in draw calls
    pub CameraId
}

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
    /// The ID of the camera. Used for filtering in draw calls
    id: CameraId,

    render_target: Option<wgpu::Texture>,

    blit_material: Option<NativeMaterial>,
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
            id: CameraId::new(),
            blit_material: None,
        }
    }

    /// Returns the ID of this camera
    #[inline]
    pub const fn get_id(&self) -> CameraId {
        self.id
    }

    /// Updates the target surface of this camera
    #[inline]
    pub fn set_target(&mut self, target: Option<CameraTarget>) {
        self.target = target;
    }

    /// Sets the background of this camera
    #[inline]
    pub fn set_background(&mut self, background: CameraBackground) {
        self.background = background;
    }

    /// Sets the background of this camera
    #[inline]
    pub fn set_viewport(&mut self, viewport: CameraViewport) {
        if !viewport.is_valid() {
            log::error!("Given viewport is invalid and will not be set: {viewport}");
            return;
        }

        self.viewport = viewport;
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

        if !needs_recreation {
            return;
        }

        log::debug!(
            "Recreating render target texture of size {}x{} for camera",
            target_size.0,
            target_size.1
        );

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

/// Internal functionality for rendering
impl Camera {
    pub(crate) fn begin_pass<'a>(
        &self,
        encoder: &'a mut wgpu::CommandEncoder,
    ) -> Option<wgpu::RenderPass<'a>> {
        let Some(render_target) = &self.render_target else {
            // No render target means no rendering
            return None;
        };

        let render_target_view = render_target.create_view(&wgpu::TextureViewDescriptor::default());

        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Camera main render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &render_target_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: self.background.to_wgpu_load_op(),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        Some(pass)
    }

    pub(crate) fn blit_to_target(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        windows: &[(Window, wgpu::SurfaceTexture)],
    ) {
        profiling::function_scope!();

        let Some(target) = self.target else {
            // No target means nowhere to blit to
            return;
        };

        let Some(rendered_image) = &self.render_target else {
            // No intermediate image we rendered to
            return;
        };

        self.set_blit_material();

        let blit_target_texture = match target {
            CameraTarget::Window(window) => {
                let Some((_, surface)) = windows.iter().find(|(win, _)| *win == window) else {
                    // Target window is not within the given surfaces, so we can't blit to it.
                    log::debug!(
                        "Target window {window} did not have an entry in the windows map. Not blitting"
                    );
                    return;
                };

                surface.texture.clone()
            }
        };

        let view_format = blit_target_texture.format().add_srgb_suffix();

        let blit_target_view = blit_target_texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(view_format),
            ..Default::default()
        });

        let color_targets = [Some(wgpu::ColorTargetState {
            format: view_format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        })];

        let blit_pipeline = match graphics::pipeline::get_pipeline(
            self.blit_material.as_ref().unwrap(),
            &color_targets,
        ) {
            Ok(bp) => bp,
            Err(e) => {
                log::error!("Failed to get camera blit pipeline. Not blitting: {e}");
                return;
            }
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Camera Blit Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &blit_target_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        // let bind_group_layout =
        //     graphics::device().create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //         label: Some("Texture sampler group layout"),
        //         entries: &[
        //             wgpu::BindGroupLayoutEntry {
        //                 binding: 0,
        //                 visibility: wgpu::ShaderStages::FRAGMENT,
        //                 ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        //                 count: None,
        //             },
        //             wgpu::BindGroupLayoutEntry {
        //                 binding: 1,
        //                 visibility: wgpu::ShaderStages::FRAGMENT,
        //                 ty: wgpu::BindingType::Texture {
        //                     sample_type: wgpu::TextureSampleType::Float { filterable: true },
        //                     view_dimension: wgpu::TextureViewDimension::D2,
        //                     multisampled: false,
        //                 },
        //                 count: None,
        //             },
        //         ],
        //     });

        // // let texture = self.get_fun_texture();
        // let texture = rendered_image;
        // let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        // let sampler = graphics::device().create_sampler(&wgpu::SamplerDescriptor {
        //     address_mode_u: wgpu::AddressMode::ClampToEdge,
        //     address_mode_v: wgpu::AddressMode::ClampToEdge,
        //     address_mode_w: wgpu::AddressMode::ClampToEdge,
        //     mag_filter: wgpu::FilterMode::Linear,
        //     min_filter: wgpu::FilterMode::Nearest,
        //     mipmap_filter: wgpu::MipmapFilterMode::Nearest,
        //     ..Default::default()
        // });

        // let texture_bind_group = graphics::device().create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("Fun texture bind group"),
        //     layout: &bind_group_layout,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::Sampler(&sampler),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::TextureView(&view),
        //         },
        //     ],
        // });

        render_pass.set_pipeline(&blit_pipeline);
        // render_pass.set_bind_group(0, &texture_bind_group, &[]);

        let actual_target_size = blit_target_texture.size();
        render_pass.set_viewport(
            self.viewport.x * (actual_target_size.width as f32),
            self.viewport.y * (actual_target_size.height as f32),
            self.viewport.w * (actual_target_size.width as f32),
            self.viewport.h * (actual_target_size.height as f32),
            0.0,
            1.0,
        );
        render_pass.draw(0..3, 0..1);
    }

    fn set_blit_material(&mut self) {
        if self.blit_material.is_some() {
            return;
        }

        self.blit_material = Some(NativeMaterial::new(builtins::shaders::BLIT.clone()));
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

/// The background of the [Camera] viewport
#[derive(Debug, Clone, Copy)]
pub enum CameraBackground {
    /// No specific background. Probably contains the contents of the previous frame
    None,

    /// A specific background color
    Color(Color),
}

impl CameraBackground {
    fn to_wgpu_load_op(self) -> wgpu::LoadOp<wgpu::Color> {
        match self {
            Self::None => wgpu::LoadOp::Load,
            Self::Color(color) => wgpu::LoadOp::Clear(color.into()),
        }
    }
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
