//! Input commands: devices are reduced to one small [`Command`] per tick, so
//! the simulation never reads a keyboard or mouse directly.

mod button;
mod command;
mod state;

pub use button::Button;
pub use command::{Axis, Command, Look};
pub use state::{Event, InputState, LOOK_LIMIT};
