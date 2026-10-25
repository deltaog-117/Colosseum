use super::aabb::Aabb;
use super::axis::Axis3;

/// How far a box stops short of a solid. Rounding could otherwise leave it
/// overlapping by a hair, and a box that overlaps a solid is stuck for good.
pub const SKIN: f32 = 1.0e-4;

/// Outcome of moving a box along one axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sweep {
    distance: f32,
    hit: bool,
}

impl Sweep {
    /// Signed distance the box may travel; never longer than requested and
    /// never of the opposite sign.
    pub fn distance(&self) -> f32 {
        self.distance
    }

    /// True if a solid cut the move short.
    pub fn hit(&self) -> bool {
        self.hit
    }
}

/// Everything solid in a level.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct World {
    solids: Vec<Aabb>,
}

impl World {
    /// Solids may touch or overlap each other.
    pub fn new(solids: Vec<Aabb>) -> Self {
        Self { solids }
    }

    /// The solids.
    pub fn solids(&self) -> &[Aabb] {
        &self.solids
    }

    /// True if `bounds` overlaps any solid.
    pub fn is_blocked(&self, bounds: &Aabb) -> bool {
        self.solids.iter().any(|solid| solid.overlaps(bounds))
    }

    /// How far `bounds` can move by `delta` along `axis` before a solid stops
    /// it. A solid that already overlaps `bounds` or lies behind the move
    /// never blocks, so a box can always back out of trouble. A zero or
    /// non-finite `delta` moves nothing.
    pub fn sweep(&self, bounds: &Aabb, axis: Axis3, delta: f32) -> Sweep {
        if delta == 0.0 || !delta.is_finite() {
            return Sweep {
                distance: 0.0,
                hit: false,
            };
        }
        let a = axis.index();
        let [p, q] = axis.others();
        let (lo, hi) = (bounds.min(), bounds.max());

        let free = self
            .solids
            .iter()
            .filter_map(|solid| {
                let (smin, smax) = (solid.min(), solid.max());
                let beside =
                    lo[p] < smax[p] && smin[p] < hi[p] && lo[q] < smax[q] && smin[q] < hi[q];
                if !beside {
                    return None;
                }
                let gap = if delta > 0.0 {
                    smin[a] - hi[a]
                } else {
                    lo[a] - smax[a]
                };
                (gap >= 0.0).then_some((gap - SKIN).max(0.0))
            })
            .fold(f32::INFINITY, f32::min);

        let travel = delta.abs();
        Sweep {
            distance: travel.min(free).copysign(delta),
            hit: free < travel,
        }
    }
}
