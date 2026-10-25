use std::time::Duration;

use sim::tick::TickRate;

#[test]
fn default_is_120_hz() {
    assert_eq!(TickRate::default().hz(), 120);
}

#[test]
fn rejects_zero_and_above_range() {
    assert!(TickRate::try_new(0).is_err());
    assert!(TickRate::try_new(1001).is_err());
}

#[test]
fn accepts_range_edges() {
    assert!(TickRate::try_new(1).is_ok());
    assert!(TickRate::try_new(1000).is_ok());
}

#[test]
fn step_of_120_hz_is_rounded_down_nanos() {
    assert_eq!(TickRate::DEFAULT.step(), Duration::from_nanos(8_333_333));
}
