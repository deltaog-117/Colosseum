/// A logical action button, independent of any physical key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    /// Move along the view direction.
    Forward,
    /// Move against the view direction.
    Back,
    /// Strafe left.
    Left,
    /// Strafe right.
    Right,
    /// Jump.
    Jump,
}

impl Button {
    /// Every button, for exhaustive iteration.
    pub const ALL: [Self; 5] = [
        Self::Forward,
        Self::Back,
        Self::Left,
        Self::Right,
        Self::Jump,
    ];

    fn bit(self) -> u8 {
        1 << (self as u8)
    }
}

/// Set of buttons, one bit each.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct ButtonSet(u8);

impl ButtonSet {
    pub(super) fn insert(&mut self, button: Button) {
        self.0 |= button.bit();
    }

    pub(super) fn remove(&mut self, button: Button) {
        self.0 &= !button.bit();
    }

    pub(super) fn contains(self, button: Button) -> bool {
        self.0 & button.bit() != 0
    }

    pub(super) fn clear(&mut self) {
        self.0 = 0;
    }
}
