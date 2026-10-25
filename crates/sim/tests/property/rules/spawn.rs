use glam::Vec3;
use proptest::prelude::*;
use sim::rules::SpawnPoints;

fn coordinate() -> impl Strategy<Value = f32> {
    -100.0f32..100.0
}

fn point() -> impl Strategy<Value = Vec3> {
    (coordinate(), coordinate(), coordinate()).prop_map(|(x, y, z)| Vec3::new(x, y, z))
}

proptest! {
    #[test]
    fn the_chosen_point_is_one_of_the_given_and_no_other_is_farther(
        points in prop::collection::vec(point(), 1..16),
        from in point(),
    ) {
        let spawns = SpawnPoints::try_new(points.clone())
            .map_err(|err| TestCaseError::fail(err.to_string()))?;
        let chosen = spawns.farthest_from(from);
        prop_assert!(points.contains(&chosen));
        for other in &points {
            prop_assert!(other.distance_squared(from) <= chosen.distance_squared(from));
        }
    }
}
