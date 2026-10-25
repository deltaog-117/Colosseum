use std::fmt;

use glam::Vec3;

/// Why an [`Aabb`] was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AabbError {
    /// A component is NaN or infinite.
    NonFinite,
    /// The box has no volume: a half extent is not positive.
    Degenerate,
}

impl fmt::Display for AabbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NonFinite => "a box component is not finite",
            Self::Degenerate => "a box half extent is not positive",
        })
    }
}

impl std::error::Error for AabbError {}

/// An axis-aligned box with volume.
///
/// Stored as a center and half extents, so moving it never changes its size
/// by rounding. Always finite, with every half extent positive.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    center: Vec3,
    half: Vec3,
}

impl Aabb {
    /// Checks every invariant of [`Aabb`].
    pub fn try_new(center: Vec3, half_extents: Vec3) -> Result<Self, AabbError> {
        if !(center.is_finite() && half_extents.is_finite()) {
            return Err(AabbError::NonFinite);
        }
        if !half_extents.cmpgt(Vec3::ZERO).all() {
            return Err(AabbError::Degenerate);
        }
        Ok(Self {
            center,
            half: half_extents,
        })
    }

    /// The box spanning `min` to `max`; `min` must be below `max` on every axis.
    pub fn from_corners(min: Vec3, max: Vec3) -> Result<Self, AabbError> {
        // Overflow or NaN in the halving surfaces as `NonFinite` from `try_new`.
        Self::try_new((min + max) * 0.5, (max - min) * 0.5)
    }

    /// Middle of the box.
    pub fn center(&self) -> Vec3 {
        self.center
    }

    /// Distance from the center to each face.
    pub fn half_extents(&self) -> Vec3 {
        self.half
    }

    /// Corner with the smallest coordinates.
    pub fn min(&self) -> Vec3 {
        self.center - self.half
    }

    /// Corner with the largest coordinates.
    pub fn max(&self) -> Vec3 {
        self.center + self.half
    }

    /// True if the interiors intersect; boxes that only touch do not overlap.
    pub fn overlaps(&self, other: &Self) -> bool {
        (self.center - other.center)
            .abs()
            .cmplt(self.half + other.half)
            .all()
    }

    /// The same box moved by `delta`, or `None` if the result is not finite.
    pub fn translated(&self, delta: Vec3) -> Option<Self> {
        let center = self.center + delta;
        center.is_finite().then_some(Self {
            center,
            half: self.half,
        })
    }
}
