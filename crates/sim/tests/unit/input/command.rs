use sim::input::{Axis, Command};

#[test]
fn axis_values_are_unit_steps() {
    assert_eq!(Axis::Negative.value(), -1.0);
    assert_eq!(Axis::Zero.value(), 0.0);
    assert_eq!(Axis::Positive.value(), 1.0);
}

#[test]
fn default_command_is_idle() {
    let idle = Command::default();
    assert_eq!(idle.forward(), Axis::Zero);
    assert_eq!(idle.strafe(), Axis::Zero);
    assert!(!idle.jump());
    assert_eq!(idle.look().dx(), 0.0);
    assert_eq!(idle.look().dy(), 0.0);
}
