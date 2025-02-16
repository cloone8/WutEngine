use thiserror::Error;

use super::{
    FromGilrsAxisErr, FromGilrsButtonErr, GamepadButton, GamepadButtonValue, GamepadId,
    PartialGamepadAxis, PartialGamepadAxisValue,
};

impl From<gilrs::GamepadId> for GamepadId {
    fn from(value: gilrs::GamepadId) -> Self {
        Self(value)
    }
}

pub(super) fn is_button_event(event: &gilrs::EventType) -> bool {
    matches!(
        event,
        gilrs::EventType::ButtonPressed(_, _)
            | gilrs::EventType::ButtonRepeated(_, _)
            | gilrs::EventType::ButtonReleased(_, _)
            | gilrs::EventType::ButtonChanged(_, _, _)
    )
}

pub(super) fn is_axis_event(event: &gilrs::EventType) -> bool {
    matches!(event, gilrs::EventType::AxisChanged(_, _, _))
}

#[derive(Debug, Error)]
pub(super) enum ButtonMapErr {
    #[error("Unknown button")]
    UnknownButton(#[from] FromGilrsButtonErr),
}

pub(super) fn get_button_event_button_and_value(
    event: &gilrs::EventType,
) -> Result<(GamepadButton, GamepadButtonValue), ButtonMapErr> {
    debug_assert!(
        is_button_event(event),
        "Non-button event given: {:?}",
        event
    );

    match event {
        gilrs::EventType::ButtonPressed(button, _) => {
            let button = GamepadButton::try_from(*button)?;
            Ok((button, GamepadButtonValue::PRESSED))
        }
        gilrs::EventType::ButtonRepeated(button, _) => {
            let button = GamepadButton::try_from(*button)?;
            Ok((button, GamepadButtonValue::PRESSED))
        }
        gilrs::EventType::ButtonReleased(button, _) => {
            let button = GamepadButton::try_from(*button)?;
            Ok((button, GamepadButtonValue::NOT_PRESSED))
        }
        gilrs::EventType::ButtonChanged(button, val, _) => {
            let button = GamepadButton::try_from(*button)?;
            Ok((button, GamepadButtonValue::new_continuous(*val)))
        }
        _ => unreachable!(),
    }
}

#[derive(Debug, Error)]
pub(super) enum AxisMapErr {
    #[error("Unknown axis")]
    UnknownButton(#[from] FromGilrsAxisErr),
}

pub(super) fn get_axis_event_axis_and_value(
    event: &gilrs::EventType,
) -> Result<(PartialGamepadAxis, PartialGamepadAxisValue), AxisMapErr> {
    debug_assert!(is_axis_event(event), "Non-axis event given: {:?}", event);

    match event {
        gilrs::EventType::AxisChanged(axis, val, _) => {
            let partial_axis = PartialGamepadAxis::try_from(*axis)?;
            Ok((partial_axis, PartialGamepadAxisValue::new(*val)))
        }
        _ => unreachable!(),
    }
}
