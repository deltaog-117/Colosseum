use std::f32::consts::PI;

use glam::Vec3;

/// Where the player's eyes are and which way they look, for the renderer.
///
/// Always finite. The world is right-handed with +Y up; yaw 0 and pitch 0
/// look down -Z, a positive yaw turns left and a positive pitch looks up.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    eye: Vec3,
    yaw: f32,
    pitch: f32,
}

impl Pose {
    /// Returns `None` if any component is not finite.
    pub fn try_new(eye: Vec3, yaw: f32, pitch: f32) -> Option<Self> {
        (eye.is_finite() && yaw.is_finite() && pitch.is_finite()).then_some(Self {
            eye,
            yaw,
            pitch,
        })
    }

    /// Eye position.
    pub fn eye(&self) -> Vec3 {
        self.eye
    }

    /// Yaw in radians.
    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    /// Pitch in radians.
    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    /// Unit vector the view looks along.
    pub fn forward(&self) -> Vec3 {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        Vec3::new(-sin_yaw * cos_pitch, sin_pitch, -cos_yaw * cos_pitch)
    }

    /// Blends toward `other`; `alpha` is clamped into `0..=1`, and a
    /// non-finite one counts as `0`. Yaw takes the short way around, so a
    /// turn across ±π does not spin the view the long way.
    pub fn lerp(&self, other: &Self, alpha: f32) -> Self {
        let alpha = if alpha.is_finite() {
            alpha.clamp(0.0, 1.0)
        } else {
            0.0
        };
        Self {
            eye: self.eye.lerp(other.eye, alpha),
            yaw: wrap_angle(self.yaw + wrap_angle(other.yaw - self.yaw) * alpha),
            pitch: self.pitch + (other.pitch - self.pitch) * alpha,
        }
    }
}

/// Maps any finite angle into `-π..=π`.
pub(super) fn wrap_angle(angle: f32) -> f32 {
    let wrapped = (angle + PI).rem_euclid(2.0 * PI) - PI;
    // `rem_euclid` of a tiny negative can round up to exactly the modulus.
    wrapped.clamp(-PI, PI)
}
