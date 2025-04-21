use glam::Vec2;

use super::InputHandler;

/// The mouse input state from an [InputHandler]
#[repr(transparent)]
pub struct InputHandlerMouse<'a> {
    pub(super) handler: &'a InputHandler,
}

#[profiling::all_functions]
impl InputHandlerMouse<'_> {
    /// The mouse movement delta this frame
    pub fn frame_delta(&self) -> Vec2 {
        self.handler.cur.mouse_delta
    }

    /// The mouse scroll delta this frame
    pub fn frame_scroll(&self) -> Vec2 {
        self.handler.cur.mouse_scroll_delta
    }

    /// Returns whether the mouse button with the given index is currently held down
    pub fn is_down(&self, button: u32) -> bool {
        self.handler.cur.mouse_pressed_buttons.contains(&button)
    }

    /// Returns whether the mouse button with the given index is currenty not held down
    pub fn is_up(&self, button: u32) -> bool {
        !self.is_down(button)
    }

    /// Returns whether the mouse button with the given index was
    /// pressed down this frame.
    pub fn pressed_this_frame(&self, button: u32) -> bool {
        self.handler.cur.mouse_pressed_buttons.contains(&button)
            && !self.handler.prev.mouse_pressed_buttons.contains(&button)
    }

    /// Returns whether the mouse button with the given index was
    /// released this frame.
    pub fn released_this_frame(&self, button: u32) -> bool {
        (!self.handler.cur.mouse_pressed_buttons.contains(&button))
            && self.handler.prev.mouse_pressed_buttons.contains(&button)
    }
}
