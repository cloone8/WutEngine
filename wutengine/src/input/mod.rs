//! Main module containing the input handling logic for WutEngine

use wutengine_core::Component;

pub mod keyboard;
pub mod mouse;

/// The main input handler component. The various input-reading engine plugins will
/// inject their read inputs into each of these components before each Update iteration.
pub struct InputHandler {}

impl Component for InputHandler {}
