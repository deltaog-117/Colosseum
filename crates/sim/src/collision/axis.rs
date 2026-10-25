use glam::Vec3;

/// One of the three world axes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis3 {
    /// Right.
    X,
    /// Up.
    Y,
    /// Backward (the view looks down -Z).
    Z,
}

impl Axis3 {
    /// Position of this axis in a [`Vec3`].
    pub fn index(self) -> usize {
        self as usize
    }

    /// Unit vector along this axis.
    pub fn unit(self) -> Vec3 {
        Vec3::AXES[self.index()]
    }

    /// Indices of the two other axes.
    pub(super) fn others(self) -> [usize; 2] {
        match self {
            Self::X => [1, 2],
            Self::Y => [0, 2],
            Self::Z => [0, 1],
        }
    }
}
