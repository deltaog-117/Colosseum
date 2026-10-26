use glam::Vec3;
use proptest::prelude::*;
use sim::bot::{Bot, Target};
use sim::collision::{Aabb, World};
use sim::combat::tuning::MAX_HEALTH;
use sim::tick::TickRate;
use sim::weapon::tuning::MAX_AMMO;

const RATE: TickRate = TickRate::DEFAULT;

/// A walled room with pillars and a floor.
fn room() -> World {
    let solid = |min: Vec3, max: Vec3| match Aabb::from_corners(min, max) {
        Ok(aabb) => aabb,
        Err(err) => panic!("fixture box is valid: {err}"),
    };
    World::new(vec![
        solid(Vec3::new(-20.0, -1.0, -20.0), Vec3::new(20.0, 0.0, 20.0)),
        solid(Vec3::new(-21.0, 0.0, -20.0), Vec3::new(-20.0, 10.0, 20.0)),
        solid(Vec3::new(20.0, 0.0, -20.0), Vec3::new(21.0, 10.0, 20.0)),
        solid(Vec3::new(-20.0, 0.0, -21.0), Vec3::new(20.0, 10.0, -20.0)),
        solid(Vec3::new(-20.0, 0.0, 20.0), Vec3::new(20.0, 10.0, 21.0)),
        solid(Vec3::new(-6.0, 0.0, -6.0), Vec3::new(-4.0, 3.0, -4.0)),
        solid(Vec3::new(4.0, 0.0, 2.0), Vec3::new(6.0, 3.0, 4.0)),
    ])
}

proptest! {
    #[test]
    fn a_bot_fighting_stays_inside_its_limits(
        seed in any::<u32>(),
        foe_x in -15.0f32..15.0,
        foe_z in -15.0f32..15.0,
        playing in any::<bool>(),
    ) {
        let world = room();
        let Ok(mut bot) = Bot::try_spawn(Vec3::new(0.0, 0.0, 12.0), &world, RATE, seed) else {
            return Ok(());
        };
        let Some(foe) = Target::try_new(Vec3::new(foe_x, 1.6, foe_z), Vec3::ZERO) else {
            return Ok(());
        };

        for _ in 0..900 {
            bot.step(&world, playing, Some(&foe), &[]);
            prop_assert!(bot.launcher().ammo() <= MAX_AMMO);
            prop_assert!(bot.health().points() <= MAX_HEALTH);
            prop_assert!(!world.is_blocked(bot.player().bounds()));
            prop_assert!(bot.player().position().is_finite());
        }
    }
}
