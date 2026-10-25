//! Box collision: solids are axis-aligned boxes, and a moving box is swept
//! one axis at a time, so it can never tunnel through a solid at any speed.
//! Thin things such as projectiles are cast as segments instead.

mod aabb;
mod axis;
mod ray;
mod world;

pub use aabb::{Aabb, AabbError};
pub use axis::Axis3;
pub use ray::{RayHit, cast_boxes};
pub use world::{SKIN, Sweep, World};
