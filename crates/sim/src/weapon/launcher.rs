use std::fmt;

use super::projectile::{Detonation, Fate, Projectile};
use super::tuning::{COOLDOWN_SECONDS, DAMAGE, LIFETIME_SECONDS, MAX_IN_FLIGHT, START_AMMO};
use crate::collision::World;
use crate::movement::Pose;
use crate::tick::TickRate;

/// Why the launcher did not fire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Denied {
    /// No ammo left.
    Empty,
    /// The previous shot is still cooling down.
    Cooling,
    /// The in-flight limit is reached.
    Full,
    /// The aim has no usable direction.
    NoAim,
}

impl fmt::Display for Denied {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "out of ammo",
            Self::Cooling => "still cooling down",
            Self::Full => "too many projectiles in flight",
            Self::NoAim => "aim has no direction",
        })
    }
}

impl std::error::Error for Denied {}

/// A rocket launcher and the rockets it has in the air.
///
/// Ammo only goes down, one per shot. Shots are at least the cooldown
/// apart, and at most `MAX_IN_FLIGHT` rockets exist at once.
#[derive(Debug, Clone)]
pub struct Launcher {
    ammo: u16,
    cooldown_left: u32,
    cooldown_ticks: u32,
    lifetime_ticks: u32,
    dt: f32,
    projectiles: Vec<Projectile>,
    // Reused every tick, so stepping never allocates.
    detonations: Vec<Detonation>,
}

impl Launcher {
    /// A loaded launcher, ready to fire.
    pub fn new(rate: TickRate) -> Self {
        Self {
            ammo: START_AMMO,
            cooldown_left: 0,
            cooldown_ticks: ticks(COOLDOWN_SECONDS, rate),
            lifetime_ticks: ticks(LIFETIME_SECONDS, rate),
            dt: rate.step().as_secs_f32(),
            projectiles: Vec::with_capacity(MAX_IN_FLIGHT),
            detonations: Vec::with_capacity(MAX_IN_FLIGHT),
        }
    }

    /// Flies every projectile one tick and cools the launcher. Call once per
    /// tick, before [`Self::try_fire`].
    pub fn step(&mut self, world: &World) {
        self.cooldown_left = self.cooldown_left.saturating_sub(1);
        self.detonations.clear();
        let (dt, detonations) = (self.dt, &mut self.detonations);
        self.projectiles
            .retain_mut(|projectile| match projectile.advance(dt, world) {
                Fate::Flying => true,
                Fate::Detonated(detonation) => {
                    detonations.push(detonation);
                    false
                }
                Fate::Expired => false,
            });
    }

    /// Releases a rocket from the eye along the view, if the launcher may.
    ///
    /// # Errors
    ///
    /// Returns [`Denied`] with the first reason the shot cannot happen; the
    /// launcher is left unchanged.
    pub fn try_fire(&mut self, aim: &Pose) -> Result<(), Denied> {
        if self.ammo == 0 {
            return Err(Denied::Empty);
        }
        if self.cooldown_left > 0 {
            return Err(Denied::Cooling);
        }
        if self.projectiles.len() >= MAX_IN_FLIGHT {
            return Err(Denied::Full);
        }
        let rocket = Projectile::launch(aim.eye(), aim.forward(), self.lifetime_ticks, DAMAGE)
            .ok_or(Denied::NoAim)?;
        self.projectiles.push(rocket);
        self.ammo -= 1;
        self.cooldown_left = self.cooldown_ticks;
        Ok(())
    }

    /// Rockets left in the launcher.
    pub fn ammo(&self) -> u16 {
        self.ammo
    }

    /// Rockets in flight.
    pub fn projectiles(&self) -> &[Projectile] {
        &self.projectiles
    }

    /// Rockets that hit a solid during the latest [`Self::step`].
    pub fn detonations(&self) -> &[Detonation] {
        &self.detonations
    }
}

/// Whole ticks that cover `seconds`, rounded up so a cooldown is never shorter
/// than asked.
fn ticks(seconds: f32, rate: TickRate) -> u32 {
    (seconds * rate.hz() as f32).ceil() as u32
}
