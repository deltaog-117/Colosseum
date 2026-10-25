use glam::Vec3;
use proptest::prelude::*;
use sim::collision::World;
use sim::movement::Pose;
use sim::tick::TickRate;
use sim::weapon::Launcher;
use sim::weapon::tuning::PROJECTILE_SPEED;

proptest! {
    #[test]
    fn a_rocket_in_open_air_flies_straight_at_a_constant_speed(
        eye in prop::array::uniform3(-100.0f32..100.0),
        yaw in -3.1f32..3.1,
        pitch in -1.5f32..1.5,
        ticks in 1u32..300,
    ) {
        let Some(aim) = Pose::try_new(Vec3::from_array(eye), yaw, pitch) else {
            return Err(TestCaseError::fail("generated pose is finite"));
        };
        let world = World::default();
        let mut launcher = Launcher::new(TickRate::DEFAULT);
        prop_assert!(launcher.try_fire(&aim).is_ok());

        for _ in 0..ticks {
            launcher.step(&world);
        }

        let rocket = launcher.projectiles()[0];
        let flown = rocket.position() - aim.eye();
        let expected = PROJECTILE_SPEED * ticks as f32 / 120.0;
        // Accumulated rounding grows with the number of steps.
        let tolerance = 1.0e-3 + expected * 1.0e-5;
        prop_assert!((flown.length() - expected).abs() < tolerance);
        prop_assert!((flown.normalize() - aim.forward()).length() < 1.0e-3);
    }

    #[test]
    fn the_drawn_position_stays_between_the_last_two_ticks(
        yaw in -3.1f32..3.1,
        pitch in -1.5f32..1.5,
        alpha in -2.0f32..3.0,
    ) {
        let Some(aim) = Pose::try_new(Vec3::ZERO, yaw, pitch) else {
            return Err(TestCaseError::fail("generated pose is finite"));
        };
        let mut launcher = Launcher::new(TickRate::DEFAULT);
        prop_assert!(launcher.try_fire(&aim).is_ok());
        launcher.step(&World::default());

        let rocket = launcher.projectiles()[0];
        let drawn = rocket.position_at(alpha);
        let span = rocket.position() - rocket.previous();
        let along = (drawn - rocket.previous()).dot(span) / span.length_squared();
        prop_assert!((-1.0e-4..=1.0 + 1.0e-4).contains(&along));
    }
}
