//! Custom and built-in graphics rendering passes

use alloc::sync::Arc;
use core::any::Any;
use core::any::TypeId;

use crate::builtins::components::rendering::Camera;
use crate::builtins::components::rendering::CameraRenderPass;

mod color;
pub use color::*;

use super::DrawCommand;

/// Metadata and info on a [RenderPass].
/// Construct with [Self::from_pass]
#[derive(derive_more::Debug, Clone)]
pub struct RenderPassInfo {
    /// The type of the pass
    pub(crate) type_id: TypeId,

    /// The name of the pass
    pub(crate) name: &'static str,

    /// The ordering of the pass relative to other passes. Higher is later
    pub(crate) order: u64,

    /// The constructor function that creates an instance of this pass
    #[debug(skip)]
    pub(crate) constructor: Arc<dyn Fn() -> Box<dyn RenderPass> + Send + Sync>,
}

impl RenderPassInfo {
    /// Create a [RenderPassInfo] from an implementation of [RenderPass]
    pub fn from_pass<T: RenderPass>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            name: T::name(),
            order: T::order(),
            constructor: Arc::new(|| T::construct()),
        }
    }
}

/// A render pass implementation
pub trait RenderPass: Send + Sync + Any {
    /// The name of the pass
    fn name() -> &'static str
    where
        Self: Sized;

    /// The order of the pass, relative to other passes. Higher is later.
    /// Most built-in passes have an `ORDER` const that you can base the return value
    /// of your custom pass on
    fn order() -> u64
    where
        Self: Sized;

    /// Construct a default version of this pass. Called once per camera
    fn construct() -> Box<dyn RenderPass>
    where
        Self: Sized;

    /// Run the pass for the given camera. Commands should be placed in `cmd`
    fn execute(
        &mut self,
        cmd: &mut wgpu::CommandEncoder,
        camera: &Camera,
        draw_commands: &[DrawCommand],
    );
}

/// Synchronize the passes on the camera with the passes in `passes`, deleting
/// any passes not in `passes`, and adding missing onces
pub(crate) fn sync_camera_passes(camera: &mut Camera, passes: &[RenderPassInfo]) {
    profiling::scope!("Synchronize passes");

    let cam_id = camera.get_id();

    // Remove all passes not present in the global runtime
    camera.render_passes.retain(|camera_pass| {
        let should_keep = passes
            .iter()
            .any(|runtime_pass| runtime_pass.type_id == camera_pass.type_id);

        if !should_keep {
            log::debug!("Removing pass {} from camera {}", camera_pass.name, cam_id);
        }

        should_keep
    });

    // Add passes present in the runtime, but missing in the camera
    let mut passes_added = false;

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

            passes_added = true;
        }
    }

    if passes_added {
        camera.render_passes.sort_by_key(|p| p.order);
    }
}
