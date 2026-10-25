//! Quake-style player movement: ground friction, strafe acceleration, air
//! control and jumping, resolved against the box [`World`](crate::collision::World).

mod player;
mod pose;
pub mod tuning;

pub use player::{Player, SpawnError};
pub use pose::Pose;
