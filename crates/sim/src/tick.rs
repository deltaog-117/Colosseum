//! Fixed-timestep timing: how many simulation ticks a stretch of wall-clock
//! time is worth, independent of the frame rate.

mod fixed_step;
mod rate;

pub use fixed_step::{Advance, FixedStep, MAX_FRAME_TIME};
pub use rate::{TickRate, TickRateError};
