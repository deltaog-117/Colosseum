use glam::{Vec2, Vec3};

use super::tuning::{REPATH_SECONDS, WAYPOINT_RADIUS};
use crate::nav::{NavGrid, Path};
use crate::tick::TickRate;

/// Walks a bot along a route: plans it, passes each waypoint and plans
/// again now and then.
#[derive(Debug, Clone)]
pub(super) struct Guide {
    path: Option<Path>,
    /// Index into the path of the waypoint being walked to.
    next: usize,
    /// Ticks until the route is planned again.
    wait: u32,
    repath_ticks: u32,
}

impl Guide {
    pub(super) fn new(rate: TickRate) -> Self {
        Self {
            path: None,
            next: 0,
            wait: 0,
            repath_ticks: ((REPATH_SECONDS * rate.hz() as f32).ceil() as u32).max(1),
        }
    }

    /// Where a body at `from` should head to reach `goal`, or `None` if
    /// there is no route or it has already arrived.
    pub(super) fn waypoint(&mut self, from: Vec3, goal: Vec3, nav: &NavGrid) -> Option<Vec3> {
        if self.wait == 0 {
            self.path = nav.path(from, goal);
            self.next = 0;
            self.wait = self.repath_ticks;
        }
        self.wait -= 1;

        let path = self.path.as_ref()?;
        let flat = |point: Vec3| Vec2::new(point.x, point.z);
        while let Some(&point) = path.waypoints().get(self.next) {
            if flat(point).distance(flat(from)) > WAYPOINT_RADIUS {
                return Some(point);
            }
            self.next += 1;
        }
        None
    }
}
