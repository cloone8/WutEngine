use wutengine_shadercompiler::{CAMERA_PARAMS_BIND_GROUP_INDEX, MATERIAL_PARAMS_BIND_GROUP_INDEX};
use wutengine_util_macro::unique_id_type32;

use crate::color::Color;
use crate::component::Component;
use crate::graphics::BindGroup;
use crate::graphics::internal_bind_groups::create_camera_bind_group;
use crate::graphics::material::{Material, MaterialParameter};
use crate::graphics::mesh::MeshTopology;
use crate::graphics::sampler::{Filtering, Sampler, WrapMode, WrapModeType};
use crate::graphics::texture::Texture;
use crate::system::Phase;
use crate::util::map;
use crate::window::Window;
use crate::{builtins, graphics, math};

mod target;
pub use target::*;

mod projection;
pub use projection::*;

mod viewport;
pub use viewport::*;

mod background;
pub use background::*;

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

    /// Bind group for the per-camera parameters
    camera_parameters: Option<BindGroup>,

    render_target: Option<wgpu::Texture>,

    blit_material: Option<Material>,
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
            id: CameraId::new(),
            target: None,
            projection: CameraProjection::Perspective(FieldOfView::Vertical(70.0)),
            background: CameraBackground::Color(Color::BLACK),
            viewport: CameraViewport::FULL_WINDOW,
            clipping_planes: (0.1, 100.0),
            camera_parameters: None,
            render_target: None,
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

        if let Some(blit_material) = self.blit_material.as_mut() {
            // Also rebind the blit material here, if it already exists. Cheaper
            // than rebinding it every frame, if the render target doesn't change
            Self::set_blit_material_params(blit_material, &render_target_texture);
        }

        self.render_target = Some(render_target_texture);
    }
}

/// Internal functionality for rendering
impl Camera {
    pub(crate) fn begin_pass<'a>(
        &mut self,
        encoder: &'a mut wgpu::CommandEncoder,
    ) -> Option<wgpu::RenderPass<'a>> {
        let Some(render_target) = &self.render_target else {
            // No render target means no rendering
            return None;
        };

        let queue = graphics::queue();

        let view_mat = math::Mat4::IDENTITY; // TODO: Get actual view matrix once transforms are implemented

        let target_size = render_target.size();

        let projection_mat = self.projection.get_matrix(
            target_size.width as f32 / target_size.height as f32,
            self.clipping_planes.0,
            self.clipping_planes.1,
        );

        let vp_mat = projection_mat * view_mat;

        let cam_bind_group = Self::get_camera_bind_group(self.id, &mut self.camera_parameters);

        cam_bind_group
            .set_parameter("view", view_mat.into(), queue)
            .unwrap();
        cam_bind_group
            .set_parameter("projection", projection_mat.into(), queue)
            .unwrap();
        cam_bind_group
            .set_parameter("vp", vp_mat.into(), queue)
            .unwrap();

        cam_bind_group.update_bind_group(graphics::device());

        let render_target_view = render_target.create_view(&wgpu::TextureViewDescriptor::default());

        // Start the actual native pass and bind the per-camera parameters
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

        pass.set_bind_group(
            CAMERA_PARAMS_BIND_GROUP_INDEX,
            cam_bind_group.get_bind_group(),
            &[],
        );

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

        if self.render_target.is_none() {
            // We haven't rendered to an intermediate render target, so nothing to blit
            return;
        }

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

        let blit_material = self.blit_material.as_mut().unwrap();

        let blit_pipeline = match graphics::pipeline::get_pipeline(
            blit_material,
            MeshTopology::Triangle,
            &color_targets,
        ) {
            Ok(bp) => bp,
            Err(e) => {
                log::error!("Failed to get camera blit pipeline. Not blitting: {e}");
                return;
            }
        };

        blit_material
            .user_bind_group
            .update_bind_group(graphics::device());

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

        render_pass.set_pipeline(&blit_pipeline);
        render_pass.set_bind_group(
            MATERIAL_PARAMS_BIND_GROUP_INDEX,
            Some(blit_material.user_bind_group.get_bind_group()),
            &[],
        );

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

        let mut mat = Material::new(builtins::shaders::BLIT.clone(), map![]);

        if let Some(render_target_texture) = self.render_target.as_ref() {
            Self::set_blit_material_params(&mut mat, render_target_texture);
        }

        self.blit_material = Some(mat);
    }

    fn set_blit_material_params(mat: &mut Material, render_target_texture: &wgpu::Texture) {
        let tex_param = MaterialParameter::Texture2D(
            Texture::new_from_existing(
                render_target_texture.create_view(&wgpu::TextureViewDescriptor::default()),
            )
            .into(),
        );

        let sampler_param = MaterialParameter::Sampler(
            Sampler::new(Filtering::Linear, WrapModeType::Single(WrapMode::Clamp)).into(),
        );

        mat.user_bind_group
            .set_parameter("source_texture", tex_param, graphics::queue())
            .unwrap();

        mat.user_bind_group
            .set_parameter("source_sampler", sampler_param, graphics::queue())
            .unwrap();
    }

    fn get_camera_bind_group(
        id: CameraId,
        camera_parameters: &mut Option<BindGroup>,
    ) -> &mut BindGroup {
        if camera_parameters.is_none() {
            *camera_parameters = Some(create_camera_bind_group(format!(
                "Camera {} parameter bind group",
                id
            )));
        }

        camera_parameters.as_mut().unwrap()
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
            |_, camera| {
                profiling::scope!("Camera update render target");
                camera.update_render_target()
            },
        );
    }
}
