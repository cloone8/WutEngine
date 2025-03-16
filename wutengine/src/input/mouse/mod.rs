//! Module for the mouse input handling logic.
//! In order to activate mouse input, use the [MouseInputPlugin] engine plugin.
//!

use std::collections::HashSet;

use glam::Vec2;
use winit::event::{ButtonId, DeviceEvent, MouseScrollDelta};

use crate::plugins::WutEnginePlugin;
use crate::time::Time;

/// The left/primary mouse button
pub const BUTTON_LEFT: u32 = 0;

/// The right/secondary mouse button
pub const BUTTON_RIGHT: u32 = 1;

/// The middle mouse button (scroll wheel)
pub const BUTTON_MIDDLE: u32 = 2;

/// The back thumb button
pub const BUTTON_THUMB_BACKWARD: u32 = 3;

/// The forward thumb button
pub const BUTTON_THUMB_FORWARD: u32 = 4;

/// The mouse input reader plugin.
/// Reads the raw mouse input from the WutEngine window handler
/// and delivers it to the InputHandler components in the world.
#[derive(Debug)]
pub struct MouseInputPlugin {
    /// The movement delta this frame
    pub(crate) mouse_delta: Vec2,

    /// The scroll delta this frame
    pub(crate) scroll_delta: Vec2,

    /// The buttons that were in a pressed state this frame
    pub(crate) buttons: HashSet<ButtonId>,

    had_event: bool,
    last_frame: usize,
}

impl Default for MouseInputPlugin {
    fn default() -> Self {
        Self {
            mouse_delta: Vec2::ZERO,
            scroll_delta: Vec2::ZERO,
            last_frame: usize::MAX,
            had_event: false,
            buttons: HashSet::default(),
        }
    }
}

impl WutEnginePlugin for MouseInputPlugin {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn pre_update(&mut self, _context: &mut crate::plugins::Context) {
        if !self.had_event {
            self.mouse_delta = Vec2::ZERO;
            self.scroll_delta = Vec2::ZERO;
        }

        self.had_event = false;
    }

    fn on_device_event(
        &mut self,
        _device: winit::event::DeviceId,
        event: &DeviceEvent,
        _context: &mut crate::plugins::Context,
    ) {
        self.check_reset();

        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.had_event = true;
                self.mouse_delta += Vec2::new(delta.0 as f32, delta.1 as f32);
            }
            DeviceEvent::MouseWheel { delta } => {
                self.had_event = true;
                self.scroll_delta += match delta {
                    MouseScrollDelta::LineDelta(x, y) => Vec2::new(*x, *y),
                    MouseScrollDelta::PixelDelta(phys_pos) => {
                        Vec2::new(phys_pos.x as f32, phys_pos.y as f32)
                    }
                };
            }
            DeviceEvent::Button { button, state } => {
                log::info!("{}", button);
                self.had_event = true;
                match state.is_pressed() {
                    true => self.buttons.insert(*button),
                    false => self.buttons.remove(button),
                };
            }
            _ => {}
        };
    }
}

impl MouseInputPlugin {
    fn check_reset(&mut self) {
        let cur_frame = Time::get().frame_num;
        if cur_frame != self.last_frame {
            // Reset the delta every frame
            self.last_frame = cur_frame;
            self.mouse_delta = Vec2::ZERO;
            self.scroll_delta = Vec2::ZERO;
        }
    }
}
