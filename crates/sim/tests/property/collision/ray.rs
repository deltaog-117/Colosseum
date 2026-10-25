use glam::Vec3;
use proptest::prelude::*;
use sim::collision::{Aabb, World};

/// How far inside a solid a point must be to count as inside, which keeps
/// float rounding on a face from failing a run.
const MARGIN: f32 = 1.0e-3;

fn aabb(center: [f32; 3], half: [f32; 3]) -> Aabb {
    match Aabb::try_new(Vec3::from_array(center), Vec3::from_array(half)) {
        Ok(aabb) => aabb,
        Err(err) => panic!("generated box is valid: {err}"),
    }
}

fn solids() -> impl Strategy<Value = Vec<Aabb>> {
    prop::collection::vec(
        (
            prop::array::uniform3(-20.0f32..20.0),
            prop::array::uniform3(0.1f32..6.0),
        )
            .prop_map(|(center, half)| aabb(center, half)),
        0..8,
    )
}

fn deep_inside(solid: &Aabb, point: Vec3) -> bool {
    (point - solid.center())
        .abs()
        .cmplt(solid.half_extents() - Vec3::splat(MARGIN))
        .all()
}

proptest! {
    #[test]
    fn a_hit_lies_on_the_segment_and_not_deep_inside_any_solid(
        solids in solids(),
        origin in prop::array::uniform3(-25.0f32..25.0),
        delta in prop::array::uniform3(-1.0e3f32..1.0e3),
    ) {
        let (origin, delta) = (Vec3::from_array(origin), Vec3::from_array(delta));
        let world = World::new(solids);
        prop_assume!(!world.solids().iter().any(|solid| deep_inside(solid, origin)));

        if let Some(hit) = world.cast(origin, delta) {
            prop_assert!((0.0..=1.0).contains(&hit.fraction()));
            prop_assert!(hit.point().is_finite());
            prop_assert!((hit.normal().length() - 1.0).abs() < 1.0e-5);
            prop_assert!(!world.solids().iter().any(|solid| deep_inside(solid, hit.point())));
        }
    }

    #[test]
    fn a_miss_means_the_segment_never_enters_a_solid(
        solids in solids(),
        origin in prop::array::uniform3(-25.0f32..25.0),
        delta in prop::array::uniform3(-60.0f32..60.0),
    ) {
        let (origin, delta) = (Vec3::from_array(origin), Vec3::from_array(delta));
        let world = World::new(solids);
        prop_assume!(world.cast(origin, delta).is_none());

        for step in 0..=200 {
            let point = origin + delta * (step as f32 / 200.0);
            prop_assert!(!world.solids().iter().any(|solid| deep_inside(solid, point)));
        }
    }

    #[test]
    fn a_hit_cannot_be_beaten_by_a_shorter_segment(
        solids in solids(),
        origin in prop::array::uniform3(-25.0f32..25.0),
        delta in prop::array::uniform3(-60.0f32..60.0),
    ) {
        let (origin, delta) = (Vec3::from_array(origin), Vec3::from_array(delta));
        let world = World::new(solids);
        let Some(hit) = world.cast(origin, delta) else {
            return Ok(());
        };
        // Cutting the segment just short of the hit must find nothing, unless it
        // started inside a solid and hit at once.
        prop_assume!(hit.fraction() > 1.0e-3);
        prop_assert!(world.cast(origin, delta * (hit.fraction() * 0.999)).is_none());
    }
}
