//! The first-person camera: a position, a view direction and a lens.

use std::f32::consts::{FRAC_PI_2, PI};
use std::fmt;

use glam::{Mat4, Vec3};

/// Pitch stops short of straight up or down, where the view direction becomes
/// parallel to the up vector and the view matrix degenerates.
pub const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;

/// Why a [`Projection`] was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionError {
    /// The vertical field of view is not a finite angle in `(0, π)` radians.
    FieldOfView,
    /// The clip planes are not finite with `0 < near < far`.
    ClipPlanes,
}

impl fmt::Display for ProjectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldOfView => f.write_str("field of view must be finite and within (0, π)"),
            Self::ClipPlanes => f.write_str("clip planes must be finite with 0 < near < far"),
        }
    }
}

impl std::error::Error for ProjectionError {}

/// A perspective lens.
///
/// Always has a field of view within `(0, π)` and finite clip planes with
/// `0 < near < far`, so the projection matrix is always finite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Projection {
    fov_y: f32,
    near: f32,
    far: f32,
}

impl Projection {
    /// `fov_y` is the vertical field of view in radians.
    pub fn try_new(fov_y: f32, near: f32, far: f32) -> Result<Self, ProjectionError> {
        if !(fov_y.is_finite() && fov_y > 0.0 && fov_y < PI) {
            return Err(ProjectionError::FieldOfView);
        }
        if !(near.is_finite() && far.is_finite() && 0.0 < near && near < far) {
            return Err(ProjectionError::ClipPlanes);
        }
        Ok(Self { fov_y, near, far })
    }

    /// Vertical field of view in radians.
    pub fn fov_y(&self) -> f32 {
        self.fov_y
    }

    /// Distance to the near clip plane.
    pub fn near(&self) -> f32 {
        self.near
    }

    /// Distance to the far clip plane.
    pub fn far(&self) -> f32 {
        self.far
    }

    /// Right-handed, with clip-space depth in `0..1` as `wgpu` expects.
    pub fn matrix(&self, aspect: f32) -> Mat4 {
        glam::camera::rh::proj::directx::perspective(self.fov_y, aspect, self.near, self.far)
    }
}

/// A view point and direction.
///
/// The world is right-handed with +Y up. Yaw 0 and pitch 0 look down -Z; a
/// positive yaw turns left and a positive pitch looks up.
///
/// `yaw` stays within `-π..=π`, `pitch` within `-PITCH_LIMIT..=PITCH_LIMIT`,
/// and the position is finite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    position: Vec3,
    yaw: f32,
    pitch: f32,
    projection: Projection,
}

impl Camera {
    /// Returns `None` if `position` is not finite.
    pub fn new(position: Vec3, projection: Projection) -> Option<Self> {
        position.is_finite().then_some(Self {
            position,
            yaw: 0.0,
            pitch: 0.0,
            projection,
        })
    }

    /// Current position.
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// Current yaw in radians.
    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    /// Current pitch in radians.
    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    /// The lens.
    pub fn projection(&self) -> &Projection {
        &self.projection
    }

    /// Turns the view by the given angles in radians.
    ///
    /// A non-finite delta is ignored, so one bad input cannot poison the
    /// view for good.
    pub fn turn(&mut self, yaw_delta: f32, pitch_delta: f32) {
        if yaw_delta.is_finite() {
            // Wrapped so a long play session never loses precision to a huge angle.
            self.yaw = wrap_angle(self.yaw + yaw_delta);
        }
        if pitch_delta.is_finite() {
            self.pitch = (self.pitch + pitch_delta).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        }
    }

    /// Places the view exactly, for state that comes from the simulation.
    ///
    /// Ignored entirely if any argument is not finite, so one bad frame
    /// cannot poison the view for good.
    pub fn set_view(&mut self, position: Vec3, yaw: f32, pitch: f32) {
        if !(position.is_finite() && yaw.is_finite() && pitch.is_finite()) {
            return;
        }
        self.position = position;
        self.yaw = wrap_angle(yaw);
        self.pitch = pitch.clamp(-PITCH_LIMIT, PITCH_LIMIT);
    }

    /// Unit vector the camera looks along.
    pub fn forward(&self) -> Vec3 {
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        Vec3::new(-sin_yaw * cos_pitch, sin_pitch, -cos_yaw * cos_pitch)
    }

    /// World-to-clip transform for a target with the given width over height.
    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        let view = glam::camera::rh::view::look_to_mat4(self.position, self.forward(), Vec3::Y);
        self.projection.matrix(aspect) * view
    }
}

/// Maps any finite angle into `-π..=π`.
fn wrap_angle(angle: f32) -> f32 {
    let wrapped = (angle + PI).rem_euclid(2.0 * PI) - PI;
    // `rem_euclid` of a tiny negative can round up to exactly the modulus.
    wrapped.clamp(-PI, PI)
}
