use super::tuning::{MAX_HEALTH, RESPAWN_SECONDS};
use crate::tick::TickRate;

/// What one hit did to a [`Health`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing: the body is already dead, or the damage was zero.
    Ignored,
    /// The body took damage and lives.
    Hurt,
    /// This hit killed the body.
    Killed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// Hit points, always within `1..=MAX_HEALTH`.
    Alive(u16),
    /// Ticks until the body respawns, always positive.
    Dead(u32),
}

/// Hit points, death and the wait to respawn.
///
/// A body is alive with at least one hit point or dead with a countdown
/// running; there is no state in between. Hit points never exceed
/// `MAX_HEALTH`, and the dead take no damage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Health {
    state: State,
    respawn_ticks: u32,
}

impl Health {
    /// A body at full health.
    pub fn new(rate: TickRate) -> Self {
        Self {
            state: State::Alive(MAX_HEALTH),
            // Rounded up, so the wait is never shorter than asked.
            respawn_ticks: ((RESPAWN_SECONDS * rate.hz() as f32).ceil() as u32).max(1),
        }
    }

    /// Takes `amount` hit points, and starts the respawn countdown if that
    /// leaves none.
    pub fn damage(&mut self, amount: u16) -> Outcome {
        let State::Alive(points) = self.state else {
            return Outcome::Ignored;
        };
        if amount == 0 {
            return Outcome::Ignored;
        }
        match points.checked_sub(amount).filter(|left| *left > 0) {
            Some(left) => {
                self.state = State::Alive(left);
                Outcome::Hurt
            }
            None => {
                self.state = State::Dead(self.respawn_ticks);
                Outcome::Killed
            }
        }
    }

    /// Restores up to `amount` hit points, never past `MAX_HEALTH`, and
    /// returns how many came back. The dead cannot be healed: only the
    /// respawn wait brings them back.
    pub fn heal(&mut self, amount: u16) -> u16 {
        let State::Alive(points) = self.state else {
            return 0;
        };
        let healed = points.saturating_add(amount).min(MAX_HEALTH);
        self.state = State::Alive(healed);
        healed - points
    }

    /// Counts the respawn wait down by one tick, and restores full health
    /// when it ends. Returns true on that tick, so the caller can put the
    /// body back in the world. Call once per tick.
    pub fn step(&mut self) -> bool {
        let State::Dead(ticks) = self.state else {
            return false;
        };
        if ticks > 1 {
            self.state = State::Dead(ticks - 1);
            false
        } else {
            self.state = State::Alive(MAX_HEALTH);
            true
        }
    }

    /// Hit points left; zero while dead.
    pub fn points(&self) -> u16 {
        match self.state {
            State::Alive(points) => points,
            State::Dead(_) => 0,
        }
    }

    /// True while the body has hit points.
    pub fn is_alive(&self) -> bool {
        matches!(self.state, State::Alive(_))
    }
}
