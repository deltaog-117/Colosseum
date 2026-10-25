use glam::Vec3;
use proptest::prelude::*;
use sim::collision::{Aabb, World};
use sim::movement::Pose;
use sim::tick::TickRate;
use sim::weapon::Launcher;
use sim::weapon::tuning::{COOLDOWN_SECONDS, DAMAGE, MAX_IN_FLIGHT, START_AMMO};

/// How far inside a solid a point must be to count as inside, which keeps
/// float rounding on a face from failing a run.
const MARGIN: f32 = 1.0e-3;

fn solid(min: Vec3, max: Vec3) -> Aabb {
    match Aabb::from_corners(min, max) {
        Ok(aabb) => aabb,
        Err(err) => panic!("fixture box is valid: {err}"),
    }
}

/// A walled room with a floor and ceiling, and two pillars in the middle.
fn room() -> World {
    World::new(vec![
        solid(Vec3::new(-20.0, -1.0, -20.0), Vec3::new(20.0, 0.0, 20.0)),
        solid(Vec3::new(-21.0, -1.0, -20.0), Vec3::new(-20.0, 10.0, 20.0)),
        solid(Vec3::new(20.0, -1.0, -20.0), Vec3::new(21.0, 10.0, 20.0)),
        solid(Vec3::new(-20.0, -1.0, -21.0), Vec3::new(20.0, 10.0, -20.0)),
        solid(Vec3::new(-20.0, -1.0, 20.0), Vec3::new(20.0, 10.0, 21.0)),
        solid(Vec3::new(-20.0, 8.0, -20.0), Vec3::new(20.0, 9.0, 20.0)),
        solid(Vec3::new(-6.0, 0.0, -6.0), Vec3::new(-4.0, 3.0, -4.0)),
        solid(Vec3::new(4.0, 0.0, 2.0), Vec3::new(6.0, 3.0, 4.0)),
    ])
}

fn deep_inside(world: &World, point: Vec3) -> bool {
    world.solids().iter().any(|solid| {
        (point - solid.center())
            .abs()
            .cmplt(solid.half_extents() - Vec3::splat(MARGIN))
            .all()
    })
}

/// Whether to pull the trigger, and where the view points, for one tick.
type Tick = (bool, f32, f32);

fn plays() -> impl Strategy<Value = Vec<Tick>> {
    prop::collection::vec((any::<bool>(), -3.1f32..3.1, -1.5f32..1.5), 1..1500)
}

fn cooldown_ticks() -> usize {
    (COOLDOWN_SECONDS * TickRate::DEFAULT.hz() as f32).ceil() as usize
}

proptest! {
    #[test]
    fn ammo_only_falls_by_the_shots_taken_and_the_air_stays_bounded(
        x in -15.0f32..15.0,
        z in -15.0f32..15.0,
        play in plays(),
    ) {
        let world = room();
        let eye = Vec3::new(x, 1.6, z);
        prop_assume!(!deep_inside(&world, eye));
        let mut launcher = Launcher::new(TickRate::DEFAULT);
        let mut shots = 0u16;

        for (trigger, yaw, pitch) in play {
            let Some(aim) = Pose::try_new(eye, yaw, pitch) else {
                return Err(TestCaseError::fail("generated pose is finite"));
            };
            launcher.step(&world);
            if trigger && launcher.try_fire(&aim).is_ok() {
                shots += 1;
            }
            prop_assert_eq!(launcher.ammo() + shots, START_AMMO);
            prop_assert!(launcher.projectiles().len() <= MAX_IN_FLIGHT);
        }
    }

    #[test]
    fn shots_are_never_closer_than_the_cooldown(
        x in -15.0f32..15.0,
        z in -15.0f32..15.0,
        play in plays(),
    ) {
        let world = room();
        let eye = Vec3::new(x, 1.6, z);
        prop_assume!(!deep_inside(&world, eye));
        let mut launcher = Launcher::new(TickRate::DEFAULT);
        let mut last_shot: Option<usize> = None;

        for (tick, (trigger, yaw, pitch)) in play.into_iter().enumerate() {
            let Some(aim) = Pose::try_new(eye, yaw, pitch) else {
                return Err(TestCaseError::fail("generated pose is finite"));
            };
            launcher.step(&world);
            if trigger && launcher.try_fire(&aim).is_ok() {
                if let Some(previous) = last_shot {
                    prop_assert!(tick - previous >= cooldown_ticks());
                }
                last_shot = Some(tick);
            }
        }
    }

    #[test]
    fn rockets_never_end_up_inside_a_solid_and_bangs_carry_full_damage(
        x in -15.0f32..15.0,
        z in -15.0f32..15.0,
        play in plays(),
    ) {
        let world = room();
        let eye = Vec3::new(x, 1.6, z);
        prop_assume!(!deep_inside(&world, eye));
        let mut launcher = Launcher::new(TickRate::DEFAULT);

        for (trigger, yaw, pitch) in play {
            let Some(aim) = Pose::try_new(eye, yaw, pitch) else {
                return Err(TestCaseError::fail("generated pose is finite"));
            };
            launcher.step(&world);
            if trigger {
                let _ = launcher.try_fire(&aim);
            }
            for rocket in launcher.projectiles() {
                prop_assert!(rocket.position().is_finite());
                prop_assert!(!deep_inside(&world, rocket.position()));
            }
            for bang in launcher.detonations() {
                prop_assert_eq!(bang.damage(), DAMAGE);
                prop_assert!(!deep_inside(&world, bang.position()));
                prop_assert!((bang.normal().length() - 1.0).abs() < 1.0e-5);
            }
        }
    }
}
