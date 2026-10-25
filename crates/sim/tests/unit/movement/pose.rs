use std::f32::consts::PI;

use glam::Vec3;
use sim::movement::Pose;

fn pose(eye: Vec3, yaw: f32, pitch: f32) -> Pose {
    match Pose::try_new(eye, yaw, pitch) {
        Some(pose) => pose,
        None => panic!("fixture pose is finite"),
    }
}

#[test]
fn rejects_non_finite_components() {
    assert_eq!(Pose::try_new(Vec3::new(f32::NAN, 0.0, 0.0), 0.0, 0.0), None);
    assert_eq!(Pose::try_new(Vec3::ZERO, f32::INFINITY, 0.0), None);
    assert_eq!(Pose::try_new(Vec3::ZERO, 0.0, f32::NAN), None);
}

#[test]
fn lerp_halfway_is_the_midpoint() {
    let a = pose(Vec3::ZERO, 0.0, -0.2);
    let b = pose(Vec3::new(2.0, 4.0, -6.0), 1.0, 0.2);
    let mid = a.lerp(&b, 0.5);
    assert_eq!(mid.eye(), Vec3::new(1.0, 2.0, -3.0));
    assert!((mid.yaw() - 0.5).abs() < 1.0e-6);
    assert!(mid.pitch().abs() < 1.0e-6);
}

#[test]
fn lerp_takes_the_short_way_around_pi() {
    let a = pose(Vec3::ZERO, PI - 0.1, 0.0);
    let b = pose(Vec3::ZERO, -PI + 0.1, 0.0);
    let mid = a.lerp(&b, 0.5);
    // The short arc passes through ±π; the long one would pass through 0.
    assert!(mid.yaw().abs() > PI - 0.01, "yaw {}", mid.yaw());
}

#[test]
fn lerp_clamps_alpha() {
    let a = pose(Vec3::ZERO, 0.0, 0.0);
    let b = pose(Vec3::ONE, 0.5, 0.1);
    assert_eq!(a.lerp(&b, -3.0), a);
    assert_eq!(a.lerp(&b, 0.0), a);
    assert_eq!(a.lerp(&b, f32::NAN), a);
    assert_eq!(a.lerp(&b, 1.0).eye(), b.eye());
    assert_eq!(a.lerp(&b, 7.0).eye(), b.eye());
}

#[test]
fn forward_follows_yaw_and_pitch() {
    let close = |a: Vec3, b: Vec3| (a - b).length() < 1.0e-6;
    assert!(close(pose(Vec3::ZERO, 0.0, 0.0).forward(), Vec3::NEG_Z));
    // A positive yaw turns left, a positive pitch looks up.
    assert!(close(
        pose(Vec3::ZERO, PI / 2.0, 0.0).forward(),
        Vec3::NEG_X
    ));
    assert!(close(pose(Vec3::ZERO, 0.0, PI / 2.0).forward(), Vec3::Y));
}
