use crate::input::gamepad::{GamepadButton, GamepadButtonValue};

use super::{InputHandler, InputState};

#[repr(transparent)]
pub struct InputHandlerGamepad<'a> {
    pub(super) handler: &'a InputHandler,
}

impl InputHandlerGamepad<'_> {
    fn is_down_from_state(state: &InputState, button: GamepadButton) -> bool {
        state
            .gamepads
            .values()
            .filter(|gp| gp.connected)
            .any(|gp| gp.buttons[button as usize].is_pressed())
    }

    pub fn is_down(&self, button: GamepadButton) -> bool {
        Self::is_down_from_state(&self.handler.cur, button)
    }

    pub fn is_up(&self, button: GamepadButton) -> bool {
        !self.is_down(button)
    }

    pub fn pressed_this_frame(&self, button: GamepadButton) -> bool {
        Self::is_down_from_state(&self.handler.cur, button)
            && !Self::is_down_from_state(&self.handler.prev, button)
    }

    pub fn released_this_frame(&self, button: GamepadButton) -> bool {
        !Self::is_down_from_state(&self.handler.cur, button)
            && Self::is_down_from_state(&self.handler.prev, button)
    }

    pub fn button_value(&self, button: GamepadButton) -> GamepadButtonValue {
        self.handler
            .cur
            .gamepads
            .values()
            .map(|v| v.buttons[button as usize])
            .max()
            .unwrap_or(GamepadButtonValue::NOT_PRESSED)
    }
}
