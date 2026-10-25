use glam::Vec3;

use super::tuning::{
    AMMO_PICKUP_AMOUNT, AMMO_PICKUP_RESPAWN_SECONDS, HEALTH_PICKUP_AMOUNT,
    HEALTH_PICKUP_RESPAWN_SECONDS, PICKUP_HALF_SIZE,
};
use crate::collision::Aabb;
use crate::combat::Health;
use crate::combat::tuning::MAX_HEALTH;
use crate::tick::TickRate;
use crate::weapon::Launcher;
use crate::weapon::tuning::MAX_AMMO;

/// What a pickup gives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickupKind {
    /// Hit points.
    Health,
    /// Rockets.
    Ammo,
}

impl PickupKind {
    /// True if taking this pickup would change anything, so a full body
    /// leaves it for later.
    pub fn is_wanted(self, health: &Health, launcher: &Launcher) -> bool {
        match self {
            Self::Health => health.is_alive() && health.points() < MAX_HEALTH,
            Self::Ammo => launcher.ammo() < MAX_AMMO,
        }
    }

    /// Hands the pickup's contents over. Gives nothing a body cannot hold.
    pub fn give(self, health: &mut Health, launcher: &mut Launcher) {
        match self {
            Self::Health => {
                health.heal(HEALTH_PICKUP_AMOUNT);
            }
            Self::Ammo => {
                launcher.add_ammo(AMMO_PICKUP_AMOUNT);
            }
        }
    }

    fn respawn_seconds(self) -> f32 {
        match self {
            Self::Health => HEALTH_PICKUP_RESPAWN_SECONDS,
            Self::Ammo => AMMO_PICKUP_RESPAWN_SECONDS,
        }
    }
}

/// A health or ammo item standing in the level.
///
/// It is either ready to be taken or counting down to be back; a taken
/// pickup cannot be taken again before the countdown ends.
#[derive(Debug, Clone, PartialEq)]
pub struct Pickup {
    kind: PickupKind,
    bounds: Aabb,
    /// Ticks until it is back; zero while ready.
    cooldown: u32,
    respawn_ticks: u32,
}

impl Pickup {
    /// A ready pickup standing with its base at `feet`, or `None` if `feet`
    /// is not finite.
    pub fn try_new(kind: PickupKind, feet: Vec3, rate: TickRate) -> Option<Self> {
        let half = Vec3::splat(PICKUP_HALF_SIZE);
        let bounds = Aabb::try_new(feet + Vec3::Y * PICKUP_HALF_SIZE, half).ok()?;
        Some(Self {
            kind,
            bounds,
            cooldown: 0,
            // Rounded up, so the wait is never shorter than asked.
            respawn_ticks: ((kind.respawn_seconds() * rate.hz() as f32).ceil() as u32).max(1),
        })
    }

    /// Counts the respawn wait down by one tick, then hands the pickup to
    /// `body` if it is ready, touched and `wants` it. Call once per tick.
    ///
    /// A `None` body (the player is dead, or the round is over) takes
    /// nothing, but the wait still runs.
    pub fn step(
        &mut self,
        body: Option<&Aabb>,
        wants: impl FnOnce(PickupKind) -> bool,
    ) -> Option<PickupKind> {
        self.cooldown = self.cooldown.saturating_sub(1);
        let touched = body.is_some_and(|body| body.overlaps(&self.bounds));
        if self.cooldown == 0 && touched && wants(self.kind) {
            self.cooldown = self.respawn_ticks;
            Some(self.kind)
        } else {
            None
        }
    }

    /// Makes the pickup ready at once, for a new round.
    pub fn reset(&mut self) {
        self.cooldown = 0;
    }

    /// What the pickup gives.
    pub fn kind(&self) -> PickupKind {
        self.kind
    }

    /// True while it can be taken.
    pub fn is_ready(&self) -> bool {
        self.cooldown == 0
    }

    /// The box a body touches to take it.
    pub fn bounds(&self) -> &Aabb {
        &self.bounds
    }
}
