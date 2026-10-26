//! Bot constants in meters, seconds and radians. Tuning values, not settings yet.

/// Fastest a bot turns its view. Finite on purpose: an instant turn would
/// make every shot land.
pub const TURN_RATE: f32 = 6.0;
/// Past this distance the bot closes in.
pub const PREFERRED_RANGE: f32 = 9.0;
/// Inside this distance the bot backs away.
pub const BACK_OFF_RANGE: f32 = 5.0;
/// Closer than this the bot holds fire, because the blast would reach it too.
pub const MIN_FIRE_RANGE: f32 = 4.0;
/// How far off the target, in radians on each axis, the view may be and
/// still fire.
pub const FIRE_CONE: f32 = 0.08;
/// Shortest time the bot strafes one way.
pub const STRAFE_MIN_SECONDS: f32 = 0.6;
/// Longest time the bot strafes one way.
pub const STRAFE_MAX_SECONDS: f32 = 1.8;
/// How far ahead the bot feels for a wall.
pub const WALL_PROBE: f32 = 0.5;
