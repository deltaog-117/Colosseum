//! Box colliders: the bounds of a mesh, as one solid.

use std::fmt;

use glam::{Mat4, Vec3};
use sim::collision::{Aabb, AabbError};

use super::buffer_data;

/// How far off an axis a transformed column may be, relative to its length.
/// Quarter turns exported as floats are never exact.
const AXIS_TOLERANCE: f32 = 1.0e-4;

/// Why a mesh cannot become a solid box.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColliderError {
    /// A primitive has no readable positions.
    MissingPositions,
    /// The node's transform turns or shears the mesh off the axes, so its
    /// bounds would be bigger than the shape.
    NotAxisAligned,
    /// The bounds are not a valid box.
    Bounds(AabbError),
}

impl fmt::Display for ColliderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPositions => f.write_str("a primitive has no readable positions"),
            Self::NotAxisAligned => f.write_str("the transform is not axis-aligned"),
            Self::Bounds(err) => write!(f, "invalid bounds: {err}"),
        }
    }
}

impl std::error::Error for ColliderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Bounds(err) => Some(err),
            Self::MissingPositions | Self::NotAxisAligned => None,
        }
    }
}

/// The box that bounds every vertex of `mesh` once placed by `world`.
pub(super) fn solid(
    mesh: &gltf::Mesh,
    world: Mat4,
    blob: Option<&[u8]>,
) -> Result<Aabb, ColliderError> {
    let mut points = Vec::new();
    for primitive in mesh.primitives() {
        let reader = primitive.reader(|buffer| buffer_data(blob, buffer));
        points.extend(
            reader
                .read_positions()
                .ok_or(ColliderError::MissingPositions)?,
        );
    }
    bounds(&points, world)
}

/// The box that bounds `points` once placed by `world`.
pub(super) fn bounds(points: &[[f32; 3]], world: Mat4) -> Result<Aabb, ColliderError> {
    if points.is_empty() {
        return Err(ColliderError::MissingPositions);
    }
    if !is_axis_aligned(&world) {
        return Err(ColliderError::NotAxisAligned);
    }
    let (min, max) = points
        .iter()
        .map(|&p| world.transform_point3(Vec3::from(p)))
        .fold((Vec3::INFINITY, Vec3::NEG_INFINITY), |(lo, hi), p| {
            (lo.min(p), hi.max(p))
        });
    Aabb::from_corners(min, max).map_err(ColliderError::Bounds)
}

/// True if each local axis lands on exactly one world axis.
fn is_axis_aligned(world: &Mat4) -> bool {
    [world.x_axis, world.y_axis, world.z_axis]
        .iter()
        .all(|column| {
            let column = column.truncate();
            let off_axis = column.length() * AXIS_TOLERANCE;
            column
                .abs()
                .to_array()
                .iter()
                .filter(|&&c| c > off_axis)
                .count()
                == 1
        })
}
