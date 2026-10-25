use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use glam::{Vec3, Vec3Swizzles};
use sim::collision::{Aabb, World};
use sim::input::{Button, Event, InputState};
use sim::movement::tuning::{
    EYE_HEIGHT, GRAVITY, HALF_WIDTH, JUMP_SPEED, LOOK_SENSITIVITY, MAX_SPEED, PITCH_LIMIT,
};
use sim::movement::{Player, SpawnError};
use sim::tick::TickRate;

const EAST_WALL_X: f32 = 20.0;

fn solid(min: Vec3, max: Vec3) -> Aabb {
    match Aabb::from_corners(min, max) {
        Ok(aabb) => aabb,
        Err(err) => panic!("fixture box is valid: {err}"),
    }
}

/// A floor with its top at y = 0, a wall on the east side and, optionally,
/// a ceiling whose underside is at `ceiling`.
fn arena(ceiling: Option<f32>) -> World {
    let floor = solid(Vec3::new(-30.0, -1.0, -30.0), Vec3::new(30.0, 0.0, 30.0));
    let wall = solid(
        Vec3::new(EAST_WALL_X, -1.0, -30.0),
        Vec3::new(EAST_WALL_X + 1.0, 10.0, 30.0),
    );
    let ceiling =
        ceiling.map(|y| solid(Vec3::new(-30.0, y, -30.0), Vec3::new(30.0, y + 1.0, 30.0)));
    World::new([floor, wall].into_iter().chain(ceiling).collect())
}

fn spawn_at(feet: Vec3, world: &World) -> Player {
    match Player::try_spawn(feet, world, TickRate::DEFAULT) {
        Ok(player) => player,
        Err(err) => panic!("fixture spawn is valid: {err}"),
    }
}

/// A player standing on the arena floor.
fn grounded(world: &World) -> Player {
    let mut player = spawn_at(Vec3::new(0.0, 0.0, 0.0), world);
    let mut input = InputState::new();
    run(&mut player, &mut input, world, 2);
    assert!(player.on_ground(), "fixture player should have landed");
    player
}

fn run(player: &mut Player, input: &mut InputState, world: &World, ticks: u32) {
    for _ in 0..ticks {
        player.step(&input.take_command(), world);
    }
}

fn hold(input: &mut InputState, buttons: &[Button]) {
    for &button in buttons {
        input.apply(Event::Press(button));
    }
}

fn turn_to(input: &mut InputState, yaw: f32) {
    // A positive yaw turns left, so the mouse moves left (negative) for it.
    input.apply(Event::MouseMotion {
        dx: -yaw / LOOK_SENSITIVITY,
        dy: 0.0,
    });
}

fn ground_speed(player: &Player) -> f32 {
    player.velocity().xz().length()
}

#[test]
fn spawn_rejects_a_non_finite_point() {
    let world = arena(None);
    let result = Player::try_spawn(Vec3::new(f32::NAN, 0.0, 0.0), &world, TickRate::DEFAULT);
    assert_eq!(result.err(), Some(SpawnError::NonFinite));
}

#[test]
fn spawn_rejects_a_point_inside_a_solid() {
    let world = arena(None);
    let result = Player::try_spawn(Vec3::new(0.0, -0.5, 0.0), &world, TickRate::DEFAULT);
    assert_eq!(result.err(), Some(SpawnError::Blocked));
}

#[test]
fn a_falling_player_lands_and_stops() {
    let world = arena(None);
    let mut player = spawn_at(Vec3::new(0.0, 3.0, 0.0), &world);
    let mut input = InputState::new();
    run(&mut player, &mut input, &world, 120);
    assert!(player.on_ground());
    assert!(
        player.position().y.abs() < 1.0e-3,
        "y {}",
        player.position().y
    );
    assert_eq!(player.velocity().y, 0.0);
}

#[test]
fn an_idle_player_on_the_ground_stays_put() {
    let world = arena(None);
    let mut player = grounded(&world);
    let before = player.position();
    run(&mut player, &mut InputState::new(), &world, 240);
    assert_eq!(player.position(), before);
}

#[test]
fn forward_reaches_top_speed_and_no_more() {
    let world = arena(None);
    let mut player = grounded(&world);
    let mut input = InputState::new();
    hold(&mut input, &[Button::Forward]);
    run(&mut player, &mut input, &world, 240);
    assert!(ground_speed(&player) > MAX_SPEED * 0.99);
    assert!(ground_speed(&player) <= MAX_SPEED + 1.0e-3);
    assert!(player.velocity().z < 0.0, "forward is -Z at yaw 0");
}

#[test]
fn a_diagonal_is_no_faster_than_a_straight_line() {
    let world = arena(None);
    let mut player = grounded(&world);
    let mut input = InputState::new();
    hold(&mut input, &[Button::Forward, Button::Right]);
    run(&mut player, &mut input, &world, 240);
    assert!(ground_speed(&player) <= MAX_SPEED + 1.0e-3);
}

#[test]
fn letting_go_brings_the_player_to_a_stop() {
    let world = arena(None);
    let mut player = grounded(&world);
    let mut input = InputState::new();
    hold(&mut input, &[Button::Forward]);
    run(&mut player, &mut input, &world, 120);
    input.apply(Event::Release(Button::Forward));
    run(&mut player, &mut input, &world, 120);
    assert_eq!(ground_speed(&player), 0.0);
}

#[test]
fn turning_right_sends_forward_toward_plus_x() {
    let world = arena(None);
    let mut player = grounded(&world);
    let mut input = InputState::new();
    turn_to(&mut input, -FRAC_PI_2);
    hold(&mut input, &[Button::Forward]);
    run(&mut player, &mut input, &world, 60);
    assert!(player.velocity().x > MAX_SPEED * 0.5);
    assert!(player.velocity().z.abs() < 1.0e-3);
}

#[test]
fn a_wall_stops_the_player_short_of_its_face() {
    let world = arena(None);
    let mut player = grounded(&world);
    let mut input = InputState::new();
    turn_to(&mut input, -FRAC_PI_2);
    hold(&mut input, &[Button::Forward]);
    run(&mut player, &mut input, &world, 600);
    let x = player.position().x;
    assert!(x <= EAST_WALL_X - HALF_WIDTH, "x {x}");
    assert!(x > EAST_WALL_X - HALF_WIDTH - 0.01, "x {x}");
    assert_eq!(player.velocity().x, 0.0);
}

#[test]
fn running_into_a_wall_at_an_angle_slides_along_it() {
    let world = arena(None);
    let mut player = grounded(&world);
    let mut input = InputState::new();
    turn_to(&mut input, -FRAC_PI_4);
    hold(&mut input, &[Button::Forward]);
    run(&mut player, &mut input, &world, 600);
    assert!(player.position().x <= EAST_WALL_X - HALF_WIDTH);
    assert!(player.velocity().z < -MAX_SPEED * 0.5);
}

#[test]
fn a_tap_on_jump_shorter_than_a_tick_still_jumps() {
    let world = arena(None);
    let mut player = grounded(&world);
    let mut input = InputState::new();
    input.apply(Event::Press(Button::Jump));
    input.apply(Event::Release(Button::Jump));
    run(&mut player, &mut input, &world, 1);
    assert!(!player.on_ground());
    assert!(player.velocity().y > 0.0);
}

#[test]
fn a_jump_rises_to_the_expected_height_and_lands() {
    let world = arena(None);
    let mut player = grounded(&world);
    let mut input = InputState::new();
    input.apply(Event::Press(Button::Jump));
    input.apply(Event::Release(Button::Jump));

    let mut apex = 0.0f32;
    for _ in 0..240 {
        player.step(&input.take_command(), &world);
        apex = apex.max(player.position().y);
    }
    let expected = JUMP_SPEED * JUMP_SPEED / (2.0 * GRAVITY);
    assert!(
        (apex - expected).abs() < 0.05,
        "apex {apex}, expected {expected}"
    );
    assert!(player.on_ground());
}

#[test]
fn a_low_ceiling_stops_the_head() {
    let ceiling = 2.5;
    let world = arena(Some(ceiling));
    let mut player = grounded(&world);
    let mut input = InputState::new();
    input.apply(Event::Press(Button::Jump));
    input.apply(Event::Release(Button::Jump));

    let mut highest = 0.0f32;
    for _ in 0..240 {
        player.step(&input.take_command(), &world);
        highest = highest.max(player.bounds().max().y);
    }
    assert!(highest <= ceiling, "head reached {highest}");
    assert!(player.on_ground(), "the player should fall back down");
}

#[test]
fn looking_far_down_stops_at_the_pitch_limit() {
    let world = arena(None);
    let mut player = grounded(&world);
    let mut input = InputState::new();
    input.apply(Event::MouseMotion {
        dx: 0.0,
        dy: 100_000.0,
    });
    run(&mut player, &mut input, &world, 1);
    assert_eq!(player.pitch(), -PITCH_LIMIT);
}

#[test]
fn the_eyes_sit_above_the_feet() {
    let world = arena(None);
    let player = grounded(&world);
    let Some(pose) = player.pose() else {
        panic!("a spawned player has a finite pose");
    };
    assert_eq!(pose.eye(), player.position() + Vec3::Y * EYE_HEIGHT);
}

#[test]
fn a_push_adds_to_the_velocity() {
    let world = arena(None);
    let mut player = grounded(&world);
    player.push(Vec3::new(3.0, 0.0, -2.0));
    assert_eq!(player.velocity().x, 3.0);
    assert_eq!(player.velocity().z, -2.0);
}

#[test]
fn an_upward_push_lifts_the_player_off_the_ground() {
    let world = arena(None);
    let mut player = grounded(&world);
    player.push(Vec3::new(0.0, 10.0, 0.0));
    assert!(!player.on_ground());
    run(&mut player, &mut InputState::new(), &world, 30);
    assert!(player.position().y > 1.0, "rose to {}", player.position().y);
}

#[test]
fn a_push_cannot_beat_the_speed_limit() {
    let world = arena(None);
    let mut player = grounded(&world);
    player.push(Vec3::splat(1.0e9));
    assert!(player.velocity().abs().max_element() <= sim::movement::tuning::TERMINAL_SPEED);
}

#[test]
fn a_non_finite_push_is_ignored() {
    let world = arena(None);
    let mut player = grounded(&world);
    let before = player.velocity();
    player.push(Vec3::new(f32::NAN, 1.0, 1.0));
    player.push(Vec3::new(f32::INFINITY, 0.0, 0.0));
    assert_eq!(player.velocity(), before);
    assert!(player.on_ground());
}
