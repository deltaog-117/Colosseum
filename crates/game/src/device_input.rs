//! Translates winit device events into the simulation's input events.

use sim::input::{Button, Event};
use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

/// Physical keys, not layout-dependent ones, so WASD keeps its shape on AZERTY.
fn button_for(code: KeyCode) -> Option<Button> {
    match code {
        KeyCode::KeyW => Some(Button::Forward),
        KeyCode::KeyS => Some(Button::Back),
        KeyCode::KeyA => Some(Button::Left),
        KeyCode::KeyD => Some(Button::Right),
        KeyCode::Space => Some(Button::Jump),
        _ => None,
    }
}

/// `None` for keys the game does not bind.
pub(crate) fn key_event(key: PhysicalKey, state: ElementState) -> Option<Event> {
    let PhysicalKey::Code(code) = key else {
        return None;
    };
    let button = button_for(code)?;
    Some(match state {
        ElementState::Pressed => Event::Press(button),
        ElementState::Released => Event::Release(button),
    })
}

/// `delta` is winit's raw motion in device counts.
pub(crate) fn mouse_motion(delta: (f64, f64)) -> Event {
    // Counts are small; f32 holds them exactly, and the sim drops non-finite values.
    Event::MouseMotion {
        dx: delta.0 as f32,
        dy: delta.1 as f32,
    }
}

#[cfg(test)]
mod tests {
    use winit::keyboard::NativeKeyCode;

    use super::*;

    fn press(code: KeyCode) -> Option<Event> {
        key_event(PhysicalKey::Code(code), ElementState::Pressed)
    }

    #[test]
    fn wasd_and_space_are_bound() {
        assert_eq!(press(KeyCode::KeyW), Some(Event::Press(Button::Forward)));
        assert_eq!(press(KeyCode::KeyS), Some(Event::Press(Button::Back)));
        assert_eq!(press(KeyCode::KeyA), Some(Event::Press(Button::Left)));
        assert_eq!(press(KeyCode::KeyD), Some(Event::Press(Button::Right)));
        assert_eq!(press(KeyCode::Space), Some(Event::Press(Button::Jump)));
    }

    #[test]
    fn release_maps_to_release() {
        let event = key_event(PhysicalKey::Code(KeyCode::KeyW), ElementState::Released);
        assert_eq!(event, Some(Event::Release(Button::Forward)));
    }

    #[test]
    fn unbound_keys_are_ignored() {
        assert_eq!(press(KeyCode::KeyQ), None);
        let unidentified = PhysicalKey::Unidentified(NativeKeyCode::Unidentified);
        assert_eq!(key_event(unidentified, ElementState::Pressed), None);
    }

    #[test]
    fn mouse_delta_keeps_its_signs() {
        assert_eq!(
            mouse_motion((3.0, -2.0)),
            Event::MouseMotion { dx: 3.0, dy: -2.0 }
        );
    }
}
