use std::cmp::Reverse;
use std::collections::BinaryHeap;

use super::grid::{Cell, NavGrid};

/// Cost of a step along a row or column, and of a diagonal step (about
/// `10 * sqrt(2)`), in whole numbers so ties break the same way every run.
const STRAIGHT: u32 = 10;
const DIAGONAL: u32 = 14;

/// The eight steps out of a cell, in the order they are tried.
const STEPS: [(i32, i32); 8] = [
    (1, 0),
    (-1, 0),
    (0, 1),
    (0, -1),
    (1, 1),
    (1, -1),
    (-1, 1),
    (-1, -1),
];

/// The cheapest run of cells from `from` to `to`, both included, or `None`
/// if walls cut the two apart.
pub(super) fn search(grid: &NavGrid, from: Cell, to: Cell) -> Option<Vec<Cell>> {
    let goal = grid.index(to);
    let mut cost = vec![u32::MAX; grid.len()];
    let mut parent = vec![usize::MAX; grid.len()];
    let mut open = BinaryHeap::new();

    let start = grid.index(from);
    cost[start] = 0;
    let first = estimate(from, to);
    open.push(Reverse((first, first, start)));

    while let Some(Reverse((total, remaining, index))) = open.pop() {
        if index == goal {
            return Some(unwind(grid, &parent, goal));
        }
        let so_far = total - remaining;
        if so_far > cost[index] {
            // A cheaper way here was found after this entry was queued.
            continue;
        }
        let cell = grid.cell_of(index);
        for (dx, dz) in STEPS {
            let Some(next) = grid.neighbor(cell, dx, dz) else {
                continue;
            };
            let step = if dx != 0 && dz != 0 {
                DIAGONAL
            } else {
                STRAIGHT
            };
            let next_index = grid.index(next);
            let next_cost = so_far + step;
            if next_cost < cost[next_index] {
                cost[next_index] = next_cost;
                parent[next_index] = index;
                let left = estimate(next, to);
                open.push(Reverse((next_cost + left, left, next_index)));
            }
        }
    }
    None
}

/// Drops every cell that continues the direction of the one before it,
/// leaving the first cell and the turns. The ends are kept.
pub(super) fn corners(cells: &[Cell]) -> Vec<Cell> {
    let direction = |a: Cell, b: Cell| {
        (
            i64::from(b.x()) - i64::from(a.x()),
            i64::from(b.z()) - i64::from(a.z()),
        )
    };
    cells
        .iter()
        .enumerate()
        .filter(|&(i, &cell)| {
            i == 0
                || i + 1 == cells.len()
                || direction(cells[i - 1], cell) != direction(cell, cells[i + 1])
        })
        .map(|(_, &cell)| cell)
        .collect()
}

/// Cost of the cheapest walk on an empty grid: never more than the real
/// cost, so the search stays exact.
fn estimate(from: Cell, to: Cell) -> u32 {
    let dx = from.x().abs_diff(to.x());
    let dz = from.z().abs_diff(to.z());
    let (long, short) = (dx.max(dz), dx.min(dz));
    short * DIAGONAL + (long - short) * STRAIGHT
}

fn unwind(grid: &NavGrid, parent: &[usize], goal: usize) -> Vec<Cell> {
    let mut cells = vec![grid.cell_of(goal)];
    let mut at = goal;
    while parent[at] != usize::MAX {
        at = parent[at];
        cells.push(grid.cell_of(at));
    }
    cells.reverse();
    cells
}
