use std::fmt;

use glam::Vec3;

/// Why a set of spawn points was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnsError {
    /// There are no points to choose from.
    Empty,
    /// A coordinate is NaN or infinite.
    NonFinite,
}

impl fmt::Display for SpawnsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "there is no spawn point",
            Self::NonFinite => "a spawn point is not finite",
        })
    }
}

impl std::error::Error for SpawnsError {}

/// Where bodies may start, as feet positions.
///
/// Never empty and always finite, so choosing a point cannot fail.
#[derive(Debug, Clone, PartialEq)]
pub struct SpawnPoints(Vec<Vec3>);

impl SpawnPoints {
    /// Checks every invariant of [`SpawnPoints`].
    ///
    /// # Errors
    ///
    /// Returns [`SpawnsError`] if `points` is empty or holds a non-finite one.
    pub fn try_new(points: Vec<Vec3>) -> Result<Self, SpawnsError> {
        if points.is_empty() {
            return Err(SpawnsError::Empty);
        }
        if !points.iter().all(|point| point.is_finite()) {
            return Err(SpawnsError::NonFinite);
        }
        Ok(Self(points))
    }

    /// Every point, in the order given.
    pub fn points(&self) -> &[Vec3] {
        &self.0
    }

    /// The point the first round starts from.
    pub fn first(&self) -> Vec3 {
        self.0[0]
    }

    /// The point farthest from `from`, so a respawn is not where the body
    /// just fell. The earliest point wins a tie.
    pub fn farthest_from(&self, from: Vec3) -> Vec3 {
        self.0.iter().copied().fold(self.first(), |best, point| {
            if point.distance_squared(from) > best.distance_squared(from) {
                point
            } else {
                best
            }
        })
    }
}
