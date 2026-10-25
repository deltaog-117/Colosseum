//! Placeholder level: a room with a few pillars, until real level geometry
//! is loaded. The meshes and the collision world are built from the same
//! constants, so what is drawn is what is solid.

use glam::Vec3;
use render::mesh::Mesh;
use sim::collision::{Aabb, World};

const FLOOR: [f32; 3] = [0.30, 0.30, 0.32];
const WALL: [f32; 3] = [0.55, 0.55, 0.58];
const PILLAR: [f32; 3] = [0.65, 0.35, 0.25];

const ROOM_HALF: f32 = 12.0;
const ROOM_HEIGHT: f32 = 6.0;
/// Thickness of the collision walls around the room. Invisible, and thick
/// enough that nothing could ever be pushed through.
const SHELL: f32 = 1.0;
const PILLAR_HEIGHT: f32 = 4.0;
const PILLAR_CENTERS: [(f32, f32); 4] = [(-5.0, -5.0), (5.0, -5.0), (-5.0, 3.0), (5.0, 3.0)];

/// Where the player's feet start, on the floor inside the room.
pub(crate) const SPAWN: Vec3 = Vec3::new(0.0, 0.0, 8.0);

pub(crate) fn meshes() -> Vec<Mesh> {
    let room = Mesh::cuboid(
        Vec3::new(-ROOM_HALF, 0.0, -ROOM_HALF),
        Vec3::new(ROOM_HALF, ROOM_HEIGHT, ROOM_HALF),
        WALL,
    );
    let pillars = PILLAR_CENTERS.map(|(x, z)| {
        let (min, max) = pillar_corners(x, z);
        Mesh::cuboid(min, max, PILLAR)
    });
    // The room is a box seen from the inside; its floor is a slab lying just
    // above it so the floor reads darker than the walls.
    let floor = Mesh::cuboid(
        Vec3::new(-ROOM_HALF, 0.0, -ROOM_HALF),
        Vec3::new(ROOM_HALF, 0.05, ROOM_HALF),
        FLOOR,
    );
    // The inputs are constants known to be valid, so a `None` is a bug here.
    [room, floor].into_iter().chain(pillars).flatten().collect()
}

fn pillar_corners(x: f32, z: f32) -> (Vec3, Vec3) {
    (
        Vec3::new(x - 1.0, 0.0, z - 1.0),
        Vec3::new(x + 1.0, PILLAR_HEIGHT, z + 1.0),
    )
}

/// The room as solid boxes: six slabs that wrap it, plus the pillars.
pub(crate) fn world() -> World {
    let (h, t, top) = (ROOM_HALF, SHELL, ROOM_HEIGHT);
    let o = h + t;
    let shell = [
        (Vec3::new(-o, -t, -o), Vec3::new(o, 0.0, o)),
        (Vec3::new(-o, top, -o), Vec3::new(o, top + t, o)),
        (Vec3::new(-o, -t, -o), Vec3::new(-h, top + t, o)),
        (Vec3::new(h, -t, -o), Vec3::new(o, top + t, o)),
        (Vec3::new(-o, -t, -o), Vec3::new(o, top + t, -h)),
        (Vec3::new(-o, -t, h), Vec3::new(o, top + t, o)),
    ];
    let pillars = PILLAR_CENTERS.map(|(x, z)| pillar_corners(x, z));
    // The inputs are constants known to be valid, so a dropped box is a bug.
    World::new(
        shell
            .into_iter()
            .chain(pillars)
            .filter_map(|(min, max)| Aabb::from_corners(min, max).ok())
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use sim::movement::Player;
    use sim::tick::TickRate;

    use super::*;

    #[test]
    fn every_placeholder_mesh_is_valid() {
        // `meshes` drops invalid cuboids, so a short list means a bad constant.
        assert_eq!(meshes().len(), 6);
    }

    #[test]
    fn every_placeholder_solid_is_valid() {
        // `world` drops invalid boxes, so a short list means a bad constant.
        assert_eq!(world().solids().len(), 10);
    }

    #[test]
    fn the_spawn_point_is_free() {
        assert!(Player::try_spawn(SPAWN, &world(), TickRate::DEFAULT).is_ok());
    }
}
