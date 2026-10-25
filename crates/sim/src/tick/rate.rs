use std::fmt;
use std::time::Duration;

const MAX_HZ: u32 = 1000;

/// Simulation ticks per second, always within `1..=1000`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickRate(u32);

/// The requested tick rate is outside the supported range.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TickRateError(u32);

impl fmt::Display for TickRateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tick rate {} Hz is outside 1..={MAX_HZ}", self.0)
    }
}

impl std::error::Error for TickRateError {}

impl TickRate {
    /// 120 Hz divides evenly into 60, 120 and 240 Hz displays, so ticks and
    /// frames line up without judder on common monitors.
    pub const DEFAULT: Self = Self(120);

    /// Builds a rate from ticks per second.
    ///
    /// # Errors
    ///
    /// Returns [`TickRateError`] if `hz` is zero or above 1000.
    pub fn try_new(hz: u32) -> Result<Self, TickRateError> {
        if (1..=MAX_HZ).contains(&hz) {
            Ok(Self(hz))
        } else {
            Err(TickRateError(hz))
        }
    }

    /// Ticks per second.
    pub fn hz(self) -> u32 {
        self.0
    }

    /// Length of one tick, rounded down to whole nanoseconds. The rounded
    /// value is the definition of a tick; no wall-clock drift accumulates
    /// because the accumulator is counted in these same steps.
    pub fn step(self) -> Duration {
        Duration::from_nanos(1_000_000_000 / u64::from(self.0))
    }
}

impl Default for TickRate {
    fn default() -> Self {
        Self::DEFAULT
    }
}
