use super::LOOK_LIMIT;

/// One movement axis, resolved to a direction.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Axis {
    /// Backward or left.
    Negative,
    /// No movement, or opposing buttons cancelling out.
    #[default]
    Zero,
    /// Forward or right.
    Positive,
}

impl Axis {
    /// Opposing buttons cancel: both held, or neither, resolve to `Zero`.
    pub(super) fn from_pair(negative: bool, positive: bool) -> Self {
        match (negative, positive) {
            (true, false) => Self::Negative,
            (false, true) => Self::Positive,
            _ => Self::Zero,
        }
    }

    /// `-1.0`, `0.0` or `1.0`.
    pub fn value(self) -> f32 {
        match self {
            Self::Negative => -1.0,
            Self::Zero => 0.0,
            Self::Positive => 1.0,
        }
    }
}

/// Raw mouse movement since the previous command, in device counts.
/// Sensitivity is applied later, by the code that turns the view.
///
/// Always finite and within `-LOOK_LIMIT..=LOOK_LIMIT` (see [`super::LOOK_LIMIT`]).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Look {
    dx: f32,
    dy: f32,
}

impl Look {
    pub(super) fn new(dx: f32, dy: f32) -> Self {
        Self { dx, dy }
    }

    /// Horizontal movement; positive is to the right.
    pub fn dx(self) -> f32 {
        self.dx
    }

    /// Vertical movement; positive is downward, as devices report it.
    pub fn dy(self) -> f32 {
        self.dy
    }
}

/// Everything the player asked for during one tick.
///
/// Built only by [`super::InputState::take_command`]; `Command::default()`
/// is the idle command.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Command {
    forward: Axis,
    strafe: Axis,
    jump: bool,
    fire: bool,
    look: Look,
}

impl Command {
    pub(super) fn new(forward: Axis, strafe: Axis, jump: bool, fire: bool, look: Look) -> Self {
        Self {
            forward,
            strafe,
            jump,
            fire,
            look,
        }
    }

    /// A command made by code instead of a device, such as a bot's. `look_x`
    /// and `look_y` are in device counts, like [`Look`]; a non-finite one
    /// counts as zero and the rest is clamped to `±LOOK_LIMIT`, so the
    /// invariants of [`Look`] hold for every caller.
    pub fn from_intent(
        forward: Axis,
        strafe: Axis,
        jump: bool,
        fire: bool,
        look_x: f32,
        look_y: f32,
    ) -> Self {
        let clean = |counts: f32| {
            if counts.is_finite() {
                counts.clamp(-LOOK_LIMIT, LOOK_LIMIT)
            } else {
                0.0
            }
        };
        Self::new(
            forward,
            strafe,
            jump,
            fire,
            Look::new(clean(look_x), clean(look_y)),
        )
    }

    /// Movement along the view direction; positive is forward.
    pub fn forward(&self) -> Axis {
        self.forward
    }

    /// Sideways movement; positive is right.
    pub fn strafe(&self) -> Axis {
        self.strafe
    }

    /// Whether jump was held, or pressed since the previous command.
    pub fn jump(&self) -> bool {
        self.jump
    }

    /// Whether fire was held, or pressed since the previous command.
    pub fn fire(&self) -> bool {
        self.fire
    }

    /// Mouse movement since the previous command.
    pub fn look(&self) -> Look {
        self.look
    }
}
