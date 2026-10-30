use glam::Vec3;

/// A route across the grid: the points to walk through, in order, ending at
/// the goal. Never empty and always finite.
#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    waypoints: Vec<Vec3>,
}

impl Path {
    /// `None` if `waypoints` is empty or holds a point that is not finite.
    pub(super) fn try_new(waypoints: Vec<Vec3>) -> Option<Self> {
        (!waypoints.is_empty() && waypoints.iter().all(|point| point.is_finite()))
            .then_some(Self { waypoints })
    }

    /// The points to walk through, nearest first.
    pub fn waypoints(&self) -> &[Vec3] {
        &self.waypoints
    }

    /// Where the path ends.
    pub fn goal(&self) -> Vec3 {
        self.waypoints[self.waypoints.len() - 1]
    }
}
