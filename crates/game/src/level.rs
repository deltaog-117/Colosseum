//! Levels: a glTF binary read into render meshes, a collision world and a
//! spawn point.
//!
//! Authoring contract, so one file drives both what is drawn and what is
//! solid:
//! - a node named `col_*` with a mesh is a box collider, invisible, and its
//!   transform must keep it axis-aligned (translation, scale, quarter turns);
//! - a node named `spawn` marks where the player's feet start;
//! - any other node with a mesh is drawn.

mod collider;
mod visual;

use std::fmt;

use glam::{Mat4, Vec3};
use gltf::buffer::Source;
use gltf::{Buffer, Gltf, Node};
use render::mesh::Mesh;
use sim::collision::{Aabb, World};

use self::collider::ColliderError;
use self::visual::VisualError;

const ARENA: &[u8] = include_bytes!("../../../assets/levels/arena.glb");
const COLLIDER_PREFIX: &str = "col_";
const SPAWN_NAME: &str = "spawn";

/// Why a level was rejected.
#[derive(Debug)]
pub(crate) enum LevelError {
    /// The bytes are not a valid glTF binary.
    Parse(gltf::Error),
    /// The file has no scene to read.
    NoScene,
    /// A `col_*` node cannot be turned into a solid box.
    Collider { node: String, source: ColliderError },
    /// A drawn node cannot be turned into a mesh.
    Visual { node: String, source: VisualError },
    /// No `spawn` node.
    NoSpawn,
    /// More than one `spawn` node.
    DuplicateSpawn,
    /// No `col_*` node, so the player would fall out of the world.
    NoColliders,
}

impl fmt::Display for LevelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(err) => write!(f, "not a valid glTF binary: {err}"),
            Self::NoScene => f.write_str("the level has no scene"),
            Self::Collider { node, source } => write!(f, "collider `{node}`: {source}"),
            Self::Visual { node, source } => write!(f, "mesh `{node}`: {source}"),
            Self::NoSpawn => f.write_str("the level has no `spawn` node"),
            Self::DuplicateSpawn => f.write_str("the level has more than one `spawn` node"),
            Self::NoColliders => f.write_str("the level has no `col_*` node"),
        }
    }
}

impl std::error::Error for LevelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Parse(err) => Some(err),
            Self::Collider { source, .. } => Some(source),
            Self::Visual { source, .. } => Some(source),
            Self::NoScene | Self::NoSpawn | Self::DuplicateSpawn | Self::NoColliders => None,
        }
    }
}

/// A loaded level: always has a spawn point and at least one solid.
#[derive(Debug)]
pub(crate) struct Level {
    meshes: Vec<Mesh>,
    world: World,
    spawn: Vec3,
}

impl Level {
    /// What to draw.
    pub(crate) fn meshes(&self) -> &[Mesh] {
        &self.meshes
    }

    /// What is solid.
    pub(crate) fn world(&self) -> &World {
        &self.world
    }

    /// Where the player's feet start.
    pub(crate) fn spawn(&self) -> Vec3 {
        self.spawn
    }
}

/// The arena shipped with the game.
pub(crate) fn arena() -> Result<Level, LevelError> {
    load(ARENA)
}

/// Reads a glTF binary (`.glb`) with its buffer embedded.
pub(crate) fn load(bytes: &[u8]) -> Result<Level, LevelError> {
    let Gltf { document, blob } = Gltf::from_slice(bytes).map_err(LevelError::Parse)?;
    let scene = document
        .default_scene()
        .or_else(|| document.scenes().next())
        .ok_or(LevelError::NoScene)?;

    let mut parts = Parts::default();
    for node in scene.nodes() {
        parts.visit(&node, Mat4::IDENTITY, blob.as_deref())?;
    }
    if parts.solids.is_empty() {
        return Err(LevelError::NoColliders);
    }
    Ok(Level {
        meshes: parts.meshes,
        world: World::new(parts.solids),
        spawn: parts.spawn.ok_or(LevelError::NoSpawn)?,
    })
}

#[derive(Default)]
struct Parts {
    meshes: Vec<Mesh>,
    solids: Vec<Aabb>,
    spawn: Option<Vec3>,
}

impl Parts {
    fn visit(&mut self, node: &Node, parent: Mat4, blob: Option<&[u8]>) -> Result<(), LevelError> {
        let world = parent * Mat4::from_cols_array_2d(&node.transform().matrix());
        let name = node.name().unwrap_or_default();
        let label = || {
            node.name()
                .map_or_else(|| format!("#{}", node.index()), str::to_owned)
        };

        if name == SPAWN_NAME {
            if self.spawn.replace(world.w_axis.truncate()).is_some() {
                return Err(LevelError::DuplicateSpawn);
            }
        } else if let Some(mesh) = node.mesh() {
            if name.starts_with(COLLIDER_PREFIX) {
                let solid =
                    collider::solid(&mesh, world, blob).map_err(|source| LevelError::Collider {
                        node: label(),
                        source,
                    })?;
                self.solids.push(solid);
            } else {
                let drawn =
                    visual::meshes(&mesh, world, blob).map_err(|source| LevelError::Visual {
                        node: label(),
                        source,
                    })?;
                self.meshes.extend(drawn);
            }
        }
        node.children()
            .try_for_each(|child| self.visit(&child, world, blob))
    }
}

/// The bytes of a buffer. Only the embedded binary chunk is supported, so a
/// level is one self-contained file.
fn buffer_data<'a>(blob: Option<&'a [u8]>, buffer: Buffer<'_>) -> Option<&'a [u8]> {
    match buffer.source() {
        Source::Bin => blob,
        Source::Uri(_) => None,
    }
}

#[cfg(test)]
mod tests;
