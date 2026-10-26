use glam::{Vec2, Vec3};

use super::tuning::{
    BACK_OFF_RANGE, FIRE_CONE, MIN_FIRE_RANGE, PREFERRED_RANGE, STRAFE_MAX_SECONDS,
    STRAFE_MIN_SECONDS, TURN_RATE, WALL_PROBE,
};
use crate::collision::{Axis3, World};
use crate::input::{Axis, Command};
use crate::movement::tuning::{LOOK_SENSITIVITY, PITCH_LIMIT};
use crate::movement::{Player, wrap_angle};
use crate::tick::TickRate;
use crate::weapon::tuning::PROJECTILE_SPEED;

/// Something a bot can shoot at: where it is and how fast it moves.
///
/// Always finite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Target {
    center: Vec3,
    velocity: Vec3,
}

impl Target {
    /// Returns `None` if either vector is not finite.
    pub fn try_new(center: Vec3, velocity: Vec3) -> Option<Self> {
        (center.is_finite() && velocity.is_finite()).then_some(Self { center, velocity })
    }

    /// The middle of the body.
    pub fn center(&self) -> Vec3 {
        self.center
    }

    /// Velocity in meters per second.
    pub fn velocity(&self) -> Vec3 {
        self.velocity
    }

    /// The candidate closest to `from`; the earliest wins a tie.
    pub fn nearest<'a>(
        from: Vec3,
        candidates: impl IntoIterator<Item = &'a Self>,
    ) -> Option<&'a Self> {
        candidates.into_iter().min_by(|a, b| {
            let (da, db) = (
                a.center.distance_squared(from),
                b.center.distance_squared(from),
            );
            da.total_cmp(&db)
        })
    }
}

/// The reflex that drives a bot: face the target, keep a fighting
/// distance, strafe, hop over what blocks the way and shoot when lined up.
///
/// It knows nothing of the level beyond what a wall probe and a line of
/// sight tell it, so it can be stopped by a dead-end; navigation replaces
/// the walking later without touching the aiming.
#[derive(Debug, Clone)]
pub struct Brain {
    /// Radians the view may turn in one tick.
    max_turn: f32,
    /// The side the bot strafes toward; never `Zero`.
    strafe: Axis,
    strafe_left: u32,
    strafe_min_ticks: u32,
    strafe_span_ticks: u32,
    /// State of a xorshift generator; never zero. Seeded, so a bot's
    /// choices replay exactly.
    rng: u32,
}

impl Brain {
    /// A brain for ticks at `rate`. Bots with different `seed`s strafe out
    /// of step with each other.
    pub fn new(rate: TickRate, seed: u32) -> Self {
        let hz = rate.hz() as f32;
        let ticks = |seconds: f32| ((seconds * hz).ceil() as u32).max(1);
        let strafe_min_ticks = ticks(STRAFE_MIN_SECONDS);
        let mut brain = Self {
            max_turn: TURN_RATE / hz,
            strafe: Axis::Positive,
            strafe_left: 0,
            strafe_min_ticks,
            strafe_span_ticks: ticks(STRAFE_MAX_SECONDS)
                .saturating_sub(strafe_min_ticks)
                .max(1),
            rng: seed.max(1),
        };
        brain.pick_strafe();
        brain
    }

    /// What `body` does this tick to fight `target`. With no target, or a
    /// body whose state is not finite, the bot stands idle.
    pub fn decide(&mut self, body: &Player, target: Option<&Target>, world: &World) -> Command {
        let (Some(target), Some(pose)) = (target, body.pose()) else {
            return Command::default();
        };
        let eye = pose.eye();
        let distance = target.center.distance(eye);

        // Lead the target by the rocket's flight time, so a runner is hit
        // where they are going rather than where they were.
        let aim = target.center + target.velocity * (distance / PROJECTILE_SPEED) - eye;
        let want_yaw = (-aim.x).atan2(-aim.z);
        let want_pitch = aim
            .y
            .atan2(Vec2::new(aim.x, aim.z).length())
            .clamp(-PITCH_LIMIT, PITCH_LIMIT);
        let yaw_error = wrap_angle(want_yaw - pose.yaw());
        let pitch_error = want_pitch - pose.pitch();

        let in_sight = world.cast(eye, target.center - eye).is_none();
        let fire = in_sight
            && distance >= MIN_FIRE_RANGE
            && yaw_error.abs() <= FIRE_CONE
            && pitch_error.abs() <= FIRE_CONE;

        let forward = if distance > PREFERRED_RANGE {
            Axis::Positive
        } else if distance < BACK_OFF_RANGE {
            Axis::Negative
        } else {
            Axis::Zero
        };
        self.tick_strafe();
        let jump = blocked_ahead(body, pose.yaw(), forward, self.strafe, world);
        if jump {
            // Hop, and slide off the other way: pressing on only repeats the bump.
            self.strafe = opposite(self.strafe);
        }

        // The player's own rule turns counts into radians, so inverting it
        // here keeps a bot's turn rate honest. Device Y and yaw both flip.
        let yaw_step = yaw_error.clamp(-self.max_turn, self.max_turn);
        let pitch_step = pitch_error.clamp(-self.max_turn, self.max_turn);
        Command::from_intent(
            forward,
            self.strafe,
            jump,
            fire,
            -yaw_step / LOOK_SENSITIVITY,
            -pitch_step / LOOK_SENSITIVITY,
        )
    }

    fn tick_strafe(&mut self) {
        if self.strafe_left == 0 {
            self.pick_strafe();
        }
        self.strafe_left -= 1;
    }

    fn pick_strafe(&mut self) {
        let roll = self.roll();
        self.strafe = if roll & 1 == 0 {
            Axis::Positive
        } else {
            Axis::Negative
        };
        self.strafe_left = self.strafe_min_ticks + (roll >> 1) % self.strafe_span_ticks;
    }

    fn roll(&mut self) -> u32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x
    }
}

fn opposite(axis: Axis) -> Axis {
    match axis {
        Axis::Positive => Axis::Negative,
        _ => Axis::Positive,
    }
}

/// True if a wall stops most of the bot's next step. Feels along each
/// horizontal axis, because a sweep moves one axis at a time.
fn blocked_ahead(body: &Player, yaw: f32, forward: Axis, strafe: Axis, world: &World) -> bool {
    // The same direction `Player` builds from a command.
    let (sin, cos) = yaw.sin_cos();
    let ahead = Vec3::new(-sin, 0.0, -cos);
    let right = Vec3::new(cos, 0.0, -sin);
    let wish = (ahead * forward.value() + right * strafe.value()).normalize_or_zero();
    if wish == Vec3::ZERO {
        return false;
    }
    let probe = wish * WALL_PROBE;
    let free = Vec2::new(
        world.sweep(body.bounds(), Axis3::X, probe.x).distance(),
        world.sweep(body.bounds(), Axis3::Z, probe.z).distance(),
    );
    free.length() < WALL_PROBE * 0.5
}
