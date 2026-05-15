use alloc::sync::Arc;

use crate::builtins::components::CameraId;

use super::DRAW_COMMAND_QUEUE;
use super::material::Material;
use super::mesh::Mesh;

/// Submits a raw draw command to the command queue
#[inline(always)]
pub fn submit_raw_draw_command(command: DrawCommand) {
    DRAW_COMMAND_QUEUE.send(command).expect("Runtime stopped")
}

pub fn render_mesh(mesh: Arc<Mesh>, material: Arc<Material>) {
    submit_raw_draw_command(DrawCommand {
        camera: None,
        mesh,
        material,
    });
}

/// A single draw command submitted to the WutEngine graphics backend.
#[derive(Debug, Clone)]
pub struct DrawCommand {
    /// The camera this draw call applies to. If [None], renders on all cameras
    pub camera: Option<CameraId>,

    pub mesh: Arc<Mesh>,

    pub material: Arc<Material>,
}
