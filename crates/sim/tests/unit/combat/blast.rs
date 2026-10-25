use glam::Vec3;
use sim::collision::{Aabb, World};
use sim::combat::impact;
use sim::combat::tuning::SELF_DAMAGE_PERCENT;
use sim::movement::Pose;
use sim::tick::TickRate;
use sim::weapon::tuning::{BLAST_RADIUS, DAMAGE, KNOCKBACK_SPEED};
use sim::weapon::{Detonation, Launcher};

const RATE: TickRate = TickRate::DEFAULT;

fn aabb(min: Vec3, max: Vec3) -> Aabb {
    match Aabb::from_corners(min, max) {
        Ok(aabb) => aabb,
        Err(err) => panic!("fixture box is valid: {err}"),
    }
}

/// A wall whose face is at z = -20, seen from the origin looking down -Z.
fn wall() -> Aabb {
    aabb(Vec3::new(-30.0, -1.0, -21.0), Vec3::new(30.0, 10.0, -20.0))
}

/// A standing body, feet on the floor at `feet`.
fn body(feet: Vec3) -> Aabb {
    aabb(
        feet + Vec3::new(-0.4, 0.0, -0.4),
        feet + Vec3::new(0.4, 1.8, 0.4),
    )
}

/// The blast of a rocket fired from the origin at the wall: at (0, 1.6, -20),
/// facing +Z.
fn blast(world: &World, targets: &[Aabb]) -> Detonation {
    let Some(aim) = Pose::try_new(Vec3::new(0.0, 1.6, 0.0), 0.0, 0.0) else {
        panic!("fixture pose is finite");
    };
    let mut launcher = Launcher::new(RATE);
    if let Err(denied) = launcher.try_fire(&aim) {
        panic!("the launcher should fire: {denied}");
    }
    for _ in 0..600 {
        launcher.step_against(world, targets);
        if let Some(detonation) = launcher.detonations().first() {
            return *detonation;
        }
    }
    panic!("the rocket never detonated");
}

fn wall_blast() -> (World, Detonation) {
    let world = World::new(vec![wall()]);
    let detonation = blast(&world, &[]);
    (world, detonation)
}

#[test]
fn a_body_in_reach_takes_damage_that_falls_with_distance() {
    let (world, detonation) = wall_blast();
    // Its nearest face is 1 m from the blast, level with it.
    let Some(hit) = impact(&detonation, &world, &body(Vec3::new(0.0, 0.7, -18.6))) else {
        panic!("a body 1 m from the blast is in reach");
    };
    let expected = (f32::from(DAMAGE) * (1.0 - 1.0 / BLAST_RADIUS)).round() as u16;
    assert_eq!(hit.damage(), expected);

    let Some(farther) = impact(&detonation, &world, &body(Vec3::new(0.0, 0.7, -17.6))) else {
        panic!("a body 2 m from the blast is in reach");
    };
    assert!(farther.damage() < hit.damage());
    assert!(farther.impulse().length() < hit.impulse().length());
}

#[test]
fn a_body_past_the_radius_is_untouched() {
    let (world, detonation) = wall_blast();
    let feet = Vec3::new(0.0, 0.7, -20.0 + BLAST_RADIUS + 1.0);
    assert!(impact(&detonation, &world, &body(feet)).is_none());
}

#[test]
fn knockback_pushes_away_from_the_blast() {
    let (world, detonation) = wall_blast();
    let Some(hit) = impact(&detonation, &world, &body(Vec3::new(0.0, 0.7, -18.6))) else {
        panic!("the body is in reach");
    };
    // Away from the wall is +Z; the body's center is a little above the blast.
    assert!(hit.impulse().z > 0.0);
    assert!(hit.impulse().length() <= KNOCKBACK_SPEED);
}

#[test]
fn a_blast_beneath_the_feet_throws_the_body_upward() {
    // A floor with its top at y = 0; the rocket is aimed straight down.
    let world = World::new(vec![aabb(
        Vec3::new(-30.0, -1.0, -30.0),
        Vec3::new(30.0, 0.0, 30.0),
    )]);
    let Some(aim) = Pose::try_new(Vec3::new(0.0, 1.6, 0.0), 0.0, -1.5) else {
        panic!("fixture pose is finite");
    };
    let mut launcher = Launcher::new(RATE);
    if let Err(denied) = launcher.try_fire(&aim) {
        panic!("the launcher should fire: {denied}");
    }
    for _ in 0..60 {
        launcher.step(&world);
        if !launcher.detonations().is_empty() {
            break;
        }
    }
    let Some(detonation) = launcher.detonations().first() else {
        panic!("the rocket should hit the floor within half a second");
    };
    let Some(hit) = impact(detonation, &world, &body(Vec3::ZERO)) else {
        panic!("a body at the blast is in reach");
    };
    assert!(hit.impulse().y > 0.9 * hit.impulse().length());
}

#[test]
fn a_wall_between_the_blast_and_the_body_blocks_it() {
    let (world, detonation) = wall_blast();
    let blocker = aabb(Vec3::new(-5.0, -1.0, -19.6), Vec3::new(5.0, 5.0, -19.5));
    let walled = World::new(vec![wall(), blocker]);
    let behind = body(Vec3::new(0.0, 0.7, -18.6));
    assert!(impact(&detonation, &world, &behind).is_some());
    assert!(impact(&detonation, &walled, &behind).is_none());
}

#[test]
fn a_direct_hit_deals_full_damage() {
    let world = World::new(vec![wall()]);
    let target = body(Vec3::new(0.0, 0.7, -10.0));
    let detonation = blast(&world, &[target]);
    // The rocket stopped at the target, not the wall.
    assert!(detonation.position().z > -10.0);
    let Some(hit) = impact(&detonation, &world, &target) else {
        panic!("a struck body is in reach");
    };
    assert_eq!(hit.damage(), DAMAGE);
    // Thrown away from the shooter, who stands at +Z.
    assert!(hit.impulse().z < 0.0);
}

#[test]
fn the_shooter_takes_a_share_of_the_damage_and_all_the_knockback() {
    let world = World::new(vec![wall()]);
    let target = body(Vec3::new(0.0, 0.7, -10.0));
    let detonation = blast(&world, &[target]);
    let Some(hit) = impact(&detonation, &world, &target) else {
        panic!("a struck body is in reach");
    };
    let own = hit.to_shooter();
    assert_eq!(own.damage(), DAMAGE * SELF_DAMAGE_PERCENT / 100);
    assert_eq!(own.impulse(), hit.impulse());
}
