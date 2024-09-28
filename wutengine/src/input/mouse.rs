//! Module for the mouse input handling logic.
//! In order to activate mouse input, use the [MouseInputPlugin] engine plugin.
//!
use crate::plugins::WutEnginePlugin;

/// The mouse input reader plugin.
/// Reads the raw mouse input from the WutEngine window handler
/// and delivers it to the InputHandler components in the world.
#[derive(Debug)]
pub struct MouseInputPlugin;

impl WutEnginePlugin for MouseInputPlugin {}
