use glam::Vec3;
use proptest::prelude::*;
use sim::collision::{Aabb, World};
use sim::combat::tuning::MAX_HEALTH;
use sim::combat::{Health, strike};
use sim::movement::{Player, Pose};
use sim::tick::TickRate;
use sim::weapon::tuning::KNOCKBACK_SPEED;
use sim::weapon::{Detonation, Launcher};

const RATE: TickRate = TickRate::DEFAULT;

fn wall_world() -> World {
    match Aabb::from_corners(Vec3::new(-30.0, -1.0, -21.0), Vec3::new(30.0, 10.0, -20.0)) {
        Ok(wall) => World::new(vec![wall]),
        Err(err) => panic!("fixture box is valid: {err}"),
    }
}

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

proptest! {
    #[test]
    fn a_strike_never_heals_and_never_over_pushes(
        x in -6.0f32..6.0,
        z in -19.5f32..-10.0,
        start in 1u16..=MAX_HEALTH,
        is_shooter in any::<bool>(),
    ) {
        let world = wall_world();
        let blast = blast(&world);
        let Ok(mut player) = Player::try_spawn(Vec3::new(x, 0.0, z), &world, RATE) else {
            return Ok(());
        };
        let mut health = Health::new(RATE);
        health.damage(MAX_HEALTH - start);
        let before = health.points();

        strike(&blast, &world, &mut player, &mut health, is_shooter);

        prop_assert!(health.points() <= before);
        prop_assert!(player.velocity().length() <= KNOCKBACK_SPEED + 1.0e-3);
    }
}
