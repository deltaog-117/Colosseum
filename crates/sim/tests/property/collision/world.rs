use glam::Vec3;
use proptest::prelude::*;
use sim::collision::{Aabb, Axis3, World};

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

fn axes() -> impl Strategy<Value = Axis3> {
    prop_oneof![Just(Axis3::X), Just(Axis3::Y), Just(Axis3::Z)]
}

proptest! {
    #[test]
    fn a_sweep_never_ends_inside_a_solid(
        solids in solids(),
        center in prop::array::uniform3(-25.0f32..25.0),
        half in prop::array::uniform3(0.1f32..1.0),
        axis in axes(),
        // Wide enough to include steps far longer than any solid is thick.
        delta in -1.0e4f32..1.0e4,
    ) {
        let world = World::new(solids);
        let start = aabb(center, half);
        prop_assume!(!world.is_blocked(&start));

        let sweep = world.sweep(&start, axis, delta);
        let moved = start.translated(axis.unit() * sweep.distance());
        let Some(moved) = moved else {
            return Err(TestCaseError::fail("sweep produced a non-finite box"));
        };
        prop_assert!(!world.is_blocked(&moved));
    }

    #[test]
    fn a_sweep_is_never_longer_than_asked_nor_reversed(
        solids in solids(),
        center in prop::array::uniform3(-25.0f32..25.0),
        half in prop::array::uniform3(0.1f32..1.0),
        axis in axes(),
        delta in -1.0e4f32..1.0e4,
    ) {
        let world = World::new(solids);
        let sweep = world.sweep(&aabb(center, half), axis, delta);
        prop_assert!(sweep.distance().abs() <= delta.abs());
        prop_assert!(sweep.distance() * delta >= 0.0);
        prop_assert_eq!(sweep.hit(), sweep.distance().abs() < delta.abs());
    }
}
