use super::button::{Button, ButtonSet};
use super::command::{Axis, Command, Look};

/// Bound on accumulated mouse movement per command. Far beyond any real
/// flick; it only keeps the sum finite if a device misbehaves.
pub const LOOK_LIMIT: f32 = 1_000_000.0;

/// A device-neutral input event, translated by the code that owns the devices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// A button went down. Repeats while held are harmless.
    Press(Button),
    /// A button went up.
    Release(Button),
    /// Raw mouse movement. Non-finite values are ignored.
    MouseMotion {
        /// Horizontal counts; positive is right.
        dx: f32,
        /// Vertical counts; positive is down.
        dy: f32,
    },
    /// The window lost focus, so no button can be assumed held any more.
    FocusLost,
}

/// Collects events between ticks and hands out one [`Command`] per tick.
#[derive(Debug, Clone, Default)]
pub struct InputState {
    held: ButtonSet,
    // Pressed since the last command, so a tap shorter than a tick still counts.
    pressed: ButtonSet,
    look_dx: f32,
    look_dy: f32,
}

impl InputState {
    /// Starts with nothing held and no pending movement.
    pub fn new() -> Self {
        Self::default()
    }

    /// Folds one event into the state.
    pub fn apply(&mut self, event: Event) {
        match event {
            Event::Press(button) => {
                self.held.insert(button);
                self.pressed.insert(button);
            }
            Event::Release(button) => self.held.remove(button),
            Event::MouseMotion { dx, dy } => {
                if dx.is_finite() && dy.is_finite() {
                    self.look_dx = (self.look_dx + dx).clamp(-LOOK_LIMIT, LOOK_LIMIT);
                    self.look_dy = (self.look_dy + dy).clamp(-LOOK_LIMIT, LOOK_LIMIT);
                }
            }
            Event::FocusLost => {
                self.held.clear();
                self.pressed.clear();
                self.look_dx = 0.0;
                self.look_dy = 0.0;
            }
        }
    }

    /// Builds the command for one tick and drains what is one-shot: mouse
    /// movement and presses. Held buttons stay, so when a frame runs several
    /// ticks only the first carries the look delta.
    pub fn take_command(&mut self) -> Command {
        let active = |button| self.held.contains(button) || self.pressed.contains(button);
        let command = Command::new(
            Axis::from_pair(active(Button::Back), active(Button::Forward)),
            Axis::from_pair(active(Button::Left), active(Button::Right)),
            active(Button::Jump),
            active(Button::Fire),
            Look::new(self.look_dx, self.look_dy),
        );
        self.pressed.clear();
        self.look_dx = 0.0;
        self.look_dy = 0.0;
        command
    }
}
