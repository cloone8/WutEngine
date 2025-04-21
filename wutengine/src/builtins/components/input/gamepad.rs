use glam::Vec2;

use crate::input::gamepad::{GamepadAxis, GamepadButton, GamepadButtonValue};

use super::{InputHandler, InputState};

/// The gamepad input state from an [InputHandler]
#[repr(transparent)]
pub struct InputHandlerGamepad<'a> {
    pub(super) handler: &'a InputHandler,
}

#[profiling::all_functions]
impl InputHandlerGamepad<'_> {
    fn is_down_from_state(state: &InputState, button: GamepadButton) -> bool {
        state
            .gamepads
            .values()
            .filter(|gp| gp.connected)
            .any(|gp| gp.buttons[button as usize].is_pressed())
    }

    /// Returns whether the given [GamepadButton] is currently down
    pub fn is_down(&self, button: GamepadButton) -> bool {
        Self::is_down_from_state(&self.handler.cur, button)
    }

    /// Returns whether the given [GamepadButton] is currently up
    pub fn is_up(&self, button: GamepadButton) -> bool {
        !self.is_down(button)
    }

    /// Returns whether the given [GamepadButton] changed from released to pressed this frame
    pub fn pressed_this_frame(&self, button: GamepadButton) -> bool {
        Self::is_down_from_state(&self.handler.cur, button)
            && !Self::is_down_from_state(&self.handler.prev, button)
    }

    /// Returns whether the given [GamepadButton] changed from pressed to released this frame
    pub fn released_this_frame(&self, button: GamepadButton) -> bool {
        !Self::is_down_from_state(&self.handler.cur, button)
            && Self::is_down_from_state(&self.handler.prev, button)
    }

    /// Returns the current value for the given [GamepadButton]. The value will be
    pub fn button_value(&self, button: GamepadButton) -> GamepadButtonValue {
        self.handler
            .cur
            .gamepads
            .values()
            .map(|v| v.buttons[button as usize])
            .max()
            .unwrap_or(GamepadButtonValue::NOT_PRESSED)
    }

    /// Returns the current X/Y values for the given [GamepadAxis]
    pub fn axis_value(&self, axis: GamepadAxis) -> Vec2 {
        let non_neutral: Vec<Vec2> = self
            .handler
            .cur
            .gamepads
            .values()
            .map(|v| v.axes[axis as usize])
            .filter(|val| !val.is_neutral())
            .map(|val| val.value())
            .collect();

        if non_neutral.is_empty() {
            Vec2::ZERO
        } else {
            non_neutral.iter().sum::<Vec2>() / (non_neutral.len() as f32)
        }
    }
}
