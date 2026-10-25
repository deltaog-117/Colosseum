use std::f32::consts::PI;

use glam::{Vec3, Vec3Swizzles};
use proptest::prelude::*;
use sim::collision::{Aabb, World};
use sim::input::{Button, Command, Event, InputState};
use sim::movement::Player;
use sim::movement::tuning::{MAX_SPEED, PITCH_LIMIT, TERMINAL_SPEED};
use sim::tick::TickRate;

fn solid(min: Vec3, max: Vec3) -> Aabb {
    match Aabb::from_corners(min, max) {
        Ok(aabb) => aabb,
        Err(err) => panic!("fixture box is valid: {err}"),
    }
}

/// A walled floor with a low ceiling and two pillars, so every kind of
/// contact (floor, wall, head, corner) shows up in random play.
fn arena() -> World {
    World::new(vec![
        solid(Vec3::new(-20.0, -1.0, -20.0), Vec3::new(20.0, 0.0, 20.0)),
        solid(Vec3::new(-21.0, -1.0, -20.0), Vec3::new(-20.0, 10.0, 20.0)),
        solid(Vec3::new(20.0, -1.0, -20.0), Vec3::new(21.0, 10.0, 20.0)),
        solid(Vec3::new(-20.0, -1.0, -21.0), Vec3::new(20.0, 10.0, -20.0)),
        solid(Vec3::new(-20.0, -1.0, 20.0), Vec3::new(20.0, 10.0, 21.0)),
        solid(Vec3::new(-20.0, 3.0, -20.0), Vec3::new(20.0, 4.0, 20.0)),
        solid(Vec3::new(-6.0, 0.0, -6.0), Vec3::new(-4.0, 3.0, -4.0)),
        solid(Vec3::new(4.0, 0.0, 2.0), Vec3::new(6.0, 3.0, 4.0)),
    ])
}

/// One tick of play: held buttons as a bit mask, then mouse motion.
type Tick = (u8, f32, f32);

fn plays(max_mask: u8) -> impl Strategy<Value = Vec<Tick>> {
    prop::collection::vec((0..=max_mask, -80.0f32..80.0, -80.0f32..80.0), 1..300)
}

fn command(input: &mut InputState, (mask, dx, dy): Tick) -> Command {
    for (bit, button) in Button::ALL.into_iter().enumerate() {
        let event = if mask & (1 << bit) != 0 {
            Event::Press(button)
        } else {
            Event::Release(button)
        };
        input.apply(event);
    }
    input.apply(Event::MouseMotion { dx, dy });
    input.take_command()
}

fn spawn(x: f32, y: f32, z: f32, world: &World) -> Option<Player> {
    Player::try_spawn(Vec3::new(x, y, z), world, TickRate::DEFAULT).ok()
}

// The jump bit is the highest that moves the player; fire sits above it.
const ALL_BUTTONS: u8 = 0b1_1111;
const WITHOUT_JUMP: u8 = 0b0_1111;

proptest! {
    #[test]
    fn the_player_never_ends_up_inside_a_solid(
        x in -18.0f32..18.0,
        z in -18.0f32..18.0,
        play in plays(ALL_BUTTONS),
    ) {
        let world = arena();
        let Some(mut player) = spawn(x, 0.0, z, &world) else {
            return Err(TestCaseError::reject("spawn overlaps a pillar"));
        };
        let mut input = InputState::new();
        for tick in play {
            player.step(&command(&mut input, tick), &world);
            prop_assert!(!world.is_blocked(player.bounds()), "at {}", player.position());
        }
    }

    #[test]
    fn state_stays_finite_and_within_its_limits(
        x in -18.0f32..18.0,
        y in 0.0f32..2.0,
        z in -18.0f32..18.0,
        play in plays(ALL_BUTTONS),
    ) {
        let world = arena();
        let Some(mut player) = spawn(x, y, z, &world) else {
            return Err(TestCaseError::reject("spawn overlaps a solid"));
        };
        let mut input = InputState::new();
        for tick in play {
            player.step(&command(&mut input, tick), &world);
            prop_assert!(player.position().is_finite());
            prop_assert!(player.velocity().abs().max_element() <= TERMINAL_SPEED);
            prop_assert!((-PI..=PI).contains(&player.yaw()));
            prop_assert!((-PITCH_LIMIT..=PITCH_LIMIT).contains(&player.pitch()));
            prop_assert!(!player.on_ground() || player.velocity().y == 0.0);
            prop_assert!(player.pose().is_some());
        }
    }

    #[test]
    fn walking_never_beats_top_speed(
        x in -18.0f32..18.0,
        z in -18.0f32..18.0,
        play in plays(WITHOUT_JUMP),
    ) {
        let world = arena();
        let Some(mut player) = spawn(x, 0.0, z, &world) else {
            return Err(TestCaseError::reject("spawn overlaps a pillar"));
        };
        let mut input = InputState::new();
        // Settle onto the floor first: the speed bound only holds on the ground.
        for _ in 0..2 {
            player.step(&input.take_command(), &world);
        }
        for tick in play {
            player.step(&command(&mut input, tick), &world);
            prop_assert!(player.on_ground());
            prop_assert!(player.velocity().xz().length() <= MAX_SPEED + 1.0e-3);
        }
    }

    #[test]
    fn the_same_inputs_give_the_same_player(
        x in -18.0f32..18.0,
        z in -18.0f32..18.0,
        play in plays(ALL_BUTTONS),
    ) {
        let world = arena();
        let (Some(mut first), Some(mut second)) = (spawn(x, 0.0, z, &world), spawn(x, 0.0, z, &world)) else {
            return Err(TestCaseError::reject("spawn overlaps a pillar"));
        };
        let (mut input_a, mut input_b) = (InputState::new(), InputState::new());
        for tick in play {
            first.step(&command(&mut input_a, tick), &world);
            second.step(&command(&mut input_b, tick), &world);
        }
        prop_assert_eq!(first.position(), second.position());
        prop_assert_eq!(first.velocity(), second.velocity());
    }
}
