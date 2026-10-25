use glam::Vec3;
use sim::rules::{SpawnPoints, SpawnsError};

fn points() -> SpawnPoints {
    match SpawnPoints::try_new(vec![
        Vec3::new(0.0, 0.0, 8.0),
        Vec3::new(0.0, 0.0, -8.0),
        Vec3::new(8.0, 0.0, 0.0),
    ]) {
        Ok(points) => points,
        Err(err) => panic!("fixture points are valid: {err}"),
    }
}

#[test]
fn no_points_are_rejected() {
    assert_eq!(SpawnPoints::try_new(vec![]), Err(SpawnsError::Empty));
}

#[test]
fn a_non_finite_point_is_rejected() {
    let result = SpawnPoints::try_new(vec![Vec3::ZERO, Vec3::new(f32::NAN, 0.0, 0.0)]);
    assert_eq!(result, Err(SpawnsError::NonFinite));
}

#[test]
fn the_first_point_is_the_first_given() {
    assert_eq!(points().first(), Vec3::new(0.0, 0.0, 8.0));
}

#[test]
fn the_farthest_point_is_chosen() {
    let far = points().farthest_from(Vec3::new(0.0, 0.0, 8.0));
    assert_eq!(far, Vec3::new(0.0, 0.0, -8.0));
}

#[test]
fn a_tie_goes_to_the_earliest_point() {
    // Equally far from both of the first two points.
    let chosen = points().farthest_from(Vec3::new(0.0, 0.0, 0.0));
    assert_eq!(chosen, Vec3::new(0.0, 0.0, 8.0));
}
