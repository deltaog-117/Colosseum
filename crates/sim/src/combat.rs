//! What a rocket's blast does to a body: damage that falls off with
//! distance, knockback away from the blast, and the health that takes it.

mod blast;
mod health;
pub mod tuning;

pub use blast::{Impact, impact};
pub use health::{Health, Outcome};
