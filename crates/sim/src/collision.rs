//! Box collision: solids are axis-aligned boxes, and a moving box is swept
//! one axis at a time, so it can never tunnel through a solid at any speed.

mod aabb;
mod axis;
mod world;

pub use aabb::{Aabb, AabbError};
pub use axis::Axis3;
pub use world::{SKIN, Sweep, World};
