use sim::input::{Axis, Button, Command, Event, InputState, LOOK_LIMIT};

fn state_with(events: &[Event]) -> InputState {
    let mut state = InputState::new();
    for &event in events {
        state.apply(event);
    }
    state
}

#[test]
fn nothing_pressed_is_idle() {
    assert_eq!(InputState::new().take_command(), Command::default());
}

#[test]
fn wasd_map_to_axis_signs() {
    let cases = [
        (Button::Forward, Axis::Positive, Axis::Zero),
        (Button::Back, Axis::Negative, Axis::Zero),
        (Button::Left, Axis::Zero, Axis::Negative),
        (Button::Right, Axis::Zero, Axis::Positive),
    ];
    for (button, forward, strafe) in cases {
        let command = state_with(&[Event::Press(button)]).take_command();
        assert_eq!(command.forward(), forward, "{button:?}");
        assert_eq!(command.strafe(), strafe, "{button:?}");
    }
}

#[test]
fn opposing_buttons_cancel() {
    let command = state_with(&[
        Event::Press(Button::Forward),
        Event::Press(Button::Back),
        Event::Press(Button::Left),
        Event::Press(Button::Right),
    ])
    .take_command();
    assert_eq!(command.forward(), Axis::Zero);
    assert_eq!(command.strafe(), Axis::Zero);
}

#[test]
fn held_button_stays_across_commands() {
    let mut state = state_with(&[Event::Press(Button::Forward)]);
    for _ in 0..3 {
        assert_eq!(state.take_command().forward(), Axis::Positive);
    }
}

#[test]
fn released_button_stops_after_the_next_command() {
    let mut state = state_with(&[Event::Press(Button::Forward)]);
    state.take_command();
    state.apply(Event::Release(Button::Forward));
    assert_eq!(state.take_command().forward(), Axis::Zero);
}

#[test]
fn tap_shorter_than_a_tick_is_not_lost() {
    let mut state = state_with(&[Event::Press(Button::Jump), Event::Release(Button::Jump)]);
    assert!(state.take_command().jump());
    assert!(!state.take_command().jump());
}

#[test]
fn look_accumulates_then_resets() {
    let mut state = state_with(&[
        Event::MouseMotion { dx: 3.0, dy: -1.0 },
        Event::MouseMotion { dx: 2.0, dy: -1.5 },
    ]);
    let first = state.take_command().look();
    assert_eq!((first.dx(), first.dy()), (5.0, -2.5));

    let second = state.take_command().look();
    assert_eq!((second.dx(), second.dy()), (0.0, 0.0));
}

#[test]
fn non_finite_motion_is_ignored() {
    let mut state = state_with(&[
        Event::MouseMotion { dx: 1.0, dy: 1.0 },
        Event::MouseMotion {
            dx: f32::NAN,
            dy: 5.0,
        },
        Event::MouseMotion {
            dx: 5.0,
            dy: f32::INFINITY,
        },
    ]);
    let look = state.take_command().look();
    assert_eq!((look.dx(), look.dy()), (1.0, 1.0));
}

#[test]
fn look_is_capped() {
    let mut state = state_with(&[
        Event::MouseMotion {
            dx: f32::MAX,
            dy: f32::MIN,
        },
        Event::MouseMotion {
            dx: f32::MAX,
            dy: f32::MIN,
        },
    ]);
    let look = state.take_command().look();
    assert_eq!((look.dx(), look.dy()), (LOOK_LIMIT, -LOOK_LIMIT));
}

#[test]
fn focus_loss_clears_buttons_taps_and_look() {
    let mut state = state_with(&[
        Event::Press(Button::Forward),
        Event::Press(Button::Jump),
        Event::Release(Button::Jump),
        Event::MouseMotion { dx: 4.0, dy: 4.0 },
        Event::FocusLost,
    ]);
    assert_eq!(state.take_command(), Command::default());
}
