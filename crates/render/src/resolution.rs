//! The low-resolution render target and how it is fitted to the window.

use std::fmt;

/// The target has a zero side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ZeroResolution;

impl fmt::Display for ZeroResolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("render resolution must be at least 1x1")
    }
}

impl std::error::Error for ZeroResolution {}

/// The size of the offscreen target the scene is drawn into.
///
/// Both sides are at least 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderResolution {
    width: u32,
    height: u32,
}

impl RenderResolution {
    /// A PS1-era 4:3 frame.
    pub const DEFAULT: Self = Self {
        width: 320,
        height: 240,
    };

    /// Rejects a zero side, which no texture can have.
    pub fn try_new(width: u32, height: u32) -> Result<Self, ZeroResolution> {
        if width == 0 || height == 0 {
            return Err(ZeroResolution);
        }
        Ok(Self { width, height })
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Width over height, for the projection.
    pub fn aspect(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    /// Where the target lands inside a window of the given size, or `None` for
    /// a window with a zero side (minimized).
    ///
    /// The target is centered and keeps its aspect ratio. It is scaled by the
    /// largest whole factor that fits, so nearest-neighbor filtering gives
    /// even pixels; only a window smaller than the target scales down.
    pub fn fit(&self, window_width: u32, window_height: u32) -> Option<Viewport> {
        if window_width == 0 || window_height == 0 {
            return None;
        }
        // u64: the products below overflow u32 for large windows.
        let (w, h) = (u64::from(self.width), u64::from(self.height));
        let (ww, wh) = (u64::from(window_width), u64::from(window_height));

        let (width, height) = if ww >= w && wh >= h {
            let scale = (ww / w).min(wh / h);
            (w * scale, h * scale)
        } else if ww * h <= wh * w {
            (ww, (h * ww / w).max(1))
        } else {
            ((w * wh / h).max(1), wh)
        };

        Some(Viewport {
            x: ((ww - width) / 2) as u32,
            y: ((wh - height) / 2) as u32,
            width: width as u32,
            height: height as u32,
        })
    }
}

/// A rectangle inside the window, in pixels from its top-left corner.
///
/// Lies within the window it was fitted to and has a non-zero size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Viewport {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Viewport {
    /// Left edge.
    pub fn x(&self) -> u32 {
        self.x
    }

    /// Top edge.
    pub fn y(&self) -> u32 {
        self.y
    }

    /// Width in pixels.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    pub fn height(&self) -> u32 {
        self.height
    }
}
