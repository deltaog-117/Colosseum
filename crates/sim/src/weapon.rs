//! The rocket launcher: ammo, cooldown and the projectiles it releases,
//! flown through the box [`World`](crate::collision::World).

mod launcher;
mod projectile;
pub mod tuning;

pub use launcher::{Denied, Launcher};
pub use projectile::{Detonation, Projectile};
