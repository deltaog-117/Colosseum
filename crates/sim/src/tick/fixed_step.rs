use std::time::Duration;

use super::rate::TickRate;

/// Longest slice of wall-clock time accepted per frame. After a stall (a
/// debugger pause, a dragged window) the sim drops the excess instead of
/// running a burst of catch-up ticks that makes the next frame even slower.
pub const MAX_FRAME_TIME: Duration = Duration::from_millis(250);

/// Result of feeding one frame's elapsed time to a [`FixedStep`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Advance {
    /// Whole simulation ticks to run this frame.
    pub ticks: u32,
    /// Progress into the next tick, in `0.0..=1.0`, for render interpolation.
    pub alpha: f32,
}

/// Accumulator that converts variable frame times into fixed ticks.
#[derive(Debug, Clone)]
pub struct FixedStep {
    step: Duration,
    // Invariant: always strictly less than `step` between calls.
    pending: Duration,
}

impl FixedStep {
    /// Starts with no pending time.
    pub fn new(rate: TickRate) -> Self {
        Self {
            step: rate.step(),
            pending: Duration::ZERO,
        }
    }

    /// Adds `elapsed` (capped at [`MAX_FRAME_TIME`]) and reports how many
    /// ticks are now due.
    pub fn advance(&mut self, elapsed: Duration) -> Advance {
        self.pending += elapsed.min(MAX_FRAME_TIME);

        let due = self.pending.as_nanos() / self.step.as_nanos();
        // Bounded by MAX_FRAME_TIME / step + 1, so the fallback is unreachable.
        let ticks = u32::try_from(due).unwrap_or(u32::MAX);
        self.pending -= self.step * ticks;

        Advance {
            ticks,
            alpha: (self.pending.as_secs_f64() / self.step.as_secs_f64()) as f32,
        }
    }

    /// Time carried over to the next call; always less than one tick.
    pub fn pending(&self) -> Duration {
        self.pending
    }

    /// Length of one tick.
    pub fn step(&self) -> Duration {
        self.step
    }
}
