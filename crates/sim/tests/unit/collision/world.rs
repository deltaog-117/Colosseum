use glam::Vec3;
use sim::collision::{Aabb, Axis3, SKIN, World};

fn aabb(min: Vec3, max: Vec3) -> Aabb {
    match Aabb::from_corners(min, max) {
        Ok(aabb) => aabb,
        Err(err) => panic!("fixture box is valid: {err}"),
    }
}

/// A 1 m cube at the origin and a wall slab from x = 5 to x = 6.
fn fixture() -> (World, Aabb) {
    let wall = aabb(Vec3::new(5.0, -10.0, -10.0), Vec3::new(6.0, 10.0, 10.0));
    let mover = aabb(Vec3::splat(-0.5), Vec3::splat(0.5));
    (World::new(vec![wall]), mover)
}

#[test]
fn free_move_goes_the_whole_way() {
    let (world, mover) = fixture();
    let sweep = world.sweep(&mover, Axis3::X, 2.0);
    assert_eq!((sweep.distance(), sweep.hit()), (2.0, false));
}

#[test]
fn move_into_a_wall_stops_a_skin_short() {
    let (world, mover) = fixture();
    let sweep = world.sweep(&mover, Axis3::X, 10.0);
    // The wall face is 4.5 m from the cube's face.
    assert!(sweep.hit());
    assert!((sweep.distance() - (4.5 - SKIN)).abs() < 1.0e-5);
}

#[test]
fn moving_away_from_a_wall_is_free() {
    let (world, mover) = fixture();
    let sweep = world.sweep(&mover, Axis3::X, -20.0);
    assert_eq!((sweep.distance(), sweep.hit()), (-20.0, false));
}

#[test]
fn a_huge_step_does_not_tunnel() {
    let (world, mover) = fixture();
    let sweep = world.sweep(&mover, Axis3::X, 1.0e6);
    assert!(sweep.hit() && sweep.distance() < 5.0);
}

#[test]
fn a_solid_beside_the_path_does_not_block() {
    let (world, mover) = fixture();
    // The wall is off to the side of a move along Z, so it cannot be hit.
    assert!(!world.sweep(&mover, Axis3::Z, 50.0).hit());
    // Raised past the wall's top, the cube clears it on X as well.
    let Some(raised) = mover.translated(Vec3::new(0.0, 20.0, 0.0)) else {
        panic!("fixture move is finite");
    };
    assert!(!world.sweep(&raised, Axis3::X, 10.0).hit());
}

#[test]
fn a_solid_that_already_overlaps_never_blocks() {
    let (world, _) = fixture();
    let inside = aabb(Vec3::new(5.2, -0.5, -0.5), Vec3::new(5.8, 0.5, 0.5));
    assert!(world.is_blocked(&inside));
    let sweep = world.sweep(&inside, Axis3::X, -1.0);
    assert_eq!((sweep.distance(), sweep.hit()), (-1.0, false));
}

#[test]
fn zero_and_non_finite_moves_go_nowhere() {
    let (world, mover) = fixture();
    for delta in [0.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        let sweep = world.sweep(&mover, Axis3::X, delta);
        assert_eq!(
            (sweep.distance(), sweep.hit()),
            (0.0, false),
            "delta {delta}"
        );
    }
}

#[test]
fn stopping_at_the_skin_again_stays_put() {
    let (world, mover) = fixture();
    let first = world.sweep(&mover, Axis3::X, 10.0);
    let Some(parked) = mover.translated(Vec3::X * first.distance()) else {
        panic!("fixture move is finite");
    };
    let second = world.sweep(&parked, Axis3::X, 1.0);
    // Rounding in `gap - SKIN` leaves a few ulps, nothing visible.
    assert!(second.hit() && second.distance() < 1.0e-6);
    assert!(!world.is_blocked(&parked));
}
