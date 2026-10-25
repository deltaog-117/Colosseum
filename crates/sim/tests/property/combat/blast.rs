use glam::Vec3;
use proptest::prelude::*;
use sim::collision::{Aabb, World};
use sim::combat::impact;
use sim::movement::Pose;
use sim::tick::TickRate;
use sim::weapon::Launcher;
use sim::weapon::tuning::{DAMAGE, KNOCKBACK_SPEED};

fn solid(min: Vec3, max: Vec3) -> Aabb {
    match Aabb::from_corners(min, max) {
        Ok(aabb) => aabb,
        Err(err) => panic!("fixture box is valid: {err}"),
    }
}

/// A walled room with a floor, a ceiling and a pillar.
fn room() -> World {
    World::new(vec![
        solid(Vec3::new(-20.0, -1.0, -20.0), Vec3::new(20.0, 0.0, 20.0)),
        solid(Vec3::new(-21.0, -1.0, -20.0), Vec3::new(-20.0, 10.0, 20.0)),
        solid(Vec3::new(20.0, -1.0, -20.0), Vec3::new(21.0, 10.0, 20.0)),
        solid(Vec3::new(-20.0, -1.0, -21.0), Vec3::new(20.0, 10.0, -20.0)),
        solid(Vec3::new(-20.0, -1.0, 20.0), Vec3::new(20.0, 10.0, 21.0)),
        solid(Vec3::new(-20.0, 8.0, -20.0), Vec3::new(20.0, 9.0, 20.0)),
        solid(Vec3::new(-2.0, 0.0, -2.0), Vec3::new(2.0, 3.0, 2.0)),
    ])
}

proptest! {
    #[test]
    fn every_impact_is_finite_and_within_the_weapon_limits(
        x in -15.0f32..15.0,
        z in -15.0f32..15.0,
        yaw in -3.1f32..3.1,
        pitch in -1.5f32..1.5,
        bx in -15.0f32..15.0,
        bz in -15.0f32..15.0,
    ) {
        let world = room();
        let eye = Vec3::new(x, 5.0, z);
        let Some(aim) = Pose::try_new(eye, yaw, pitch) else {
            return Err(TestCaseError::fail("generated pose is finite"));
        };
        let mut launcher = Launcher::new(TickRate::DEFAULT);
        prop_assume!(launcher.try_fire(&aim).is_ok());
        let body = solid(Vec3::new(bx - 0.4, 0.0, bz - 0.4), Vec3::new(bx + 0.4, 1.8, bz + 0.4));

        for _ in 0..700 {
            launcher.step(&world);
            for detonation in launcher.detonations() {
                if let Some(hit) = impact(detonation, &world, &body) {
                    prop_assert!(hit.impulse().is_finite());
                    prop_assert!(hit.impulse().length() <= KNOCKBACK_SPEED * 1.001);
                    prop_assert!((1..=DAMAGE).contains(&hit.damage()));
                    let own = hit.to_shooter();
                    prop_assert!(own.damage() >= 1 && own.damage() <= hit.damage());
                    prop_assert_eq!(own.impulse(), hit.impulse());
                }
            }
        }
    }
}
