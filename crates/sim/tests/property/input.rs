use proptest::prelude::*;
use sim::input::{Axis, Button, Command, Event, InputState, LOOK_LIMIT};

fn any_button() -> impl Strategy<Value = Button> {
    prop::sample::select(Button::ALL.to_vec())
}

fn any_event() -> impl Strategy<Value = Event> {
    prop_oneof![
        any_button().prop_map(Event::Press),
        any_button().prop_map(Event::Release),
        // Any bit pattern, so NaN, infinities and extremes all occur.
        (any::<f32>(), any::<f32>()).prop_map(|(dx, dy)| Event::MouseMotion { dx, dy }),
        Just(Event::FocusLost),
    ]
}

fn any_events() -> impl Strategy<Value = Vec<Event>> {
    prop::collection::vec(any_event(), 0..100)
}

fn fed(events: &[Event]) -> InputState {
    let mut state = InputState::new();
    for &event in events {
        state.apply(event);
    }
    state
}

/// Whether `command` has `button` active, valid when no opposing button is.
fn is_active(command: &Command, button: Button) -> bool {
    match button {
        Button::Forward => command.forward() == Axis::Positive,
        Button::Back => command.forward() == Axis::Negative,
        Button::Left => command.strafe() == Axis::Negative,
        Button::Right => command.strafe() == Axis::Positive,
        Button::Jump => command.jump(),
        Button::Fire => command.fire(),
    }
}

proptest! {
    #[test]
    fn look_is_finite_and_bounded(events in any_events()) {
        let look = fed(&events).take_command().look();
        prop_assert!(look.dx().is_finite() && look.dy().is_finite());
        prop_assert!(look.dx().abs() <= LOOK_LIMIT && look.dy().abs() <= LOOK_LIMIT);
    }

    #[test]
    fn look_is_consumed_exactly_once(events in any_events()) {
        let mut state = fed(&events);
        state.take_command();
        let look = state.take_command().look();
        prop_assert_eq!((look.dx(), look.dy()), (0.0, 0.0));
    }

    #[test]
    fn a_tap_is_seen_by_exactly_one_command(button in any_button()) {
        let mut state = fed(&[Event::Press(button), Event::Release(button)]);
        prop_assert!(is_active(&state.take_command(), button));
        prop_assert_eq!(state.take_command(), Command::default());
    }

    #[test]
    fn pressing_one_button_activates_only_it(button in any_button()) {
        let command = fed(&[Event::Press(button)]).take_command();
        for other in Button::ALL {
            prop_assert_eq!(is_active(&command, other), other == button);
        }
    }

    #[test]
    fn held_button_survives_any_number_of_commands(button in any_button(), takes in 1usize..20) {
        let mut state = fed(&[Event::Press(button)]);
        for _ in 0..takes {
            prop_assert!(is_active(&state.take_command(), button));
        }
    }

    #[test]
    fn focus_loss_always_returns_to_idle(events in any_events()) {
        let mut state = fed(&events);
        state.apply(Event::FocusLost);
        prop_assert_eq!(state.take_command(), Command::default());
    }
}
