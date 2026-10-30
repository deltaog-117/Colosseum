use glam::Vec3;
use sim::collision::{Aabb, World};
use sim::nav::tuning::{CELL_SIZE, MAX_CELLS};
use sim::nav::{NavError, NavGrid};

fn aabb(min: Vec3, max: Vec3) -> Aabb {
    match Aabb::from_corners(min, max) {
        Ok(aabb) => aabb,
        Err(err) => panic!("fixture box is valid: {err}"),
    }
}

/// A 20 x 20 m floor and nothing else.
fn floor() -> Vec<Aabb> {
    vec![aabb(
        Vec3::new(-10.0, -1.0, -10.0),
        Vec3::new(10.0, 0.0, 10.0),
    )]
}

fn grid(solids: Vec<Aabb>) -> NavGrid {
    match NavGrid::build(&World::new(solids), 0.0, CELL_SIZE) {
        Ok(grid) => grid,
        Err(err) => panic!("fixture grid builds: {err}"),
    }
}

/// A wall across the floor with a doorway `gap` wide, centred on x = 0.
fn wall_with_gap(gap: f32) -> Vec<Aabb> {
    let mut solids = floor();
    solids.push(aabb(
        Vec3::new(-10.0, 0.0, -0.5),
        Vec3::new(-gap / 2.0, 4.0, 0.5),
    ));
    solids.push(aabb(
        Vec3::new(gap / 2.0, 0.0, -0.5),
        Vec3::new(10.0, 4.0, 0.5),
    ));
    solids
}

fn length(from: Vec3, path: &sim::nav::Path) -> f32 {
    std::iter::once(from)
        .chain(path.waypoints().iter().copied())
        .map_while(Some)
        .collect::<Vec<_>>()
        .windows(2)
        .map(|pair| pair[0].distance(pair[1]))
        .sum()
}

#[test]
fn a_world_with_nothing_solid_has_no_grid() {
    let result = NavGrid::build(&World::new(Vec::new()), 0.0, CELL_SIZE);
    assert_eq!(result, Err(NavError::NoSolids));
}

#[test]
fn unusable_floors_and_cell_sizes_are_refused() {
    let world = World::new(floor());
    assert_eq!(
        NavGrid::build(&world, f32::NAN, CELL_SIZE),
        Err(NavError::BadFloor)
    );
    for cell in [0.0, -0.5, f32::NAN, f32::INFINITY, 0.9] {
        assert_eq!(
            NavGrid::build(&world, 0.0, cell),
            Err(NavError::BadCellSize),
            "cell size {cell}"
        );
    }
}

#[test]
fn a_grid_too_large_for_its_cells_is_refused() {
    let side = (MAX_CELLS as f32).sqrt() * CELL_SIZE * 2.0;
    let world = World::new(vec![aabb(
        Vec3::new(0.0, -1.0, 0.0),
        Vec3::new(side, 0.0, side),
    )]);
    assert_eq!(
        NavGrid::build(&world, 0.0, CELL_SIZE),
        Err(NavError::TooLarge)
    );
}

#[test]
fn the_grid_covers_the_solids() {
    let grid = grid(floor());
    assert_eq!(grid.width(), 40);
    assert_eq!(grid.depth(), 40);
    assert_eq!(grid.cell_size(), CELL_SIZE);
}

#[test]
fn open_floor_is_walkable_and_a_wall_is_not() {
    let grid = grid(wall_with_gap(2.0));
    let walkable = |at: Vec3| grid.cell_at(at).is_some_and(|cell| grid.is_walkable(cell));
    assert!(walkable(Vec3::new(5.0, 0.0, 5.0)));
    assert!(walkable(Vec3::new(0.0, 0.0, 0.0)), "the doorway is open");
    assert!(!walkable(Vec3::new(5.0, 0.0, 0.0)), "inside the wall");
    assert!(
        !walkable(Vec3::new(5.0, 0.0, 0.9)),
        "too close to the wall for a body to fit"
    );
}

#[test]
fn a_point_outside_the_grid_or_not_finite_is_in_no_cell() {
    let grid = grid(floor());
    assert!(grid.cell_at(Vec3::new(50.0, 0.0, 0.0)).is_none());
    assert!(grid.cell_at(Vec3::new(f32::NAN, 0.0, 0.0)).is_none());
}

#[test]
fn a_cells_center_lies_in_that_cell() {
    let grid = grid(floor());
    let Some(cell) = grid.cell_at(Vec3::new(3.3, 0.0, -4.1)) else {
        panic!("the point is on the floor");
    };
    assert_eq!(grid.cell_at(grid.center(cell)), Some(cell));
}

#[test]
fn in_the_open_the_path_is_one_straight_leg() {
    let grid = grid(floor());
    let (from, to) = (Vec3::new(-6.0, 0.0, -6.0), Vec3::new(6.0, 0.0, 6.0));
    let Some(path) = grid.path(from, to) else {
        panic!("open floor is crossable");
    };
    assert_eq!(path.waypoints().len(), 1);
    assert_eq!(path.goal(), to);
}

#[test]
fn a_path_goes_through_the_doorway_and_not_the_wall() {
    let grid = grid(wall_with_gap(2.0));
    let (from, to) = (Vec3::new(-6.0, 0.0, -6.0), Vec3::new(6.0, 0.0, 6.0));
    let Some(path) = grid.path(from, to) else {
        panic!("the doorway lets it through");
    };
    assert!(
        path.waypoints().len() > 1,
        "a straight line would cross the wall"
    );
    let through = path
        .waypoints()
        .iter()
        .any(|point| point.z.abs() < 1.0 && point.x.abs() < 1.0);
    assert!(through, "no waypoint in the doorway: {path:?}");
    assert!(length(from, &path) >= from.distance(to));
}

#[test]
fn a_doorway_narrower_than_a_body_is_not_a_way_through() {
    let grid = grid(wall_with_gap(0.6));
    let path = grid.path(Vec3::new(-6.0, 0.0, -6.0), Vec3::new(6.0, 0.0, 6.0));
    assert!(path.is_none());
}

#[test]
fn a_sealed_room_cannot_be_reached() {
    let mut solids = floor();
    for (min, max) in [
        (Vec3::new(2.0, 0.0, 2.0), Vec3::new(6.0, 4.0, 2.5)),
        (Vec3::new(2.0, 0.0, 6.0), Vec3::new(6.0, 4.0, 6.5)),
        (Vec3::new(2.0, 0.0, 2.0), Vec3::new(2.5, 4.0, 6.5)),
        (Vec3::new(6.0, 0.0, 2.0), Vec3::new(6.5, 4.0, 6.5)),
    ] {
        solids.push(aabb(min, max));
    }
    let grid = grid(solids);
    let path = grid.path(Vec3::new(-6.0, 0.0, -6.0), Vec3::new(4.2, 0.0, 4.2));
    assert!(path.is_none());
}

#[test]
fn a_goal_inside_a_wall_ends_beside_it() {
    let grid = grid(wall_with_gap(2.0));
    let from = Vec3::new(-6.0, 0.0, -6.0);
    let Some(path) = grid.path(from, Vec3::new(5.0, 0.0, 0.0)) else {
        panic!("the wall's face is within reach");
    };
    let goal = path.goal();
    let Some(cell) = grid.cell_at(goal) else {
        panic!("the path ends on the grid");
    };
    assert!(grid.is_walkable(cell));
}

#[test]
fn a_start_in_the_goals_cell_gives_a_single_waypoint() {
    let grid = grid(floor());
    let at = Vec3::new(1.1, 0.0, 1.1);
    let Some(path) = grid.path(at, Vec3::new(1.2, 0.0, 1.2)) else {
        panic!("a step away is reachable");
    };
    assert_eq!(path.waypoints().len(), 1);
}

#[test]
fn a_start_or_goal_off_the_grid_has_no_path() {
    let grid = grid(floor());
    let inside = Vec3::new(0.0, 0.0, 0.0);
    let outside = Vec3::new(80.0, 0.0, 0.0);
    assert!(grid.path(outside, inside).is_none());
    assert!(grid.path(inside, outside).is_none());
}

#[test]
fn cells_over_a_gap_in_the_floor_are_not_walkable() {
    let mut solids = vec![
        aabb(Vec3::new(-10.0, -1.0, -10.0), Vec3::new(-2.0, 0.0, 10.0)),
        aabb(Vec3::new(2.0, -1.0, -10.0), Vec3::new(10.0, 0.0, 10.0)),
    ];
    solids.push(aabb(
        Vec3::new(-10.0, -5.0, -10.0),
        Vec3::new(10.0, -4.0, 10.0),
    ));
    let grid = grid(solids);
    let pit = grid.cell_at(Vec3::new(0.0, 0.0, 0.0));
    assert!(pit.is_some_and(|cell| !grid.is_walkable(cell)));
    assert!(
        grid.path(Vec3::new(-6.0, 0.0, 0.0), Vec3::new(6.0, 0.0, 0.0))
            .is_none()
    );
}
