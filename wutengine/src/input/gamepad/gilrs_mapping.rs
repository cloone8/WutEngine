use super::GamepadId;

impl From<gilrs::GamepadId> for GamepadId {
    fn from(value: gilrs::GamepadId) -> Self {
        Self(value)
    }
}
