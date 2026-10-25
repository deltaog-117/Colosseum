use std::f32::consts::PI;

use glam::Vec3;
use proptest::prelude::*;
use sim::movement::Pose;

fn poses() -> impl Strategy<Value = Pose> {
    (
        prop::array::uniform3(-100.0f32..100.0),
        -PI..PI,
        -1.5f32..1.5,
    )
        .prop_map(
            |(eye, yaw, pitch)| match Pose::try_new(Vec3::from_array(eye), yaw, pitch) {
                Some(pose) => pose,
                None => panic!("generated pose is finite"),
            },
        )
}

/// Distance between two angles, going the short way around.
fn angle_gap(a: f32, b: f32) -> f32 {
    let gap = (a - b).rem_euclid(2.0 * PI);
    gap.min(2.0 * PI - gap)
}

proptest! {
    #[test]
    fn the_ends_of_a_lerp_are_its_inputs(a in poses(), b in poses()) {
        let start = a.lerp(&b, 0.0);
        prop_assert_eq!(start.eye(), a.eye());
        prop_assert!(angle_gap(start.yaw(), a.yaw()) < 1.0e-4);

        let end = a.lerp(&b, 1.0);
        prop_assert!((end.eye() - b.eye()).abs().max_element() < 1.0e-3);
        prop_assert!(angle_gap(end.yaw(), b.yaw()) < 1.0e-4);
    }

    #[test]
    fn a_lerp_stays_between_its_inputs(a in poses(), b in poses(), alpha in 0.0f32..=1.0) {
        let mid = a.lerp(&b, alpha);
        let (lo, hi) = (a.eye().min(b.eye()), a.eye().max(b.eye()));
        let slack = Vec3::splat(1.0e-3);
        prop_assert!(mid.eye().cmpge(lo - slack).all() && mid.eye().cmple(hi + slack).all());
        prop_assert!(mid.pitch() >= a.pitch().min(b.pitch()) - 1.0e-4);
        prop_assert!(mid.pitch() <= a.pitch().max(b.pitch()) + 1.0e-4);
    }

    #[test]
    fn the_yaw_of_a_lerp_stays_in_range(a in poses(), b in poses(), alpha in any::<f32>()) {
        prop_assert!((-PI..=PI).contains(&a.lerp(&b, alpha).yaw()));
    }

    #[test]
    fn a_lerp_never_turns_further_than_the_short_arc(a in poses(), b in poses(), alpha in 0.0f32..=1.0) {
        let turned = angle_gap(a.lerp(&b, alpha).yaw(), a.yaw());
        prop_assert!(turned <= angle_gap(b.yaw(), a.yaw()) + 1.0e-4);
    }
}

proptest! {
    #[test]
    fn forward_is_a_unit_vector(pose in poses()) {
        prop_assert!((pose.forward().length() - 1.0).abs() < 1.0e-5);
    }
}
