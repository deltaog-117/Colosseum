use glam::Vec3;

use super::tuning::PROJECTILE_SPEED;
use crate::collision::World;

/// A projectile that struck a solid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Detonation {
    position: Vec3,
    normal: Vec3,
    damage: u16,
}

impl Detonation {
    /// Where the projectile met the solid.
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Unit normal of the face it hit.
    pub fn normal(&self) -> Vec3 {
        self.normal
    }

    /// Damage the projectile carried.
    pub fn damage(&self) -> u16 {
        self.damage
    }
}

/// What became of a projectile after one tick.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Fate {
    Flying,
    Detonated(Detonation),
    Expired,
}

/// A rocket in flight: a point moving in a straight line at a fixed speed.
///
/// Always finite, never inside a solid it has passed, and it expires after
/// a fixed number of ticks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Projectile {
    previous: Vec3,
    position: Vec3,
    velocity: Vec3,
    ticks_left: u32,
    damage: u16,
}

impl Projectile {
    /// `None` unless `origin` is finite and `direction` is finite and non-zero.
    pub(super) fn launch(
        origin: Vec3,
        direction: Vec3,
        lifetime_ticks: u32,
        damage: u16,
    ) -> Option<Self> {
        let direction = direction.normalize_or_zero();
        (origin.is_finite() && direction != Vec3::ZERO).then_some(Self {
            previous: origin,
            position: origin,
            velocity: direction * PROJECTILE_SPEED,
            ticks_left: lifetime_ticks,
            damage,
        })
    }

    /// Current position.
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Position one tick ago, the start of the interpolation.
    pub fn previous(&self) -> Vec3 {
        self.previous
    }

    /// Where to draw it between the last two ticks; `alpha` is clamped into
    /// `0..=1`, and a non-finite one counts as `0`.
    pub fn position_at(&self, alpha: f32) -> Vec3 {
        let alpha = if alpha.is_finite() {
            alpha.clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.previous.lerp(self.position, alpha)
    }

    /// Moves one tick of `dt` seconds. The move is cast as a segment, so a
    /// thin wall stops the projectile however fast it flies.
    pub(super) fn advance(&mut self, dt: f32, world: &World) -> Fate {
        if self.ticks_left == 0 {
            return Fate::Expired;
        }
        self.ticks_left -= 1;
        self.previous = self.position;
        let delta = self.velocity * dt;
        if let Some(hit) = world.cast(self.position, delta) {
            self.position = hit.point();
            return Fate::Detonated(Detonation {
                position: hit.point(),
                normal: hit.normal(),
                damage: self.damage,
            });
        }
        self.position += delta;
        Fate::Flying
    }
}
