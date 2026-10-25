//! Match rules: where the player spawns, what the pickups give, and the
//! round clock with its score.

mod pickup;
mod round;
mod score;
mod spawn;
pub mod tuning;

pub use pickup::{Pickup, PickupKind};
pub use round::{Phase, Round, Transition};
pub use score::Score;
pub use spawn::{SpawnPoints, SpawnsError};
