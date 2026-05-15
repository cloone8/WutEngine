//! Custom and built-in graphics rendering passes

use alloc::sync::Arc;
use core::any::Any;
use core::any::TypeId;
use core::ops::Deref;
use std::collections::BTreeSet;
use std::sync::RwLock;

use crate::builtins::components::rendering::Camera;
use crate::builtins::components::rendering::CameraRenderPass;
use crate::util::InitOnce;

#[derive(derive_more::Debug, Clone)]
pub struct RenderPassInfo {
    pub(crate) type_id: TypeId,
    pub(crate) name: &'static str,
    pub(crate) order: u64,

    #[debug(skip)]
    pub(crate) constructor: Arc<dyn Fn() -> Box<dyn RenderPass> + Send + Sync>,
}

impl RenderPassInfo {
    pub fn from_pass<T: RenderPass>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            name: T::name(),
            order: T::order(),
            constructor: Arc::new(|| T::construct()),
        }
    }
}

impl PartialEq for RenderPassInfo {
    fn eq(&self, other: &Self) -> bool {
        self.order.eq(&other.order)
    }
}

impl Eq for RenderPassInfo {}

impl PartialOrd for RenderPassInfo {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RenderPassInfo {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.order.cmp(&other.order)
    }
}

pub trait RenderPass: Send + Sync + Any {
    fn name() -> &'static str
    where
        Self: Sized;

    fn order() -> u64
    where
        Self: Sized;

    fn construct() -> Box<dyn RenderPass>
    where
        Self: Sized;

    fn execute(&mut self, cmd: &mut wgpu::CommandEncoder, camera: &Camera);
}

#[derive(Debug, Clone)]
pub struct ColorPass;

impl ColorPass {
    pub const ORDER: u64 = u64::MAX / 2;
}

impl RenderPass for ColorPass {
    fn name() -> &'static str
    where
        Self: Sized,
    {
        "Color"
    }

    fn order() -> u64 {
        Self::ORDER
    }

    fn construct() -> Box<dyn RenderPass>
    where
        Self: Sized,
    {
        Box::new(ColorPass)
    }

    fn execute(&mut self, cmd: &mut wgpu::CommandEncoder, camera: &Camera) {
        log::trace!("Running color pass");

        let target_tex = camera
            .get_render_target()
            .expect("Missing render target in color pass");

        let target_view = target_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let mut render_pass = cmd.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Color"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &target_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: camera.background.to_wgpu_load_op(),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        camera
            .set_camera_bind_group_on_pass(&mut render_pass)
            .expect("Failed to set camera bind group");
    }
}

pub(crate) fn sync_camera_passes(camera: &mut Camera, passes: &[RenderPassInfo]) {
    profiling::scope!("Synchronize passes");

    let cam_id = camera.get_id();
    let mut passes_changed = false;

    // Remove all passes not present in the global runtime
    camera.render_passes.retain(|camera_pass| {
        let should_keep = passes
            .iter()
            .any(|runtime_pass| runtime_pass.type_id == camera_pass.type_id);

        if !should_keep {
            log::debug!("Removing pass {} from camera {}", camera_pass.name, cam_id);
            passes_changed = true;
        }

        should_keep
    });

    // Add passes present in the runtime, but missing in the camera
    for pass in passes {
        if !camera
            .render_passes
            .iter()
            .any(|camera_pass| camera_pass.type_id == pass.type_id)
        {
            log::debug!("Adding pass {} to camera {}", pass.name, cam_id);

            camera.render_passes.push(CameraRenderPass {
                type_id: pass.type_id,
                name: pass.name,
                order: pass.order,
                pass: (pass.constructor)(),
            });

            passes_changed = true;
        }
    }

    if passes_changed {
        camera.render_passes.sort_by(|a, b| a.order.cmp(&b.order));
    }
}
