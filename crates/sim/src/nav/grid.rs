use std::fmt;

use glam::Vec3;

use super::path::Path;
use super::search;
use super::tuning::{MAX_CELLS, PROBE_DROP, PROBE_LIFT, SNAP_RINGS};
use crate::collision::{Aabb, Axis3, World};
use crate::movement::tuning::{HALF_WIDTH, HEIGHT};

/// Why a [`NavGrid`] could not be built.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavError {
    /// The world has nothing solid to lay a grid over.
    NoSolids,
    /// The floor height is not finite.
    BadFloor,
    /// The cell size is not finite, not positive, or wider than the player.
    BadCellSize,
    /// The world is so large for the cell size that the grid would not fit.
    TooLarge,
}

impl fmt::Display for NavError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NoSolids => "the world has nothing solid to navigate",
            Self::BadFloor => "floor height is not finite",
            Self::BadCellSize => "cell size must be positive and no wider than the player",
            Self::TooLarge => "the grid would hold too many cells",
        })
    }
}

impl std::error::Error for NavError {}

/// One square of a [`NavGrid`]. Only a grid hands them out, so a cell is
/// always inside the grid it came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    x: u32,
    z: u32,
}

impl Cell {
    /// Column, counted from the grid's low-X edge.
    pub fn x(&self) -> u32 {
        self.x
    }

    /// Row, counted from the grid's low-Z edge.
    pub fn z(&self) -> u32 {
        self.z
    }
}

/// Which spots on one floor a player-sized body can stand on.
///
/// A cell is walkable if the body fits there with a floor underneath. The
/// cell is no wider than the body, so a body that fits at two neighbouring
/// centres fits all the way between them; a path of walkable cells is
/// therefore walkable. It covers one floor height: ramps and upper levels
/// wait for a richer collision model.
#[derive(Debug, Clone, PartialEq)]
pub struct NavGrid {
    min_x: f32,
    min_z: f32,
    cell: f32,
    floor: f32,
    width: u32,
    depth: u32,
    walkable: Vec<bool>,
}

impl NavGrid {
    /// Lays a grid of `cell`-sized squares over everything solid in `world`
    /// and marks where a body standing on `floor` fits.
    ///
    /// # Errors
    ///
    /// Returns [`NavError`] if the world is empty, `floor` or `cell` is
    /// unusable, or the grid would exceed [`MAX_CELLS`].
    pub fn build(world: &World, floor: f32, cell: f32) -> Result<Self, NavError> {
        if !floor.is_finite() {
            return Err(NavError::BadFloor);
        }
        if !(cell.is_finite() && cell > 0.0 && cell <= 2.0 * HALF_WIDTH) {
            return Err(NavError::BadCellSize);
        }
        let (low, high) = world
            .solids()
            .iter()
            .fold(None, |span: Option<(Vec3, Vec3)>, solid| {
                Some(span.map_or((solid.min(), solid.max()), |(lo, hi)| {
                    (lo.min(solid.min()), hi.max(solid.max()))
                }))
            })
            .ok_or(NavError::NoSolids)?;

        // Float-to-int casts saturate, so an absurd span lands in `TooLarge`.
        let count = |extent: f32| ((extent / cell).ceil() as u32).max(1);
        let (width, depth) = (count(high.x - low.x), count(high.z - low.z));
        if u64::from(width) * u64::from(depth) > MAX_CELLS {
            return Err(NavError::TooLarge);
        }

        let mut grid = Self {
            min_x: low.x,
            min_z: low.z,
            cell,
            floor,
            width,
            depth,
            walkable: Vec::new(),
        };
        grid.walkable = (0..depth)
            .flat_map(|z| (0..width).map(move |x| Cell { x, z }))
            .map(|at| grid.fits(world, at))
            .collect();
        Ok(grid)
    }

    /// Side of one cell.
    pub fn cell_size(&self) -> f32 {
        self.cell
    }

    /// Cells along X.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Cells along Z.
    pub fn depth(&self) -> u32 {
        self.depth
    }

    /// True if a body can stand in `cell`.
    pub fn is_walkable(&self, cell: Cell) -> bool {
        self.walkable
            .get(self.index(cell))
            .copied()
            .unwrap_or(false)
    }

    /// The cell holding `point`'s position on the floor, or `None` outside
    /// the grid or for a point that is not finite.
    pub fn cell_at(&self, point: Vec3) -> Option<Cell> {
        let locate = |value: f32, origin: f32, count: u32| {
            let at = ((value - origin) / self.cell).floor();
            (at >= 0.0 && at < count as f32).then_some(at as u32)
        };
        Some(Cell {
            x: locate(point.x, self.min_x, self.width)?,
            z: locate(point.z, self.min_z, self.depth)?,
        })
    }

    /// The middle of `cell`, on the floor.
    pub fn center(&self, cell: Cell) -> Vec3 {
        Vec3::new(
            self.min_x + (cell.x as f32 + 0.5) * self.cell,
            self.floor,
            self.min_z + (cell.z as f32 + 0.5) * self.cell,
        )
    }

    /// The cheapest walk from `from` to `to`, or `None` if either point lies
    /// outside the grid or far from anything walkable, or if walls cut the
    /// two apart. A point inside a wall starts or ends at the nearest
    /// walkable cell instead.
    ///
    /// The path leaves out the cell `from` is in, and ends at `to` itself
    /// when `to` is walkable.
    pub fn path(&self, from: Vec3, to: Vec3) -> Option<Path> {
        let start = self.snap(self.cell_at(from)?)?;
        let goal_cell = self.cell_at(to)?;
        let goal = self.snap(goal_cell)?;
        let cells = search::search(self, start, goal)?;

        let mut waypoints: Vec<Vec3> = search::corners(&cells)
            .into_iter()
            .skip(1)
            .map(|cell| self.center(cell))
            .collect();
        let end = if goal == goal_cell {
            Vec3::new(to.x, self.floor, to.z)
        } else {
            self.center(goal)
        };
        match waypoints.last_mut() {
            Some(last) => *last = end,
            None => waypoints.push(end),
        }
        Path::try_new(waypoints)
    }

    pub(super) fn len(&self) -> usize {
        self.walkable.len()
    }

    pub(super) fn index(&self, cell: Cell) -> usize {
        cell.z as usize * self.width as usize + cell.x as usize
    }

    pub(super) fn cell_of(&self, index: usize) -> Cell {
        let width = self.width as usize;
        Cell {
            x: (index % width) as u32,
            z: (index / width) as u32,
        }
    }

    /// The walkable cell one step from `cell`, if the step is allowed. A
    /// diagonal step needs both cells beside it open, so a path never cuts
    /// a wall's corner.
    pub(super) fn neighbor(&self, cell: Cell, dx: i32, dz: i32) -> Option<Cell> {
        let next = self.offset(cell, dx, dz)?;
        if !self.is_walkable(next) {
            return None;
        }
        let open = |side: Option<Cell>| side.is_some_and(|side| self.is_walkable(side));
        let corner_clear = dx == 0
            || dz == 0
            || (open(self.offset(cell, dx, 0)) && open(self.offset(cell, 0, dz)));
        corner_clear.then_some(next)
    }

    fn offset(&self, cell: Cell, dx: i32, dz: i32) -> Option<Cell> {
        let shift = |at: u32, by: i32, count: u32| {
            let moved = i64::from(at) + i64::from(by);
            (0..i64::from(count))
                .contains(&moved)
                .then_some(moved as u32)
        };
        Some(Cell {
            x: shift(cell.x, dx, self.width)?,
            z: shift(cell.z, dz, self.depth)?,
        })
    }

    fn fits(&self, world: &World, at: Cell) -> bool {
        let half = Vec3::new(HALF_WIDTH, HEIGHT * 0.5, HALF_WIDTH);
        let center = self.center(at) + Vec3::Y * (PROBE_LIFT + half.y);
        let Ok(body) = Aabb::try_new(center, half) else {
            return false;
        };
        !world.is_blocked(&body) && world.sweep(&body, Axis3::Y, -PROBE_DROP).hit()
    }

    /// `cell` itself if walkable, else the walkable cell nearest to it
    /// within [`SNAP_RINGS`] rings; the first found wins a tie.
    fn snap(&self, cell: Cell) -> Option<Cell> {
        if self.is_walkable(cell) {
            return Some(cell);
        }
        let rings = SNAP_RINGS as i32;
        (-rings..=rings)
            .flat_map(|dz| (-rings..=rings).map(move |dx| (dx, dz)))
            .filter_map(|(dx, dz)| Some(((dx, dz), self.offset(cell, dx, dz)?)))
            .filter(|&(_, near)| self.is_walkable(near))
            .min_by_key(|&((dx, dz), _)| dx * dx + dz * dz)
            .map(|(_, near)| near)
    }
}
