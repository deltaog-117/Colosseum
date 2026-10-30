//! Navigation: a grid of the spots a body can stand on, and A* paths across
//! it, so a bot can reach what it cannot see.

mod grid;
mod path;
mod search;
pub mod tuning;

pub use grid::{Cell, NavError, NavGrid};
pub use path::Path;
