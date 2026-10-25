use std::time::Duration;

use proptest::prelude::*;
use sim::tick::{FixedStep, MAX_FRAME_TIME, TickRate};

fn frame_times() -> impl Strategy<Value = Vec<Duration>> {
    // Up to 10 s per frame, so both normal frames and clamped stalls occur.
    prop::collection::vec(
        (0u64..10_000_000_000).prop_map(Duration::from_nanos),
        0..200,
    )
}

proptest! {
    #[test]
    fn alpha_stays_in_unit_range(hz in 1u32..=1000, frames in frame_times()) {
        let mut step = FixedStep::new(TickRate::try_new(hz)?);
        for elapsed in frames {
            let alpha = step.advance(elapsed).alpha;
            prop_assert!((0.0..=1.0).contains(&alpha));
        }
    }

    #[test]
    fn pending_is_always_below_one_step(hz in 1u32..=1000, frames in frame_times()) {
        let mut step = FixedStep::new(TickRate::try_new(hz)?);
        for elapsed in frames {
            step.advance(elapsed);
            prop_assert!(step.pending() < step.step());
        }
    }

    #[test]
    fn ticks_per_frame_are_bounded(hz in 1u32..=1000, frames in frame_times()) {
        let mut step = FixedStep::new(TickRate::try_new(hz)?);
        let bound = (MAX_FRAME_TIME + step.step()).as_nanos() / step.step().as_nanos();
        for elapsed in frames {
            prop_assert!(u128::from(step.advance(elapsed).ticks) <= bound);
        }
    }

    #[test]
    fn unclamped_time_is_conserved(
        hz in 1u32..=1000,
        nanos in prop::collection::vec(0u64..250_000_000, 0..200),
    ) {
        let mut step = FixedStep::new(TickRate::try_new(hz)?);
        let mut ticks = 0u32;
        let mut total = Duration::ZERO;
        for n in nanos {
            let elapsed = Duration::from_nanos(n);
            total += elapsed;
            ticks += step.advance(elapsed).ticks;
        }
        prop_assert_eq!(step.step() * ticks + step.pending(), total);
    }
}
