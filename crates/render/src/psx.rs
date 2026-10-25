//! The PS1 look: vertex snapping, fog and dithering, as validated settings.
//!
//! The settings are packed into a small uniform the scene shader reads, so
//! the look can be tuned (or switched off) without touching WGSL.

use std::fmt;
use std::num::NonZeroU32;

use crate::resolution::RenderResolution;

/// Why a [`Fog`] or [`PsxSettings`] was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PsxError {
    /// Fog start is negative or not below its end, or either is not finite.
    FogRange,
    /// A fog color component is outside `0..=1` or not finite.
    FogColor,
    /// Dither strength is outside `0..=1` or not finite.
    DitherStrength,
}

impl fmt::Display for PsxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::FogRange => "fog needs finite distances with 0 <= start < end",
            Self::FogColor => "fog color components must be finite and within 0..=1",
            Self::DitherStrength => "dither strength must be finite and within 0..=1",
        })
    }
}

impl std::error::Error for PsxError {}

/// Distance fog: geometry fades to a color between two view depths.
///
/// `0 <= start < end`, and the color is linear RGB within `0..=1`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fog {
    start: f32,
    end: f32,
    color: [f32; 3],
}

impl Fog {
    /// Checks every invariant of [`Fog`]. Distances are in world units
    /// along the view direction.
    pub fn try_new(start: f32, end: f32, color: [f32; 3]) -> Result<Self, PsxError> {
        // `!(a < b)` rather than `a >= b`, so NaN is rejected too.
        if !(start.is_finite() && end.is_finite() && start >= 0.0 && start < end) {
            return Err(PsxError::FogRange);
        }
        if !color.iter().all(|c| (0.0..=1.0).contains(c)) {
            return Err(PsxError::FogColor);
        }
        Ok(Self { start, end, color })
    }

    /// Depth at which fog begins.
    pub fn start(&self) -> f32 {
        self.start
    }

    /// Depth at which geometry is fully fogged.
    pub fn end(&self) -> f32 {
        self.end
    }

    /// Linear RGB of the fog.
    pub fn color(&self) -> [f32; 3] {
        self.color
    }
}

/// How the scene is dressed up as a PS1 frame.
///
/// Each effect can be off: no snapping, no fog, or zero dither strength.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PsxSettings {
    snap_steps_per_pixel: Option<NonZeroU32>,
    fog: Option<Fog>,
    dither_strength: f32,
}

impl PsxSettings {
    /// Checks every invariant of [`PsxSettings`].
    ///
    /// `snap_steps_per_pixel` is how many grid cells a target pixel holds:
    /// `1` snaps vertices to whole pixels like the original hardware, larger
    /// values snap more finely. `dither_strength` is `0` (off) to `1`.
    pub fn try_new(
        snap_steps_per_pixel: Option<NonZeroU32>,
        fog: Option<Fog>,
        dither_strength: f32,
    ) -> Result<Self, PsxError> {
        if !(0.0..=1.0).contains(&dither_strength) {
            return Err(PsxError::DitherStrength);
        }
        Ok(Self {
            snap_steps_per_pixel,
            fog,
            dither_strength,
        })
    }

    /// Every effect off: the scene is drawn with plain vertex colors.
    pub const fn disabled() -> Self {
        Self {
            snap_steps_per_pixel: None,
            fog: None,
            dither_strength: 0.0,
        }
    }

    /// Grid cells per target pixel, or `None` when vertices are not snapped.
    pub fn snap_steps_per_pixel(&self) -> Option<NonZeroU32> {
        self.snap_steps_per_pixel
    }

    /// The fog, if any.
    pub fn fog(&self) -> Option<Fog> {
        self.fog
    }

    /// Dither strength; `0` means off.
    pub fn dither_strength(&self) -> f32 {
        self.dither_strength
    }

    /// Packs the settings for the shader's `Psx` uniform, as three `vec4`s:
    ///
    /// 1. fog color in `xyz`;
    /// 2. fog start, one over the fog span, fog on (`1` or `0`), dither strength;
    /// 3. snap grid width and height in cells, snap on (`1` or `0`).
    ///
    /// The snap grid depends on the target size, so it is packed per resolution.
    pub fn pack(&self, resolution: RenderResolution) -> [[f32; 4]; 3] {
        let (fog_color, fog_start, fog_scale, fog_on) = match self.fog {
            Some(fog) => (fog.color, fog.start, 1.0 / (fog.end - fog.start), 1.0),
            None => ([0.0; 3], 0.0, 0.0, 0.0),
        };
        let (grid_x, grid_y, snap_on) = match self.snap_steps_per_pixel {
            Some(steps) => {
                let steps = steps.get() as f32;
                (
                    resolution.width() as f32 * steps,
                    resolution.height() as f32 * steps,
                    1.0,
                )
            }
            None => (0.0, 0.0, 0.0),
        };
        [
            [fog_color[0], fog_color[1], fog_color[2], 0.0],
            [fog_start, fog_scale, fog_on, self.dither_strength],
            [grid_x, grid_y, snap_on, 0.0],
        ]
    }
}

impl Default for PsxSettings {
    /// The shipped look: whole-pixel snapping, dark fog and full dithering.
    fn default() -> Self {
        Self {
            snap_steps_per_pixel: NonZeroU32::new(1),
            fog: Some(Fog {
                start: 12.0,
                end: 45.0,
                color: DEFAULT_FOG_COLOR,
            }),
            dither_strength: 1.0,
        }
    }
}

/// Matches the renderer's clear color, so fogged geometry meets the
/// background without a seam.
const DEFAULT_FOG_COLOR: [f32; 3] = [0.02, 0.02, 0.03];
