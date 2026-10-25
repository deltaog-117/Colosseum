//! Static colored meshes, validated once at construction.

use std::fmt;

use bytemuck::{Pod, Zeroable};
use glam::Vec3;

/// One corner of a triangle: a position and a linear RGB color.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Pod, Zeroable)]
pub struct Vertex {
    /// World-space position.
    pub position: [f32; 3],
    /// Linear RGB.
    pub color: [f32; 3],
}

/// Why a [`Mesh`] was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshError {
    /// No vertices or no indices.
    Empty,
    /// The index count is not a multiple of three.
    PartialTriangle,
    /// An index points past the last vertex.
    IndexOutOfBounds,
    /// A position or color component is NaN or infinite.
    NonFinite,
}

impl fmt::Display for MeshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Empty => "mesh has no vertices or no indices",
            Self::PartialTriangle => "index count is not a multiple of three",
            Self::IndexOutOfBounds => "an index points past the last vertex",
            Self::NonFinite => "a vertex component is not finite",
        })
    }
}

impl std::error::Error for MeshError {}

/// An indexed triangle list.
///
/// Never empty, every index is in range and every component is finite, so
/// the GPU can draw it without further checks.
#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Mesh {
    /// Checks every invariant of [`Mesh`].
    pub fn try_new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Result<Self, MeshError> {
        if vertices.is_empty() || indices.is_empty() {
            return Err(MeshError::Empty);
        }
        if !indices.len().is_multiple_of(3) {
            return Err(MeshError::PartialTriangle);
        }
        if vertices
            .iter()
            .flat_map(|v| v.position.into_iter().chain(v.color))
            .any(|c| !c.is_finite())
        {
            return Err(MeshError::NonFinite);
        }
        // A `usize` count that does not fit `u32` cannot be indexed anyway.
        let count = u32::try_from(vertices.len()).map_err(|_| MeshError::IndexOutOfBounds)?;
        if indices.iter().any(|&i| i >= count) {
            return Err(MeshError::IndexOutOfBounds);
        }
        Ok(Self { vertices, indices })
    }

    /// An axis-aligned box. Faces are shaded by orientation, since the
    /// renderer has no lights yet and flat color hides the shape.
    ///
    /// Returns `None` unless `min < max` on every axis and all inputs are
    /// finite.
    pub fn cuboid(min: Vec3, max: Vec3, color: [f32; 3]) -> Option<Self> {
        if !(min.is_finite() && max.is_finite() && min.cmplt(max).all()) {
            return None;
        }
        let corner = |x: bool, y: bool, z: bool| {
            Vec3::new(
                if x { max.x } else { min.x },
                if y { max.y } else { min.y },
                if z { max.z } else { min.z },
            )
        };
        // Each face lists its four corners; the index buffer is the same for all.
        let faces: [([Vec3; 4], f32); 6] = [
            (
                [
                    corner(false, true, false),
                    corner(false, true, true),
                    corner(true, true, true),
                    corner(true, true, false),
                ],
                1.0,
            ),
            (
                [
                    corner(false, false, false),
                    corner(true, false, false),
                    corner(true, false, true),
                    corner(false, false, true),
                ],
                0.45,
            ),
            (
                [
                    corner(false, false, false),
                    corner(false, true, false),
                    corner(true, true, false),
                    corner(true, false, false),
                ],
                0.8,
            ),
            (
                [
                    corner(false, false, true),
                    corner(true, false, true),
                    corner(true, true, true),
                    corner(false, true, true),
                ],
                0.8,
            ),
            (
                [
                    corner(false, false, false),
                    corner(false, false, true),
                    corner(false, true, true),
                    corner(false, true, false),
                ],
                0.65,
            ),
            (
                [
                    corner(true, false, false),
                    corner(true, true, false),
                    corner(true, true, true),
                    corner(true, false, true),
                ],
                0.65,
            ),
        ];

        let mut vertices = Vec::with_capacity(24);
        let mut indices = Vec::with_capacity(36);
        for (corners, shade) in faces {
            let base = vertices.len() as u32;
            vertices.extend(corners.map(|p| Vertex {
                position: p.to_array(),
                color: color.map(|c| c * shade),
            }));
            indices.extend([0, 1, 2, 0, 2, 3].map(|i| base + i));
        }
        Self::try_new(vertices, indices).ok()
    }

    /// The corners.
    pub fn vertices(&self) -> &[Vertex] {
        &self.vertices
    }

    /// Triangle corners as indices into [`Self::vertices`].
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }
}
