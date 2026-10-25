//! Weapon constants in meters and seconds. Tuning values, not settings yet.

/// Rocket speed; a bit under three times the player's top speed, so a rocket
/// outruns a player but can still be dodged across a room.
pub const PROJECTILE_SPEED: f32 = 25.0;
/// Time between shots.
pub const COOLDOWN_SECONDS: f32 = 0.8;
/// How long a rocket flies before it fizzles out in open air.
pub const LIFETIME_SECONDS: f32 = 5.0;
/// Rockets in the launcher at spawn.
pub const START_AMMO: u16 = 10;
/// Damage dealt by a direct hit. Carried by the projectile; applying it is
/// the job of hit detection.
pub const DAMAGE: u16 = 100;
/// Most rockets in flight at once. The cooldown and lifetime allow far fewer;
/// this only bounds memory if they are ever tuned apart.
pub const MAX_IN_FLIGHT: usize = 32;
