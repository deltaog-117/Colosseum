//! Movement constants in meters and seconds. Tuning values, not settings yet.

use std::f32::consts::FRAC_PI_2;

/// Half the width of the player's box.
pub const HALF_WIDTH: f32 = 0.4;
/// Height of the player's box, feet to head.
pub const HEIGHT: f32 = 1.8;
/// Height of the eyes above the feet.
pub const EYE_HEIGHT: f32 = 1.6;

/// Top speed on the ground, and the speed target of ground acceleration.
pub const MAX_SPEED: f32 = 8.0;
/// How quickly input pulls velocity toward the wish direction, per second.
pub const ACCELERATION: f32 = 10.0;
/// Rate at which the ground slows an idle player, per second.
pub const FRICTION: f32 = 6.0;
/// Below this speed friction acts as if the player were this fast, so slow
/// sliding ends in a stop instead of an endless crawl.
pub const STOP_SPEED: f32 = 2.5;
/// Air acceleration only counts speed along the wish direction up to this
/// much. Small on purpose: it is what makes air-strafing curve a jump
/// without letting the player steer freely.
pub const AIR_SPEED_CAP: f32 = 0.85;
/// Downward acceleration while airborne.
pub const GRAVITY: f32 = 22.0;
/// Upward speed given by a jump.
pub const JUMP_SPEED: f32 = 7.5;
/// Per-axis speed limit, which keeps a long fall or a chain of strafe jumps
/// finite.
pub const TERMINAL_SPEED: f32 = 50.0;
/// How close to a floor the player counts as standing on it.
pub const GROUND_PROBE: f32 = 0.02;

/// Radians of turn per device count.
pub const LOOK_SENSITIVITY: f32 = 0.0022;
/// Pitch stops short of straight up or down, where the view direction
/// becomes parallel to the up vector. Matches the renderer's camera limit.
pub const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;
