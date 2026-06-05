use alloc::sync::Arc;
use glam::Mat4;

use crate::builtins::components::rendering::CameraId;

use super::DRAW_COMMAND_QUEUE;
use super::material::Material;
use super::mesh::Mesh;

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
