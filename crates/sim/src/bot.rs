//! Computer-controlled opponents: a reflex [`Brain`] that turns what a bot
//! knows into the same [`Command`](crate::input::Command) a keyboard
//! produces, and the [`Bot`] that owns the body, launcher and score it
//! fights with.

mod agent;
mod brain;
pub mod tuning;

pub use agent::Bot;
pub use brain::{Brain, Target};
