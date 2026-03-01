use crate::builtins::components::CameraId;

use super::DRAW_COMMAND_QUEUE;

/// Submits a raw draw command to the command queue
#[inline(always)]
pub fn submit_raw_draw_command(command: DrawCommand) {
    DRAW_COMMAND_QUEUE.send(command).expect("Runtime stopped")
}

/// A single draw command submitted to the WutEngine graphics backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawCommand {
    /// The camera this draw call applies to. If [None], renders on all cameras
    pub camera: Option<CameraId>,
}

impl DrawCommand {
    pub(crate) const NOOP: Self = Self { camera: None };
}

impl Default for DrawCommand {
    #[inline]
    fn default() -> Self {
        Self::NOOP
    }
}
