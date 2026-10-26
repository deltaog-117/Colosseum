use sim::input::{Axis, Command, LOOK_LIMIT};

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
    assert!(!idle.fire());
    assert_eq!(idle.look().dx(), 0.0);
    assert_eq!(idle.look().dy(), 0.0);
}

#[test]
fn an_intent_command_carries_what_was_asked() {
    let command = Command::from_intent(Axis::Positive, Axis::Negative, true, true, 12.5, -3.0);
    assert_eq!(command.forward(), Axis::Positive);
    assert_eq!(command.strafe(), Axis::Negative);
    assert!(command.jump());
    assert!(command.fire());
    assert_eq!(command.look().dx(), 12.5);
    assert_eq!(command.look().dy(), -3.0);
}

#[test]
fn an_intent_look_is_cleaned_like_a_device_look() {
    let command = Command::from_intent(
        Axis::Zero,
        Axis::Zero,
        false,
        false,
        f32::NAN,
        f32::INFINITY,
    );
    assert_eq!(command.look().dx(), 0.0);
    assert_eq!(command.look().dy(), 0.0);

    let huge = Command::from_intent(Axis::Zero, Axis::Zero, false, false, 1.0e30, -1.0e30);
    assert_eq!(huge.look().dx(), LOOK_LIMIT);
    assert_eq!(huge.look().dy(), -LOOK_LIMIT);
}
