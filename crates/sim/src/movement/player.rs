use std::fmt;

use glam::{Vec2, Vec3};

use super::pose::{Pose, wrap_angle};
use super::tuning::{
    ACCELERATION, AIR_SPEED_CAP, EYE_HEIGHT, FRICTION, GRAVITY, GROUND_PROBE, HALF_WIDTH, HEIGHT,
    JUMP_SPEED, LOOK_SENSITIVITY, MAX_SPEED, PITCH_LIMIT, STOP_SPEED, TERMINAL_SPEED,
};
use crate::collision::{Aabb, Axis3, World};
use crate::input::{Command, Look};
use crate::tick::TickRate;

/// Why a [`Player`] could not be placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnError {
    /// The spawn point is not finite.
    NonFinite,
    /// The player's box would overlap a solid there.
    Blocked,
}

impl fmt::Display for SpawnError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::NonFinite => "spawn point is not finite",
            Self::Blocked => "spawn point is inside a solid",
        })
    }
}

impl std::error::Error for SpawnError {}

/// The player's body: a box that moves through a [`World`].
///
/// The box never overlaps a solid, its velocity stays within
/// `±TERMINAL_SPEED` per axis, `yaw` within `-π..=π` and `pitch` within
/// `±PITCH_LIMIT`. While `on_ground`, vertical velocity is zero.
#[derive(Debug, Clone)]
pub struct Player {
    bounds: Aabb,
    velocity: Vec3,
    yaw: f32,
    pitch: f32,
    on_ground: bool,
    dt: f32,
}

impl Player {
    /// Places the player with its feet at `feet`, at rest and looking down -Z.
    ///
    /// # Errors
    ///
    /// Returns [`SpawnError`] if `feet` is not finite or the player's box
    /// would overlap a solid there.
    pub fn try_spawn(feet: Vec3, world: &World, rate: TickRate) -> Result<Self, SpawnError> {
        if !feet.is_finite() {
            return Err(SpawnError::NonFinite);
        }
        let half = Vec3::new(HALF_WIDTH, HEIGHT * 0.5, HALF_WIDTH);
        let bounds =
            Aabb::try_new(feet + Vec3::Y * half.y, half).map_err(|_| SpawnError::NonFinite)?;
        if world.is_blocked(&bounds) {
            return Err(SpawnError::Blocked);
        }
        Ok(Self {
            bounds,
            velocity: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: false,
            dt: rate.step().as_secs_f32(),
        })
    }

    /// Advances the player by one tick.
    pub fn step(&mut self, command: &Command, world: &World) {
        self.turn(command.look());
        let wish = self.wish_direction(command);

        // Jumping before friction leaves the ground speed untouched, which is
        // what lets chained jumps keep their speed.
        if self.on_ground && command.jump() {
            self.velocity.y = JUMP_SPEED;
            self.on_ground = false;
        }
        if self.on_ground {
            self.apply_friction();
            self.accelerate(wish, MAX_SPEED);
        } else {
            self.accelerate(wish, AIR_SPEED_CAP);
            self.velocity.y -= GRAVITY * self.dt;
        }
        self.velocity = self
            .velocity
            .clamp(Vec3::splat(-TERMINAL_SPEED), Vec3::splat(TERMINAL_SPEED));

        self.slide(world);
        self.settle(world);
    }

    /// Adds `impulse` to the velocity, as a blast does. An upward push lifts
    /// the player off the ground. A non-finite impulse is ignored, and the
    /// velocity keeps its `TERMINAL_SPEED` limit.
    pub fn push(&mut self, impulse: Vec3) {
        if !impulse.is_finite() {
            return;
        }
        self.velocity = (self.velocity + impulse)
            .clamp(Vec3::splat(-TERMINAL_SPEED), Vec3::splat(TERMINAL_SPEED));
        if impulse.y > 0.0 {
            self.on_ground = false;
        }
    }

    /// Position of the feet, the middle of the bottom face.
    pub fn position(&self) -> Vec3 {
        self.bounds.center() - Vec3::Y * self.bounds.half_extents().y
    }

    /// The box that collides with the world.
    pub fn bounds(&self) -> &Aabb {
        &self.bounds
    }

    /// Velocity in meters per second.
    pub fn velocity(&self) -> Vec3 {
        self.velocity
    }

    /// Yaw in radians.
    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    /// Pitch in radians.
    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    /// True while standing on a solid.
    pub fn on_ground(&self) -> bool {
        self.on_ground
    }

    /// The view for rendering. `None` only if the state is not finite, which
    /// the invariants rule out.
    pub fn pose(&self) -> Option<Pose> {
        Pose::try_new(self.position() + Vec3::Y * EYE_HEIGHT, self.yaw, self.pitch)
    }

    fn turn(&mut self, look: Look) {
        // Device Y grows downward and a positive yaw turns left, so both flip.
        self.yaw = wrap_angle(self.yaw - look.dx() * LOOK_SENSITIVITY);
        self.pitch = (self.pitch - look.dy() * LOOK_SENSITIVITY).clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }

    /// Unit horizontal direction the player asks to go, or zero. Normalized
    /// so a diagonal is no faster than a straight line.
    fn wish_direction(&self, command: &Command) -> Vec3 {
        let (sin, cos) = self.yaw.sin_cos();
        let forward = Vec3::new(-sin, 0.0, -cos);
        let right = Vec3::new(cos, 0.0, -sin);
        (forward * command.forward().value() + right * command.strafe().value()).normalize_or_zero()
    }

    fn apply_friction(&mut self) {
        let speed = Vec2::new(self.velocity.x, self.velocity.z).length();
        if speed <= 0.0 {
            return;
        }
        let drop = speed.max(STOP_SPEED) * FRICTION * self.dt;
        let scale = (speed - drop).max(0.0) / speed;
        self.velocity.x *= scale;
        self.velocity.z *= scale;
    }

    /// Quake's accelerate: only the speed gained along `wish` counts, up to
    /// `speed_cap`. Speed across `wish` is untouched, which is what allows
    /// air-strafing past `MAX_SPEED`.
    fn accelerate(&mut self, wish: Vec3, speed_cap: f32) {
        if wish == Vec3::ZERO {
            return;
        }
        let missing = speed_cap - self.velocity.dot(wish);
        if missing <= 0.0 {
            return;
        }
        let gain = (ACCELERATION * MAX_SPEED * self.dt).min(missing);
        self.velocity += wish * gain;
    }

    /// Moves one axis at a time, so a wall stops only the part of the motion
    /// that runs into it and the rest slides along.
    fn slide(&mut self, world: &World) {
        for axis in [Axis3::X, Axis3::Z, Axis3::Y] {
            let sweep = world.sweep(&self.bounds, axis, self.velocity[axis.index()] * self.dt);
            if let Some(moved) = self.bounds.translated(axis.unit() * sweep.distance()) {
                self.bounds = moved;
            }
            if sweep.hit() {
                self.velocity[axis.index()] = 0.0;
            }
        }
    }

    /// Decides whether the player stands on something, and pulls it down
    /// onto the surface so it does not hover within the probe distance.
    fn settle(&mut self, world: &World) {
        self.on_ground = false;
        if self.velocity.y > 0.0 {
            return;
        }
        let probe = world.sweep(&self.bounds, Axis3::Y, -GROUND_PROBE);
        if !probe.hit() {
            return;
        }
        if let Some(moved) = self.bounds.translated(Vec3::Y * probe.distance()) {
            self.bounds = moved;
        }
        self.velocity.y = 0.0;
        self.on_ground = true;
    }
}
