//! The heads-up display as data: rectangles and text collected into a flat
//! list of textured quads, which the renderer draws over the scene.
//!
//! Coordinates are pixels of the low-resolution target, with the origin at
//! the top-left corner and `y` growing downward.

pub mod font;

use bytemuck::{Pod, Zeroable};

/// Most quads one batch holds. A quad past the limit is dropped, which only
/// loses HUD detail and cannot corrupt the frame; the buffers on the GPU are
/// sized for exactly this many.
pub const MAX_QUADS: usize = 4096;

/// Pixels a glyph advances the pen, in unscaled pixels: five wide plus a gap.
const ADVANCE: u32 = 6;
/// The empty column that trails the last glyph of a text.
const TRAILING_GAP: u32 = 1;

/// One corner of a HUD quad.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct HudVertex {
    /// Target pixels.
    pub position: [f32; 2],
    /// Atlas coordinates.
    pub uv: [f32; 2],
    /// Linear RGBA, not premultiplied.
    pub color: [f32; 4],
}

/// The quads of one frame of HUD.
///
/// Holds a whole number of quads, four vertices each, never more than
/// [`MAX_QUADS`], and every component is finite.
#[derive(Debug, Clone, Default)]
pub struct HudBatch {
    vertices: Vec<HudVertex>,
}

impl HudBatch {
    /// An empty batch.
    pub fn new() -> Self {
        Self::default()
    }

    /// Empties the batch but keeps its memory, so a frame allocates nothing
    /// once the HUD has settled.
    pub fn clear(&mut self) {
        self.vertices.clear();
    }

    /// The vertices, four per quad, in draw order.
    pub fn vertices(&self) -> &[HudVertex] {
        &self.vertices
    }

    /// How many quads are in the batch.
    pub fn quad_count(&self) -> usize {
        self.vertices.len() / 4
    }

    /// Adds a filled rectangle. Ignored if a number is not finite or the
    /// size is not positive.
    pub fn rect(&mut self, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
        if width <= 0.0 || height <= 0.0 {
            return;
        }
        let [u0, v0, u1, v1] = font::cell_uv(font::SOLID_CELL);
        // Every corner samples the middle of the solid cell, where it is
        // fully opaque whatever the filtering.
        let (u, v) = ((u0 + u1) / 2.0, (v0 + v1) / 2.0);
        self.quad([x, y], [width, height], [u, v, u, v], color);
    }

    /// Adds `text` with its top-left corner at `(x, y)`, each glyph
    /// `scale` times its natural size. Ignored if `scale` is 0 or a number
    /// is not finite.
    pub fn text(&mut self, x: f32, y: f32, scale: u32, color: [f32; 4], text: &str) {
        if scale == 0 || !x.is_finite() || !y.is_finite() {
            return;
        }
        // Whole pixels, so scaled glyphs keep even texels.
        let (x, y) = (x.round(), y.round());
        let size = (font::CELL * scale) as f32;
        let advance = (ADVANCE * scale) as f32;
        for (index, ch) in text.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let uv = font::cell_uv(font::cell_for(ch));
            self.quad([x + index as f32 * advance, y], [size, size], uv, color);
        }
    }

    fn quad(&mut self, origin: [f32; 2], size: [f32; 2], uv: [f32; 4], color: [f32; 4]) {
        let finite = origin
            .iter()
            .chain(&size)
            .chain(&color)
            .all(|v| v.is_finite());
        if !finite || self.quad_count() >= MAX_QUADS {
            return;
        }
        let [x, y] = origin;
        let [u0, v0, u1, v1] = uv;
        let corner = |dx: f32, dy: f32, u, v| HudVertex {
            position: [x + dx * size[0], y + dy * size[1]],
            uv: [u, v],
            color,
        };
        self.vertices.extend([
            corner(0.0, 0.0, u0, v0),
            corner(1.0, 0.0, u1, v0),
            corner(0.0, 1.0, u0, v1),
            corner(1.0, 1.0, u1, v1),
        ]);
    }
}

/// Width in pixels of `text` drawn at `scale`, without the gap after the
/// last glyph, so it can be centered or right-aligned exactly. Saturates
/// rather than overflowing for absurd input.
pub fn text_width(text: &str, scale: u32) -> u32 {
    let glyphs = u32::try_from(text.chars().count()).unwrap_or(u32::MAX);
    if glyphs == 0 {
        return 0;
    }
    glyphs
        .saturating_mul(ADVANCE)
        .saturating_sub(TRAILING_GAP)
        .saturating_mul(scale)
}
