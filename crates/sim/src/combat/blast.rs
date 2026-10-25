use glam::Vec3;

use crate::collision::{Aabb, World};
use crate::weapon::Detonation;
use crate::weapon::tuning::{BLAST_RADIUS, KNOCKBACK_SPEED};

use super::tuning::SELF_DAMAGE_PERCENT;

/// How far in front of the struck face the line to a body starts. Without
/// it the line would begin on the face and slip through a thin wall.
const LIFT: f32 = 0.05;

/// What a blast does to one body.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Impact {
    damage: u16,
    impulse: Vec3,
}

impl Impact {
    /// Hit points to take; at least one.
    pub fn damage(&self) -> u16 {
        self.damage
    }

    /// Velocity to add to the body: finite, and never faster than
    /// `KNOCKBACK_SPEED`.
    pub fn impulse(&self) -> Vec3 {
        self.impulse
    }

    /// The same blast as felt by the shooter, who takes only a share of the
    /// damage but all of the knockback. Never drops below one hit point.
    pub fn to_shooter(self) -> Self {
        Self {
            damage: (u32::from(self.damage) * u32::from(SELF_DAMAGE_PERCENT) / 100).max(1) as u16,
            impulse: self.impulse,
        }
    }
}

/// What `detonation` does to `body`, or `None` if the body is out of reach
/// or walled off from the blast.
///
/// Strength falls off linearly from full at the body's surface to zero at
/// `BLAST_RADIUS`. Knockback pushes from the blast through the body's center.
pub fn impact(detonation: &Detonation, world: &World, body: &Aabb) -> Option<Impact> {
    let blast = detonation.position();
    let reach = (blast - blast.clamp(body.min(), body.max())).length();
    let strength = 1.0 - reach / BLAST_RADIUS;
    // Also rejects a NaN, which compares false.
    if !(strength > 0.0 && strength <= 1.0) {
        return None;
    }
    let damage = (f32::from(detonation.damage()) * strength).round() as u16;
    if damage == 0 {
        return None;
    }

    let start = blast + detonation.normal() * LIFT;
    let to_body = body.center() - start;
    if world.cast(start, to_body).is_some() {
        return None;
    }

    let away = (body.center() - blast).try_normalize().unwrap_or(Vec3::Y);
    Some(Impact {
        damage,
        impulse: away * KNOCKBACK_SPEED * strength,
    })
}
