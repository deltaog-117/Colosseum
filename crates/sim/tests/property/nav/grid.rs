use glam::Vec3;
use proptest::prelude::*;
use sim::collision::{Aabb, World};
use sim::nav::NavGrid;
use sim::nav::tuning::CELL_SIZE;

/// A 20 x 20 m floor with up to six random boxes standing on it.
fn level(boxes: &[(f32, f32, f32, f32)]) -> World {
    let solid = |min: Vec3, max: Vec3| match Aabb::from_corners(min, max) {
        Ok(aabb) => aabb,
        Err(err) => panic!("fixture box is valid: {err}"),
    };
    let mut solids = vec![solid(
        Vec3::new(-10.0, -1.0, -10.0),
        Vec3::new(10.0, 0.0, 10.0),
    )];
    solids.extend(
        boxes
            .iter()
            .map(|&(x, z, w, d)| solid(Vec3::new(x, 0.0, z), Vec3::new(x + w, 3.0, z + d))),
    );
    World::new(solids)
}

fn boxes() -> impl Strategy<Value = Vec<(f32, f32, f32, f32)>> {
    prop::collection::vec((-9.0f32..7.0, -9.0f32..7.0, 0.3f32..4.0, 0.3f32..4.0), 0..6)
}

fn walkable(grid: &NavGrid, at: Vec3) -> bool {
    grid.cell_at(at).is_some_and(|cell| grid.is_walkable(cell))
}

proptest! {
    #[test]
    fn a_path_only_crosses_walkable_cells(
        boxes in boxes(),
        from in (-9.0f32..9.0, -9.0f32..9.0),
        to in (-9.0f32..9.0, -9.0f32..9.0),
    ) {
        let world = level(&boxes);
        let Ok(grid) = NavGrid::build(&world, 0.0, CELL_SIZE) else {
            return Ok(());
        };
        let from = Vec3::new(from.0, 0.0, from.1);
        let to = Vec3::new(to.0, 0.0, to.1);
        let Some(path) = grid.path(from, to) else {
            return Ok(());
        };

        // Legs between turns run along a row, a column or a diagonal, so
        // sampling one cell apart visits exactly the cells they cross.
        // A start inside a wall is moved to the nearest open cell, so there
        // is no first leg to check from there.
        let mut at = grid
            .cell_at(from)
            .filter(|&cell| grid.is_walkable(cell))
            .map(|cell| grid.center(cell));
        let waypoints = path.waypoints();
        for (i, point) in waypoints.iter().enumerate() {
            prop_assert!(walkable(&grid, *point), "waypoint {i} is not walkable");
            let is_last = i + 1 == waypoints.len();
            if let (Some(start), false) = (at, is_last) {
                let leg = *point - start;
                let steps = (leg.x.abs().max(leg.z.abs()) / CELL_SIZE).round() as u32;
                for step in 0..=steps {
                    let sample = start + leg * (step as f32 / steps.max(1) as f32);
                    prop_assert!(walkable(&grid, sample), "leg {i} crosses a wall at {sample}");
                }
            }
            at = Some(*point);
        }
    }

    #[test]
    fn a_path_is_never_shorter_than_the_straight_line(
        boxes in boxes(),
        from in (-9.0f32..9.0, -9.0f32..9.0),
        to in (-9.0f32..9.0, -9.0f32..9.0),
    ) {
        let world = level(&boxes);
        let Ok(grid) = NavGrid::build(&world, 0.0, CELL_SIZE) else {
            return Ok(());
        };
        let from = Vec3::new(from.0, 0.0, from.1);
        let to = Vec3::new(to.0, 0.0, to.1);
        let Some(path) = grid.path(from, to) else {
            return Ok(());
        };
        let walked: f32 = std::iter::once(from)
            .chain(path.waypoints().iter().copied())
            .collect::<Vec<_>>()
            .windows(2)
            .map(|pair| pair[0].distance(pair[1]))
            .sum();
        // The first leg starts at the start cell's middle, up to one cell off.
        prop_assert!(walked + 2.0 * CELL_SIZE >= from.distance(to));
    }

    #[test]
    fn the_same_question_gets_the_same_path(
        boxes in boxes(),
        from in (-9.0f32..9.0, -9.0f32..9.0),
        to in (-9.0f32..9.0, -9.0f32..9.0),
    ) {
        let world = level(&boxes);
        let Ok(grid) = NavGrid::build(&world, 0.0, CELL_SIZE) else {
            return Ok(());
        };
        let from = Vec3::new(from.0, 0.0, from.1);
        let to = Vec3::new(to.0, 0.0, to.1);
        prop_assert_eq!(grid.path(from, to), grid.path(from, to));
    }

    #[test]
    fn a_path_exists_both_ways_or_neither(
        boxes in boxes(),
        from in (-9.0f32..9.0, -9.0f32..9.0),
        to in (-9.0f32..9.0, -9.0f32..9.0),
    ) {
        let world = level(&boxes);
        let Ok(grid) = NavGrid::build(&world, 0.0, CELL_SIZE) else {
            return Ok(());
        };
        let from = Vec3::new(from.0, 0.0, from.1);
        let to = Vec3::new(to.0, 0.0, to.1);
        prop_assert_eq!(grid.path(from, to).is_some(), grid.path(to, from).is_some());
    }
}
