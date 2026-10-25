//! Drawn geometry: glTF primitives as flat-colored render meshes.

use std::fmt;

use glam::{Mat4, Vec3};
use gltf::mesh::Mode;
use render::mesh::{Mesh, MeshError, Vertex};

use super::buffer_data;

/// Why a mesh cannot be drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VisualError {
    /// A primitive is not a triangle list.
    NotTriangles,
    /// A primitive has no readable positions.
    MissingPositions,
    /// The normal count differs from the position count.
    NormalCount,
    /// The renderer rejected the geometry.
    Mesh(MeshError),
}

impl fmt::Display for VisualError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotTriangles => f.write_str("a primitive is not a triangle list"),
            Self::MissingPositions => f.write_str("a primitive has no readable positions"),
            Self::NormalCount => f.write_str("the normal count differs from the position count"),
            Self::Mesh(err) => write!(f, "invalid geometry: {err}"),
        }
    }
}

impl std::error::Error for VisualError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Mesh(err) => Some(err),
            Self::NotTriangles | Self::MissingPositions | Self::NormalCount => None,
        }
    }
}

/// One render mesh per primitive of `mesh`, placed by `world`.
pub(super) fn meshes(
    mesh: &gltf::Mesh,
    world: Mat4,
    blob: Option<&[u8]>,
) -> Result<Vec<Mesh>, VisualError> {
    mesh.primitives()
        .map(|primitive| {
            if primitive.mode() != Mode::Triangles {
                return Err(VisualError::NotTriangles);
            }
            let reader = primitive.reader(|buffer| buffer_data(blob, buffer));
            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .ok_or(VisualError::MissingPositions)?
                .collect();
            let normals: Option<Vec<[f32; 3]>> = reader.read_normals().map(Iterator::collect);
            // An unindexed primitive lists its triangles back to back.
            let indices: Vec<u32> = match reader.read_indices() {
                Some(indices) => indices.into_u32().collect(),
                None => {
                    let count = u32::try_from(positions.len())
                        .map_err(|_| VisualError::Mesh(MeshError::IndexOutOfBounds))?;
                    (0..count).collect()
                }
            };
            let [r, g, b, _] = primitive
                .material()
                .pbr_metallic_roughness()
                .base_color_factor();
            build(&positions, normals.as_deref(), indices, [r, g, b], world)
        })
        .collect()
}

/// Places `positions` by `world` and shades each vertex by its normal.
pub(super) fn build(
    positions: &[[f32; 3]],
    normals: Option<&[[f32; 3]]>,
    indices: Vec<u32>,
    color: [f32; 3],
    world: Mat4,
) -> Result<Mesh, VisualError> {
    if normals.is_some_and(|n| n.len() != positions.len()) {
        return Err(VisualError::NormalCount);
    }
    let vertices = positions
        .iter()
        .enumerate()
        .map(|(i, &p)| {
            let shade = normals.map_or(1.0, |n| shade(world.transform_vector3(Vec3::from(n[i]))));
            Vertex {
                position: world.transform_point3(Vec3::from(p)).to_array(),
                color: color.map(|c| c * shade),
            }
        })
        .collect();
    Mesh::try_new(vertices, indices).map_err(VisualError::Mesh)
}

/// Brightness for a face looking along `normal`. The renderer has no lights
/// yet and flat color hides the shape, so faces are shaded by orientation,
/// the same way as the placeholder boxes in `render::mesh`.
pub(super) fn shade(normal: Vec3) -> f32 {
    let a = normal.abs();
    if !(a.is_finite() && a.max_element() > 0.0) {
        return 1.0;
    }
    if a.y >= a.x && a.y >= a.z {
        if normal.y > 0.0 { 1.0 } else { 0.45 }
    } else if a.z >= a.x {
        0.8
    } else {
        0.65
    }
}
