//! Rules constants in meters and seconds. Tuning values, not settings yet.

/// Length of the playing part of a round.
pub const ROUND_SECONDS: f32 = 180.0;
/// How long the final score stays up before the next round starts.
pub const INTERMISSION_SECONDS: f32 = 8.0;
/// Hit points a health pickup restores.
pub const HEALTH_PICKUP_AMOUNT: u16 = 25;
/// Rockets an ammo pickup adds.
pub const AMMO_PICKUP_AMOUNT: u16 = 5;
/// Wait before a taken health pickup is back.
pub const HEALTH_PICKUP_RESPAWN_SECONDS: f32 = 20.0;
/// Wait before a taken ammo pickup is back.
pub const AMMO_PICKUP_RESPAWN_SECONDS: f32 = 15.0;
/// Half the edge of the cube a pickup is touched through; generous, so
/// running past one is enough.
pub const PICKUP_HALF_SIZE: f32 = 0.35;
