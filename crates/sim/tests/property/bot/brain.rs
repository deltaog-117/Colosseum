use glam::Vec3;
use proptest::prelude::*;
use sim::bot::tuning::{MIN_FIRE_RANGE, TURN_RATE};
use sim::bot::{Brain, Target};
use sim::collision::{Aabb, World};
use sim::input::LOOK_LIMIT;
use sim::movement::Player;
use sim::movement::tuning::LOOK_SENSITIVITY;
use sim::tick::TickRate;

const RATE: TickRate = TickRate::DEFAULT;

fn aabb(min: Vec3, max: Vec3) -> Aabb {
    match Aabb::from_corners(min, max) {
        Ok(aabb) => aabb,
        Err(err) => panic!("fixture box is valid: {err}"),
    }
}

fn floor() -> Aabb {
    aabb(
        Vec3::new(-100.0, -1.0, -100.0),
        Vec3::new(100.0, 0.0, 100.0),
    )
}

fn open() -> World {
    World::new(vec![floor()])
}

/// A long wall across the way to anything north of the origin.
fn walled() -> World {
    World::new(vec![
        floor(),
        aabb(Vec3::new(-100.0, 0.0, -11.0), Vec3::new(100.0, 8.0, -10.0)),
    ])
}

fn target(center: Vec3, velocity: Vec3) -> Target {
    match Target::try_new(center, velocity) {
        Some(target) => target,
        None => panic!("generated target is finite"),
    }
}

fn spawn(world: &World) -> Player {
    match Player::try_spawn(Vec3::ZERO, world, RATE) {
        Ok(player) => player,
        Err(err) => panic!("fixture spawn is free: {err}"),
    }
}

proptest! {
    #[test]
    fn every_command_is_finite_bounded_and_turns_no_faster_than_the_limit(
        seed in any::<u32>(),
        x in -50.0f32..50.0,
        y in 0.0f32..10.0,
        z in -50.0f32..50.0,
        vx in -20.0f32..20.0,
        vz in -20.0f32..20.0,
    ) {
        let world = open();
        let mut player = spawn(&world);
        let mut brain = Brain::new(RATE, seed);
        let mark = target(Vec3::new(x, y, z), Vec3::new(vx, 0.0, vz));
        let limit = TURN_RATE / RATE.hz() as f32;

        for _ in 0..200 {
            let command = brain.decide(&player, Some(&mark), &world);
            let (dx, dy) = (command.look().dx(), command.look().dy());
            prop_assert!(dx.is_finite() && dy.is_finite());
            prop_assert!(dx.abs() <= LOOK_LIMIT && dy.abs() <= LOOK_LIMIT);
            prop_assert!(dx.abs() * LOOK_SENSITIVITY <= limit + 1.0e-4);
            prop_assert!(dy.abs() * LOOK_SENSITIVITY <= limit + 1.0e-4);
            player.step(&command, &world);
        }
    }

    #[test]
    fn a_still_target_in_the_open_is_eventually_fired_at(
        seed in any::<u32>(),
        angle in 0.0f32..std::f32::consts::TAU,
        range in 12.0f32..40.0,
        height in 0.5f32..3.0,
    ) {
        let world = open();
        let mut player = spawn(&world);
        let mut brain = Brain::new(RATE, seed);
        let mark = target(
            Vec3::new(range * angle.sin(), height, range * angle.cos()),
            Vec3::ZERO,
        );

        let mut fired = false;
        for _ in 0..720 {
            let command = brain.decide(&player, Some(&mark), &world);
            fired |= command.fire();
            player.step(&command, &world);
        }
        prop_assert!(fired, "never fired at a target {range} m away");
    }

    #[test]
    fn a_wall_in_the_way_means_no_shots_at_all(
        seed in any::<u32>(),
        x in -8.0f32..8.0,
        range in 15.0f32..40.0,
    ) {
        let world = walled();
        let mut player = spawn(&world);
        let mut brain = Brain::new(RATE, seed);
        let mark = target(Vec3::new(x, 1.6, -range), Vec3::ZERO);

        for _ in 0..600 {
            let command = brain.decide(&player, Some(&mark), &world);
            prop_assert!(!command.fire());
            player.step(&command, &world);
        }
    }

    #[test]
    fn the_bot_never_fires_from_inside_the_blast_distance(
        seed in any::<u32>(),
        x in -3.0f32..3.0,
        z in -3.0f32..3.0,
    ) {
        let world = open();
        let player = spawn(&world);
        let mut brain = Brain::new(RATE, seed);
        let mark = target(Vec3::new(x, 1.6, z), Vec3::ZERO);
        prop_assume!(mark.center().distance(Vec3::new(0.0, 1.6, 0.0)) < MIN_FIRE_RANGE);

        let command = brain.decide(&player, Some(&mark), &world);
        prop_assert!(!command.fire());
    }
}
