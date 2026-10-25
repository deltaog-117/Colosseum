//! Combat constants. Tuning values, not settings yet.

/// Health of a fresh body.
pub const MAX_HEALTH: u16 = 100;
/// How long a dead body waits before it comes back.
pub const RESPAWN_SECONDS: f32 = 2.0;
/// Share of a blast's damage the shooter takes from their own rocket, in
/// percent, so a rocket jump hurts but does not kill from full health.
pub const SELF_DAMAGE_PERCENT: u16 = 50;
