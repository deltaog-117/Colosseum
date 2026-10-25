use glam::Vec3;
use sim::collision::{Aabb, AabbError};

fn unit_box(center: Vec3) -> Aabb {
    match Aabb::try_new(center, Vec3::splat(0.5)) {
        Ok(aabb) => aabb,
        Err(err) => panic!("fixture box is valid: {err}"),
    }
}

#[test]
fn rejects_non_finite_components() {
    assert_eq!(
        Aabb::try_new(Vec3::new(f32::NAN, 0.0, 0.0), Vec3::ONE),
        Err(AabbError::NonFinite)
    );
    assert_eq!(
        Aabb::try_new(Vec3::ZERO, Vec3::new(1.0, f32::INFINITY, 1.0)),
        Err(AabbError::NonFinite)
    );
}

#[test]
fn rejects_boxes_without_volume() {
    assert_eq!(
        Aabb::try_new(Vec3::ZERO, Vec3::new(1.0, 0.0, 1.0)),
        Err(AabbError::Degenerate)
    );
    assert_eq!(
        Aabb::from_corners(Vec3::ONE, Vec3::ZERO),
        Err(AabbError::Degenerate)
    );
}

#[test]
fn corners_round_trip() {
    let min = Vec3::new(-2.0, 0.0, 4.0);
    let max = Vec3::new(2.0, 3.0, 6.0);
    let Ok(aabb) = Aabb::from_corners(min, max) else {
        panic!("fixture corners are ordered");
    };
    assert_eq!((aabb.min(), aabb.max()), (min, max));
}

#[test]
fn intersecting_boxes_overlap() {
    assert!(unit_box(Vec3::ZERO).overlaps(&unit_box(Vec3::new(0.9, 0.0, 0.0))));
}

#[test]
fn touching_boxes_do_not_overlap() {
    assert!(!unit_box(Vec3::ZERO).overlaps(&unit_box(Vec3::new(1.0, 0.0, 0.0))));
}

#[test]
fn translated_keeps_the_size() {
    let moved = unit_box(Vec3::ZERO).translated(Vec3::new(3.0, 0.1, -7.0));
    assert_eq!(moved.map(|b| b.half_extents()), Some(Vec3::splat(0.5)));
}

#[test]
fn translated_refuses_a_non_finite_result() {
    assert_eq!(
        unit_box(Vec3::ZERO).translated(Vec3::new(f32::NAN, 0.0, 0.0)),
        None
    );
}
