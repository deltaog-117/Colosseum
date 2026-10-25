//! Placeholder level: a room with a few pillars, until real geometry and
//! collision exist in `sim`.

use glam::Vec3;
use render::mesh::Mesh;

const FLOOR: [f32; 3] = [0.30, 0.30, 0.32];
const WALL: [f32; 3] = [0.55, 0.55, 0.58];
const PILLAR: [f32; 3] = [0.65, 0.35, 0.25];

/// Where the player starts, standing inside the room.
pub(crate) const SPAWN: Vec3 = Vec3::new(0.0, 1.7, 8.0);

pub(crate) fn meshes() -> Vec<Mesh> {
    let room = Mesh::cuboid(
        Vec3::new(-12.0, 0.0, -12.0),
        Vec3::new(12.0, 6.0, 12.0),
        WALL,
    );
    let pillars = [(-5.0, -5.0), (5.0, -5.0), (-5.0, 3.0), (5.0, 3.0)].map(|(x, z)| {
        Mesh::cuboid(
            Vec3::new(x - 1.0, 0.0, z - 1.0),
            Vec3::new(x + 1.0, 4.0, z + 1.0),
            PILLAR,
        )
    });
    // The room is a box seen from the inside; its floor is a slab lying just
    // above it so the floor reads darker than the walls.
    let floor = Mesh::cuboid(
        Vec3::new(-12.0, 0.0, -12.0),
        Vec3::new(12.0, 0.05, 12.0),
        FLOOR,
    );
    // The inputs are constants known to be valid, so a `None` is a bug here.
    [room, floor].into_iter().chain(pillars).flatten().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_placeholder_mesh_is_valid() {
        // `meshes` drops invalid cuboids, so a short list means a bad constant.
        assert_eq!(meshes().len(), 6);
    }
}
