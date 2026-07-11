//! Graphics APIs and rendering functionality

use alloc::sync::Arc;
use core::any::TypeId;
use std::sync::mpsc::{Receiver, Sender, channel};

pub use wutengine_graphics::*;
use wutengine_graphics::{material::Material, mesh::Mesh, renderpass::RenderPass};
use wutengine_math::Mat4;
use wutengine_util::InitOnce;

use crate::builtins::components::rendering::CameraId;

#[cfg(feature = "development_overlay")]
pub(crate) mod dev_overlays;

/// A single draw command submitted to the WutEngine graphics backend.
#[derive(Debug, Clone)]
pub struct DrawCommand {
    /// The camera this draw call applies to. If [None], renders on all cameras
    pub camera: Option<CameraId>,

    /// The mesh to render
    pub mesh: Arc<Mesh>,

    /// The material to render with
    pub material: Arc<Material>,

    /// The transform/model matrix to use
    pub transform: Mat4,
}

/// The global draw command queue
static DRAW_COMMAND_QUEUE: InitOnce<Sender<DrawCommand>> = InitOnce::new_checked();

/// Initializes the global draw command queue, and returns its receiving end.
pub(crate) fn initialize_command_queue() -> Receiver<DrawCommand> {
    let (send, recv) = channel::<DrawCommand>();

    InitOnce::init(&DRAW_COMMAND_QUEUE, send);

    recv
}

/// Submits a raw draw command to the command queue
#[inline(always)]
pub fn submit_raw_draw_command(command: DrawCommand) {
    DRAW_COMMAND_QUEUE.send(command).expect("Runtime stopped")
}

/// Submit a command to render the given mesh using the given material and model transform
/// matrix
pub fn render_mesh(mesh: Arc<Mesh>, material: Arc<Material>, transform: Mat4) {
    submit_raw_draw_command(DrawCommand {
        camera: None,
        mesh,
        material,
        transform,
    });
}

/// Metadata and info on a [RenderPass].
/// Construct with [Self::from_pass]
#[derive(derive_more::Debug)]
pub struct RenderPassInfo<T, D>
where
    D: ?Sized,
{
    /// The type of the pass
    pub type_id: TypeId,

    /// The name of the pass
    pub name: &'static str,

    /// The ordering of the pass relative to other passes. Higher is later
    pub order: u64,

    /// The constructor function that creates an instance of this pass
    #[debug(skip)]
    pub constructor: Arc<dyn Fn() -> Box<dyn RenderPass<T, D>> + Send + Sync>,
}

impl<T, D> Clone for RenderPassInfo<T, D>
where
    D: ?Sized,
{
    fn clone(&self) -> Self {
        Self {
            type_id: self.type_id,
            name: self.name,
            order: self.order,
            constructor: self.constructor.clone(),
        }
    }
}

impl<T, D> RenderPassInfo<T, D>
where
    D: ?Sized,
{
    /// Create a [RenderPassInfo] from an implementation of [RenderPass]
    pub(crate) fn from_pass<P: RenderPass<T, D>>() -> Self {
        Self {
            type_id: TypeId::of::<P>(),
            name: P::name(),
            order: P::order(),
            constructor: Arc::new(|| P::construct()),
        }
    }
}
