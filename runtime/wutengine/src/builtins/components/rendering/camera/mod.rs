use core::any::TypeId;

use wutengine_assets::assets::{
    mesh::MeshTopology,
    sampler::{FilterMode, WrapMode, WrapModeType},
};
use wutengine_graphics::{BindGroup, internal_bind_groups::create_camera_bind_group, label, wgpu};
use wutengine_math::{Color, Mat4};
use wutengine_shadercompiler::{CAMERA_PARAMS_BIND_GROUP_INDEX, MATERIAL_PARAMS_BIND_GROUP_INDEX};
use wutengine_util::map;
use wutengine_util_macro::unique_id_type32;

use crate::{
    builtins,
    builtins::components::Transform,
    component::Component,
    graphics,
    graphics::{
        DrawCommand,
        material::{Material, MaterialParameter},
        renderpass::RenderPass,
        sampler::Sampler,
        texture::Texture,
    },
    system::Phase,
    window::Window,
};

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
    pub target: Option<CameraTarget>,

    /// The projection this camera uses
    pub projection: CameraProjection,

    /// The background of this camera's viewport
    pub background: CameraBackground,

    /// The viewport dimensions
    pub viewport: CameraViewport,

    /// The near/far clipping planes
    pub clipping_planes: (f32, f32),

    // == Runtime ==
    /// The ID of the camera. Used for filtering in draw calls
    id: CameraId,

    view_matrix: Mat4,

    projection_matrix: Mat4,

    /// Bind group for the per-camera parameters
    camera_parameters: Option<BindGroup>,

    render_target: Option<wgpu::Texture>,

    blit_material: Option<Material>,

    /// Render passes active on this camera. Updated before each frame by the main runtime
    pub(crate) render_passes: Vec<ActiveCameraRenderPass>,
}

/// Container for an enabled [RenderPass]
#[derive(derive_more::Debug)]
pub(crate) struct ActiveCameraRenderPass {
    /// The type of the pass
    pub(crate) type_id: TypeId,

    /// The name of the pass
    pub(crate) name: &'static str,

    /// The order of the pass relative to other passes
    pub(crate) order: u64,

    /// The pass itself
    #[debug(skip)]
    pub(crate) pass: Box<dyn for<'a> RenderPass<Camera, [DrawCommand]>>,
}

impl Default for Camera {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Error when the camera bind group wasn't initialized yet
#[derive(Debug, derive_more::Display, derive_more::Error)]
#[display(
    "Camera bind group not yet initialized. Make sure to call this function from inside a renderpass only"
)]
pub struct MissingCameraBindGroupErr;

/// Public API
impl Camera {
    /// Creates a new default camera component
    pub fn new() -> Self {
        Self {
            id: CameraId::new(),
            view_matrix: Mat4::IDENTITY,
            projection_matrix: Mat4::IDENTITY,
            target: None,
            projection: CameraProjection::Perspective(FieldOfView::Vertical(70.0)),
            background: CameraBackground::Color(Color::BLACK),
            viewport: CameraViewport::FULL_WINDOW,
            clipping_planes: (0.1, 100.0),
            camera_parameters: None,
            render_target: None,
            blit_material: None,
            render_passes: Vec::new(),
        }
    }

    /// Returns the ID of this camera
    #[inline]
    pub const fn get_id(&self) -> CameraId {
        self.id
    }

    /// Returns the current render target texture of this camera, if configured
    #[inline]
    pub fn get_render_target(&self) -> Option<&wgpu::Texture> {
        self.render_target.as_ref()
    }

    /// Sets this camera's parameter bind group on the given pass
    pub fn set_camera_bind_group_on_pass(
        &self,
        pass: &mut wgpu::RenderPass,
    ) -> Result<(), MissingCameraBindGroupErr> {
        let cam_bind_group = self
            .camera_parameters
            .as_ref()
            .ok_or(MissingCameraBindGroupErr)?;

        pass.set_bind_group(
            CAMERA_PARAMS_BIND_GROUP_INDEX,
            cam_bind_group.get_bind_group(),
            &[],
        );

        Ok(())
    }

    /// Returns the current view matrix for this camera
    #[inline]
    pub fn get_view_mat(&self) -> Mat4 {
        self.view_matrix
    }

    /// Returns the current projection matrix for this camera
    #[inline]
    pub fn get_proj_mat(&self) -> Mat4 {
        self.projection_matrix
    }
}

/// System implementations
impl Camera {
    fn update_render_target(&mut self) {
        let Some(camera_target) = self.target else {
            log::trace!("Camera has no target configured, so not updating render target");
            // If the camera has no target configured, free the render target
            if let Some(render_target) = self.render_target.take() {
                log::trace!("Freeing current camera render target");
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
            label: label!("Camera render target texture"),
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

    fn update_view_projection(&mut self, transform: Mat4) {
        let Some(render_target) = &self.render_target else {
            // No render target means no rendering
            return;
        };

        self.view_matrix = transform.inverse();

        let target_size = render_target.size();

        self.projection_matrix = self.projection.get_matrix(
            target_size.width as f32 / target_size.height as f32,
            self.clipping_planes.0,
            self.clipping_planes.1,
        );
    }

    fn update_cam_bind_group(&mut self) {
        if self.render_target.is_none() {
            // No render target means nothing to do
            return;
        }

        let queue = graphics::queue();

        let cam_bind_group = Self::get_camera_bind_group(self.id, &mut self.camera_parameters);

        cam_bind_group
            .set_parameter("view", self.view_matrix.into(), queue)
            .unwrap();
        cam_bind_group
            .set_parameter("projection", self.projection_matrix.into(), queue)
            .unwrap();

        let vp_mat = self.projection_matrix * self.view_matrix;

        cam_bind_group
            .set_parameter("vp", vp_mat.into(), queue)
            .unwrap();

        cam_bind_group.update_bind_group(graphics::device());
    }
}

/// Internal functionality for rendering
impl Camera {
    /// Blits this camera's rendertexture to the given target
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
            .raw_bind_group_mut()
            .update_bind_group(graphics::device());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: label!("Camera Blit Pass"),
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
            Some(
                blit_material
                    .raw_bind_group()
                    .get_bind_group()
                    .expect("Blit material bind group out of date"),
            ),
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
            Sampler::new(
                FilterMode::Linear,
                FilterMode::Linear,
                WrapModeType::Single(WrapMode::Clamp),
            )
            .into(),
        );

        mat.raw_bind_group_mut()
            .set_parameter("source_texture", tex_param, graphics::queue())
            .unwrap();

        mat.raw_bind_group_mut()
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
    const ID: uuid::NonNilUuid =
        uuid::NonNilUuid::new(uuid::uuid!("564bccfc-8b3e-49b5-9855-48c42cef713f")).unwrap();

    fn insert_default_component_systems(manifest: &mut crate::runtime::SystemManifest)
    where
        Self: Sized,
    {
        manifest.add_system::<(&mut Camera, Option<&Transform>)>(
            Phase::PreRender,
            "Camera pre-render preparation",
            |_, (camera, transform)| {
                profiling::scope!("Camera pre-render preparation");

                camera.update_render_target();

                camera.update_view_projection(
                    transform
                        .map(|t| t.local_to_world())
                        .unwrap_or(Mat4::IDENTITY),
                );

                camera.update_cam_bind_group();
            },
        );
    }
}
