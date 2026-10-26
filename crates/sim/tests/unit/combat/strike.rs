use glam::Vec3;
use sim::collision::{Aabb, World};
use sim::combat::tuning::{MAX_HEALTH, SELF_DAMAGE_PERCENT};
use sim::combat::{Health, Outcome, strike};
use sim::movement::{Player, Pose};
use sim::tick::TickRate;
use sim::weapon::{Detonation, Launcher};

const RATE: TickRate = TickRate::DEFAULT;

fn aabb(min: Vec3, max: Vec3) -> Aabb {
    match Aabb::from_corners(min, max) {
        Ok(aabb) => aabb,
        Err(err) => panic!("fixture box is valid: {err}"),
    }
}

/// A wall whose face is at z = -20, seen from the origin looking down -Z.
fn wall_world() -> World {
    World::new(vec![aabb(
        Vec3::new(-30.0, -1.0, -21.0),
        Vec3::new(30.0, 10.0, -20.0),
    )])
}

/// The blast of a rocket fired from the origin at the wall, at (0, 1.6, -20).
fn blast(world: &World) -> Detonation {
    let Some(aim) = Pose::try_new(Vec3::new(0.0, 1.6, 0.0), 0.0, 0.0) else {
        panic!("fixture pose is finite");
    };
    let mut launcher = Launcher::new(RATE);
    if let Err(denied) = launcher.try_fire(&aim) {
        panic!("the launcher should fire: {denied}");
    }
    for _ in 0..600 {
        launcher.step(world);
        if let Some(detonation) = launcher.detonations().first() {
            return *detonation;
        }
    }
    panic!("the rocket never detonated");
}

fn player_at(feet: Vec3, world: &World) -> Player {
    match Player::try_spawn(feet, world, RATE) {
        Ok(player) => player,
        Err(err) => panic!("fixture spawn is free: {err}"),
    }
}

#[test]
fn a_body_in_reach_is_hurt_and_pushed() {
    let world = wall_world();
    let blast = blast(&world);
    let mut player = player_at(Vec3::new(0.0, 0.0, -18.6), &world);
    let mut health = Health::new(RATE);

    let outcome = strike(&blast, &world, &mut player, &mut health, false);

    assert_eq!(outcome, Outcome::Hurt);
    assert!(health.points() < MAX_HEALTH);
    assert!(player.velocity().length() > 0.0);
}

#[test]
fn the_shooter_takes_a_share_of_the_damage() {
    let world = wall_world();
    let blast = blast(&world);
    let feet = Vec3::new(0.0, 0.0, -18.6);

    let (mut other, mut other_health) = (player_at(feet, &world), Health::new(RATE));
    strike(&blast, &world, &mut other, &mut other_health, false);
    let (mut own, mut own_health) = (player_at(feet, &world), Health::new(RATE));
    strike(&blast, &world, &mut own, &mut own_health, true);

    let taken = |health: &Health| MAX_HEALTH - health.points();
    let expected = (u32::from(taken(&other_health)) * u32::from(SELF_DAMAGE_PERCENT) / 100) as u16;
    assert_eq!(taken(&own_health), expected);
    assert_eq!(own.velocity(), other.velocity());
}

#[test]
fn a_body_out_of_reach_is_left_alone() {
    let world = wall_world();
    let blast = blast(&world);
    let mut player = player_at(Vec3::new(0.0, 0.0, 0.0), &world);
    let mut health = Health::new(RATE);

    let outcome = strike(&blast, &world, &mut player, &mut health, false);

    assert_eq!(outcome, Outcome::Ignored);
    assert_eq!(health.points(), MAX_HEALTH);
    assert_eq!(player.velocity(), Vec3::ZERO);
}

#[test]
fn the_dead_are_not_pushed() {
    let world = wall_world();
    let blast = blast(&world);
    let mut player = player_at(Vec3::new(0.0, 0.0, -18.6), &world);
    let mut health = Health::new(RATE);
    health.damage(MAX_HEALTH);

    let outcome = strike(&blast, &world, &mut player, &mut health, false);

    assert_eq!(outcome, Outcome::Ignored);
    assert_eq!(player.velocity(), Vec3::ZERO);
}

#[test]
fn a_blast_that_kills_reports_it() {
    let world = wall_world();
    let blast = blast(&world);
    let mut player = player_at(Vec3::new(0.0, 0.0, -19.4), &world);
    let mut health = Health::new(RATE);
    health.damage(MAX_HEALTH - 1);

    let outcome = strike(&blast, &world, &mut player, &mut health, false);

    assert_eq!(outcome, Outcome::Killed);
    assert!(!health.is_alive());
}
