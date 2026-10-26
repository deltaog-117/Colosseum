use glam::Vec3;
use sim::bot::tuning::{BACK_OFF_RANGE, PREFERRED_RANGE, TURN_RATE};
use sim::bot::{Brain, Target};
use sim::collision::{Aabb, World};
use sim::input::Axis;
use sim::movement::Player;
use sim::movement::tuning::{EYE_HEIGHT, LOOK_SENSITIVITY};
use sim::tick::TickRate;

const RATE: TickRate = TickRate::DEFAULT;

fn aabb(min: Vec3, max: Vec3) -> Aabb {
    match Aabb::from_corners(min, max) {
        Ok(aabb) => aabb,
        Err(err) => panic!("fixture box is valid: {err}"),
    }
}

/// A big floor and nothing else.
fn open() -> World {
    World::new(vec![aabb(
        Vec3::new(-100.0, -1.0, -100.0),
        Vec3::new(100.0, 0.0, 100.0),
    )])
}

fn bot_at(feet: Vec3, world: &World) -> Player {
    match Player::try_spawn(feet, world, RATE) {
        Ok(player) => player,
        Err(err) => panic!("fixture spawn is free: {err}"),
    }
}

/// A motionless target at eye height, `ahead` meters down -Z from the origin.
fn target_ahead(ahead: f32) -> Target {
    match Target::try_new(Vec3::new(0.0, EYE_HEIGHT, -ahead), Vec3::ZERO) {
        Some(target) => target,
        None => panic!("fixture target is finite"),
    }
}

#[test]
fn without_a_target_the_bot_stands_idle() {
    let world = open();
    let mut brain = Brain::new(RATE, 1);
    let command = brain.decide(&bot_at(Vec3::ZERO, &world), None, &world);
    assert_eq!(command.forward(), Axis::Zero);
    assert_eq!(command.strafe(), Axis::Zero);
    assert!(!command.fire());
    assert!(!command.jump());
}

#[test]
fn a_target_dead_ahead_in_the_open_is_closed_in_on_and_fired_at() {
    let world = open();
    let mut brain = Brain::new(RATE, 1);
    let command = brain.decide(
        &bot_at(Vec3::ZERO, &world),
        Some(&target_ahead(PREFERRED_RANGE + 6.0)),
        &world,
    );
    assert_eq!(command.forward(), Axis::Positive);
    assert!(command.fire());
}

#[test]
fn a_target_inside_back_off_range_is_backed_away_from_and_not_fired_at() {
    let world = open();
    let mut brain = Brain::new(RATE, 1);
    let command = brain.decide(
        &bot_at(Vec3::ZERO, &world),
        Some(&target_ahead(BACK_OFF_RANGE - 2.0)),
        &world,
    );
    assert_eq!(command.forward(), Axis::Negative);
    assert!(!command.fire());
}

#[test]
fn at_a_comfortable_range_the_bot_holds_its_ground_and_strafes() {
    let world = open();
    let mut brain = Brain::new(RATE, 1);
    let command = brain.decide(
        &bot_at(Vec3::ZERO, &world),
        Some(&target_ahead((PREFERRED_RANGE + BACK_OFF_RANGE) / 2.0)),
        &world,
    );
    assert_eq!(command.forward(), Axis::Zero);
    assert_ne!(command.strafe(), Axis::Zero);
}

#[test]
fn a_target_behind_is_turned_toward_at_a_limited_rate_without_firing() {
    let world = open();
    let mut brain = Brain::new(RATE, 1);
    let behind = match Target::try_new(Vec3::new(0.0, EYE_HEIGHT, 15.0), Vec3::ZERO) {
        Some(target) => target,
        None => panic!("fixture target is finite"),
    };
    let command = brain.decide(&bot_at(Vec3::ZERO, &world), Some(&behind), &world);

    assert!(!command.fire());
    let turned = command.look().dx().abs() * LOOK_SENSITIVITY;
    let limit = TURN_RATE / RATE.hz() as f32;
    assert!(turned > 0.0);
    assert!(
        turned <= limit + 1.0e-4,
        "turned {turned} rad, limit {limit}"
    );
}

#[test]
fn a_wall_in_the_line_of_sight_stops_the_shooting() {
    let wall = aabb(Vec3::new(-5.0, 0.0, -11.0), Vec3::new(5.0, 6.0, -10.0));
    let mut solids = open().solids().to_vec();
    solids.push(wall);
    let world = World::new(solids);
    let mut brain = Brain::new(RATE, 1);
    let command = brain.decide(
        &bot_at(Vec3::ZERO, &world),
        Some(&target_ahead(20.0)),
        &world,
    );
    assert!(!command.fire());
}

#[test]
fn a_bot_boxed_in_hops() {
    // A one-body-wide slot with a dead end ahead: every direction the bot
    // can ask for is blocked.
    let mut solids = open().solids().to_vec();
    solids.push(aabb(Vec3::new(0.401, 0.0, -5.0), Vec3::new(2.0, 5.0, 5.0)));
    solids.push(aabb(
        Vec3::new(-2.0, 0.0, -5.0),
        Vec3::new(-0.401, 5.0, 5.0),
    ));
    solids.push(aabb(
        Vec3::new(-2.0, 0.0, -5.0),
        Vec3::new(2.0, 5.0, -0.401),
    ));
    let world = World::new(solids);
    let mut brain = Brain::new(RATE, 1);
    let command = brain.decide(
        &bot_at(Vec3::ZERO, &world),
        Some(&target_ahead(30.0)),
        &world,
    );
    assert!(command.jump());
}

#[test]
fn the_same_seed_replays_the_same_choices() {
    let world = open();
    let body = bot_at(Vec3::ZERO, &world);
    let target = target_ahead(12.0);
    let (mut a, mut b) = (Brain::new(RATE, 7), Brain::new(RATE, 7));
    for _ in 0..500 {
        assert_eq!(
            a.decide(&body, Some(&target), &world),
            b.decide(&body, Some(&target), &world)
        );
    }
}

#[test]
fn a_target_must_be_finite() {
    assert!(Target::try_new(Vec3::new(f32::NAN, 0.0, 0.0), Vec3::ZERO).is_none());
    assert!(Target::try_new(Vec3::ZERO, Vec3::new(0.0, f32::INFINITY, 0.0)).is_none());
}

#[test]
fn the_nearest_target_wins_and_the_earliest_wins_a_tie() {
    let at = |x: f32| match Target::try_new(Vec3::new(x, 0.0, 0.0), Vec3::ZERO) {
        Some(target) => target,
        None => panic!("fixture target is finite"),
    };
    let candidates = [at(9.0), at(-3.0), at(3.0)];
    let nearest = Target::nearest(Vec3::ZERO, &candidates);
    assert_eq!(nearest, Some(&candidates[1]));
    assert_eq!(Target::nearest(Vec3::ZERO, &[]), None);
}
