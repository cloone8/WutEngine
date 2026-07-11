//! WutEngine-to-egui input mapping

use wutengine_input::keyboard;
use wutengine_math::Vec2;

use crate::{utils, wutengine_to_egui_key};

/// Checks a given mouse button for pressed/released events, and adds
/// them to the events list if applicable
fn add_mouse_button(
    button: u32,
    egui_button: egui::PointerButton,
    pos: egui::Pos2,
    modifiers: egui::Modifiers,
    events: &mut Vec<egui::Event>,
) {
    if wutengine_input::mouse::button_pressed(None, button) {
        events.push(egui::Event::PointerButton {
            pos,
            button: egui_button,
            pressed: true,
            modifiers,
        });
    } else if wutengine_input::mouse::button_released(None, button) {
        events.push(egui::Event::PointerButton {
            pos,
            button: egui_button,
            pressed: false,
            modifiers,
        });
    }
}

/// Checks for mouse events for the given window
pub(crate) fn add_mouse_events(
    window: wutengine_input::WindowIdentifier,
    modifiers: egui::Modifiers,
    scale_factor: f32,
    events: &mut Vec<egui::Event>,
) {
    let mouse_raw = wutengine_input::mouse::pos_delta(None);

    if mouse_raw != Vec2::ZERO {
        events.push(egui::Event::MouseMoved(utils::to_egui_vec2(mouse_raw)));
    }

    if let Some((pointer_window, pointer_pos)) = wutengine_input::mouse::screen_pos(None)
        && pointer_window == window
    {
        let pointer_pos = utils::to_egui_pos2(pointer_pos, scale_factor);

        if mouse_raw != Vec2::ZERO {
            events.push(egui::Event::PointerMoved(pointer_pos));
        }

        add_mouse_button(
            wutengine_input::mouse::BUTTON_LEFT,
            egui::PointerButton::Primary,
            pointer_pos,
            modifiers,
            events,
        );
        add_mouse_button(
            wutengine_input::mouse::BUTTON_RIGHT,
            egui::PointerButton::Secondary,
            pointer_pos,
            modifiers,
            events,
        );
        add_mouse_button(
            wutengine_input::mouse::BUTTON_MIDDLE,
            egui::PointerButton::Middle,
            pointer_pos,
            modifiers,
            events,
        );
        add_mouse_button(
            wutengine_input::mouse::BUTTON_BACK,
            egui::PointerButton::Extra1,
            pointer_pos,
            modifiers,
            events,
        );
        add_mouse_button(
            wutengine_input::mouse::BUTTON_FORWARD,
            egui::PointerButton::Extra2,
            pointer_pos,
            modifiers,
            events,
        );
    }

    let scroll_raw = wutengine_input::mouse::scroll_delta(None);

    if scroll_raw != Vec2::ZERO {
        events.push(egui::Event::MouseWheel {
            unit: egui::MouseWheelUnit::Line,
            delta: utils::to_egui_vec2(scroll_raw),
            phase: egui::TouchPhase::Move,
            modifiers,
        });
    }
}

/// Gathers the currently held modifier keys
fn gather_modifiers() -> egui::Modifiers {
    use wutengine_input::keyboard::key_held;

    const IS_MACOS: bool = cfg!(any(target_os = "macos", target_os = "ios"));

    let alt = key_held(None, keyboard::Key::AltLeft) || key_held(None, keyboard::Key::AltRight);

    let ctrl =
        key_held(None, keyboard::Key::ControlLeft) || key_held(None, keyboard::Key::ControlRight);

    let shift =
        key_held(None, keyboard::Key::ShiftLeft) || key_held(None, keyboard::Key::ShiftRight);

    let mac_cmd = IS_MACOS
        && (key_held(None, keyboard::Key::SuperLeft) || key_held(None, keyboard::Key::SuperRight));

    let command = if IS_MACOS { mac_cmd } else { ctrl };

    egui::Modifiers {
        alt,
        ctrl,
        shift,
        mac_cmd,
        command,
    }
}

/// Winit also sends control characters as text, so we ignore those
/// Ignore those.
/// We also ignore '\r', '\n', '\t'.
/// Newlines are handled by the `Key::Enter` event.
fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !is_in_private_use_area && !chr.is_ascii_control()
}

/// Adds keyboard events to the given events list. Returns the currently held modifier keys
pub(crate) fn add_keyboard_events(events: &mut Vec<egui::Event>) -> egui::Modifiers {
    let modifiers = gather_modifiers();

    let logical_inputs = wutengine_input::keyboard::logical_inputs(None);

    for logical_input in logical_inputs {
        let event = match logical_input {
            keyboard::LogicalInput::Pressed(logical_key) => {
                let Some(key) = wutengine_to_egui_key(logical_key) else {
                    continue;
                };

                egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers,
                }
            }
            keyboard::LogicalInput::Text(txt) => {
                let is_cmd = modifiers.ctrl || modifiers.command || modifiers.mac_cmd;

                if is_cmd || txt.chars().any(|c| !is_printable_char(c)) {
                    continue;
                }

                egui::Event::Text(txt)
            }
            keyboard::LogicalInput::Released(logical_key) => {
                let Some(key) = wutengine_to_egui_key(logical_key) else {
                    continue;
                };

                egui::Event::Key {
                    key,
                    physical_key: None,
                    pressed: false,
                    repeat: false,
                    modifiers,
                }
            }
        };

        events.push(event);
    }

    modifiers
}
