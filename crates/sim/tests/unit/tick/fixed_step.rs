use std::time::Duration;

use sim::tick::{FixedStep, MAX_FRAME_TIME, TickRate};

#[test]
fn zero_elapsed_runs_no_ticks() {
    let mut step = FixedStep::new(TickRate::DEFAULT);
    let advance = step.advance(Duration::ZERO);
    assert_eq!(advance.ticks, 0);
    assert_eq!(advance.alpha, 0.0);
}

#[test]
fn exactly_one_step_runs_one_tick() {
    let mut step = FixedStep::new(TickRate::DEFAULT);
    let advance = step.advance(step.step());
    assert_eq!(advance.ticks, 1);
    assert_eq!(step.pending(), Duration::ZERO);
}

#[test]
fn partial_step_carries_over() {
    let mut step = FixedStep::new(TickRate::DEFAULT);
    let half = step.step() / 2;
    assert_eq!(step.advance(half).ticks, 0);
    assert_eq!(step.advance(step.step() - half).ticks, 1);
}

#[test]
fn long_stall_is_clamped() {
    let mut step = FixedStep::new(TickRate::DEFAULT);
    let advance = step.advance(Duration::from_secs(60));
    let most = MAX_FRAME_TIME.as_nanos() / step.step().as_nanos();
    assert_eq!(u128::from(advance.ticks), most);
}
