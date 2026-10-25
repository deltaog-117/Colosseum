use glam::Vec3;

use super::aabb::Aabb;
use super::world::World;

/// Where a segment first met a solid.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RayHit {
    fraction: f32,
    point: Vec3,
    normal: Vec3,
}

impl RayHit {
    /// How far along the segment the hit lies, within `0..=1`.
    pub fn fraction(&self) -> f32 {
        self.fraction
    }

    /// The hit position.
    pub fn point(&self) -> Vec3 {
        self.point
    }

    /// Unit normal of the face that was hit. For a segment that starts
    /// inside a solid there is no entry face, so this points back along the
    /// segment instead.
    pub fn normal(&self) -> Vec3 {
        self.normal
    }
}

impl World {
    /// The first solid the segment from `origin` to `origin + delta` enters,
    /// or `None` if it flies clear. Exact at any length, so a fast mover
    /// cannot tunnel through a thin wall. A segment that only grazes a face,
    /// or leaves a solid it starts on, hits nothing. Non-finite input hits
    /// nothing either.
    pub fn cast(&self, origin: Vec3, delta: Vec3) -> Option<RayHit> {
        cast_boxes(self.solids(), origin, delta)
    }
}

/// [`World::cast`] against any set of boxes, such as the bodies of targets.
pub fn cast_boxes(boxes: &[Aabb], origin: Vec3, delta: Vec3) -> Option<RayHit> {
    if !(origin.is_finite() && delta.is_finite()) {
        return None;
    }
    let (fraction, entry_normal) = boxes
        .iter()
        .filter_map(|solid| enter(solid, origin, delta))
        .min_by(|a, b| a.0.total_cmp(&b.0))?;
    let normal = entry_normal.unwrap_or_else(|| -delta.normalize_or_zero());
    Some(RayHit {
        fraction,
        point: origin + delta * fraction,
        normal,
    })
}

/// Slab test: the span of the segment parameter inside each pair of
/// parallel faces, intersected over the three axes. Returns the entry
/// fraction (clamped to 0) and the entry face normal, `None` when the
/// segment starts inside the box.
fn enter(solid: &Aabb, origin: Vec3, delta: Vec3) -> Option<(f32, Option<Vec3>)> {
    let (lo, hi) = (solid.min(), solid.max());
    let mut near = f32::NEG_INFINITY;
    let mut far = f32::INFINITY;
    let mut normal = None;
    for axis in 0..3 {
        if delta[axis] == 0.0 {
            // Parallel to these faces: inside the slab for the whole segment, or never.
            if origin[axis] <= lo[axis] || origin[axis] >= hi[axis] {
                return None;
            }
            continue;
        }
        let t_lo = (lo[axis] - origin[axis]) / delta[axis];
        let t_hi = (hi[axis] - origin[axis]) / delta[axis];
        let (t_in, t_out) = if t_lo < t_hi {
            (t_lo, t_hi)
        } else {
            (t_hi, t_lo)
        };
        if t_in > near {
            near = t_in;
            // The face we cross first is the one on the side the segment comes from.
            normal = Some(Vec3::AXES[axis] * -delta[axis].signum());
        }
        far = far.min(t_out);
    }
    // `far > 0` drops boxes behind or only touched on the way out; `near < far`
    // drops a segment that merely grazes an edge.
    if !(near < far && far > 0.0 && near <= 1.0) {
        return None;
    }
    if near < 0.0 {
        Some((0.0, None))
    } else {
        Some((near, normal))
    }
}
