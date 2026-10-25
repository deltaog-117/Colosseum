//! Frame pacing: turns wall-clock instants into fixed simulation ticks.

use std::time::Instant;

use sim::tick::{Advance, FixedStep, TickRate};

pub(crate) struct GameLoop {
    step: FixedStep,
    last_frame: Instant,
}

impl GameLoop {
    pub(crate) fn new(rate: TickRate, now: Instant) -> Self {
        Self {
            step: FixedStep::new(rate),
            last_frame: now,
        }
    }

    /// `now` is a parameter, not read here, so tests can drive the clock.
    pub(crate) fn frame(&mut self, now: Instant) -> Advance {
        // Saturating: the OS clock is monotonic, but a zero elapsed time is a
        // safer answer than a panic if an `Instant` ever arrives out of order.
        let elapsed = now.saturating_duration_since(self.last_frame);
        self.last_frame = now;
        self.step.advance(elapsed)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn one_second_of_frames_runs_one_second_of_ticks() {
        let start = Instant::now();
        let mut game_loop = GameLoop::new(TickRate::DEFAULT, start);

        let mut ticks = 0;
        for frame in 1..=60 {
            let now = start + Duration::from_millis(1000 * frame / 60);
            ticks += game_loop.frame(now).ticks;
        }

        // 120 ticks/s, give or take the one tick still pending.
        assert!((119..=120).contains(&ticks), "ran {ticks} ticks");
    }

    #[test]
    fn out_of_order_instant_runs_no_ticks() {
        let start = Instant::now();
        let mut game_loop = GameLoop::new(TickRate::DEFAULT, start + Duration::from_secs(1));
        assert_eq!(game_loop.frame(start).ticks, 0);
    }
}
