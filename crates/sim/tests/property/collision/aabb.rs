use glam::Vec3;
use proptest::prelude::*;
use sim::collision::Aabb;

fn boxes() -> impl Strategy<Value = Aabb> {
    (
        (-100.0f32..100.0, -100.0f32..100.0, -100.0f32..100.0),
        (0.01f32..10.0, 0.01f32..10.0, 0.01f32..10.0),
    )
        .prop_map(|((cx, cy, cz), (hx, hy, hz))| {
            match Aabb::try_new(Vec3::new(cx, cy, cz), Vec3::new(hx, hy, hz)) {
                Ok(aabb) => aabb,
                Err(err) => panic!("generated box is valid: {err}"),
            }
        })
}

proptest! {
    #[test]
    fn overlap_is_symmetric(a in boxes(), b in boxes()) {
        prop_assert_eq!(a.overlaps(&b), b.overlaps(&a));
    }

    #[test]
    fn every_box_overlaps_itself(a in boxes()) {
        prop_assert!(a.overlaps(&a));
    }

    #[test]
    fn min_is_below_max_on_every_axis(a in boxes()) {
        prop_assert!(a.min().cmplt(a.max()).all());
    }

    #[test]
    fn translating_keeps_the_half_extents(
        a in boxes(),
        dx in -1.0e3f32..1.0e3,
        dy in -1.0e3f32..1.0e3,
        dz in -1.0e3f32..1.0e3,
    ) {
        let moved = a.translated(Vec3::new(dx, dy, dz));
        prop_assert_eq!(moved.map(|m| m.half_extents()), Some(a.half_extents()));
    }
}
