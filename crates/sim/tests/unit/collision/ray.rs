use glam::Vec3;
use sim::collision::{Aabb, World};

fn aabb(min: Vec3, max: Vec3) -> Aabb {
    match Aabb::from_corners(min, max) {
        Ok(aabb) => aabb,
        Err(err) => panic!("fixture box is valid: {err}"),
    }
}

/// A wall slab from x = 5 to x = 6, and a second one behind it, x = 8 to x = 9.
fn walls() -> World {
    World::new(vec![
        aabb(Vec3::new(8.0, -10.0, -10.0), Vec3::new(9.0, 10.0, 10.0)),
        aabb(Vec3::new(5.0, -10.0, -10.0), Vec3::new(6.0, 10.0, 10.0)),
    ])
}

#[test]
fn a_segment_into_a_wall_hits_its_near_face() {
    let hit = walls().cast(Vec3::ZERO, Vec3::new(10.0, 0.0, 0.0));
    let Some(hit) = hit else {
        panic!("the wall is in the way");
    };
    assert_eq!(hit.fraction(), 0.5);
    assert_eq!(hit.point(), Vec3::new(5.0, 0.0, 0.0));
    assert_eq!(hit.normal(), Vec3::NEG_X);
}

#[test]
fn the_nearest_solid_wins_whatever_the_order() {
    let hit = walls().cast(Vec3::new(20.0, 0.0, 0.0), Vec3::new(-20.0, 0.0, 0.0));
    let Some(hit) = hit else {
        panic!("a wall is in the way");
    };
    assert_eq!(hit.point(), Vec3::new(9.0, 0.0, 0.0));
    assert_eq!(hit.normal(), Vec3::X);
}

#[test]
fn a_segment_that_stops_short_hits_nothing() {
    assert_eq!(walls().cast(Vec3::ZERO, Vec3::new(4.9, 0.0, 0.0)), None);
}

#[test]
fn a_segment_that_misses_sideways_hits_nothing() {
    assert_eq!(
        walls().cast(Vec3::new(0.0, 11.0, 0.0), Vec3::new(10.0, 0.0, 0.0)),
        None
    );
}

#[test]
fn a_segment_pointing_away_hits_nothing() {
    assert_eq!(walls().cast(Vec3::ZERO, Vec3::new(-10.0, 0.0, 0.0)), None);
}

#[test]
fn a_very_long_step_cannot_tunnel_a_thin_wall() {
    let thin = World::new(vec![aabb(
        Vec3::new(5.0, -1.0, -1.0),
        Vec3::new(5.001, 1.0, 1.0),
    )]);
    assert!(thin.cast(Vec3::ZERO, Vec3::new(1.0e6, 0.0, 0.0)).is_some());
}

#[test]
fn a_segment_sliding_along_a_face_hits_nothing() {
    let delta = Vec3::new(0.0, 0.0, 4.0);
    assert_eq!(walls().cast(Vec3::new(5.0, 0.0, -2.0), delta), None);
}

#[test]
fn leaving_a_solid_from_its_surface_hits_nothing() {
    assert_eq!(
        walls().cast(Vec3::new(6.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0)),
        None
    );
}

#[test]
fn starting_inside_a_solid_hits_at_once_with_the_normal_pointing_back() {
    let hit = walls().cast(Vec3::new(5.5, 0.0, 0.0), Vec3::new(3.0, 0.0, 0.0));
    let Some(hit) = hit else {
        panic!("the origin is inside the wall");
    };
    assert_eq!(hit.fraction(), 0.0);
    assert_eq!(hit.point(), Vec3::new(5.5, 0.0, 0.0));
    assert_eq!(hit.normal(), Vec3::NEG_X);
}

#[test]
fn a_diagonal_segment_reports_the_face_it_crossed_first() {
    let hit = walls().cast(Vec3::new(0.0, 5.0, 0.0), Vec3::new(10.0, -10.0, 0.0));
    let Some(hit) = hit else {
        panic!("the diagonal crosses the wall");
    };
    assert_eq!(hit.point(), Vec3::new(5.0, 0.0, 0.0));
    assert_eq!(hit.normal(), Vec3::NEG_X);
}

#[test]
fn non_finite_input_hits_nothing() {
    let world = walls();
    assert_eq!(world.cast(Vec3::ZERO, Vec3::new(f32::NAN, 0.0, 0.0)), None);
    assert_eq!(
        world.cast(Vec3::new(f32::INFINITY, 0.0, 0.0), Vec3::X),
        None
    );
}
