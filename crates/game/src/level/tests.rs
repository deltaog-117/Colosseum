use std::error::Error;

use glam::{Mat4, Vec3};
use proptest::prelude::*;
use sim::movement::Player;
use sim::rules::PickupKind;
use sim::tick::TickRate;

use super::collider::{ColliderError, bounds};
use super::visual::shade;
use super::*;

type TestResult = Result<(), Box<dyn Error>>;

const UNIT_CUBE: [[f32; 3]; 8] = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [1.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
    [1.0, 0.0, 1.0],
    [0.0, 1.0, 1.0],
    [1.0, 1.0, 1.0],
];

/// A `.glb` whose only mesh (index 0) is the unit cube's eight corners, with
/// the given nodes (a JSON array body) in its one scene.
fn glb(nodes: &str) -> Vec<u8> {
    glb_with_roots("0,1,2,3", nodes)
}

fn glb_with_roots(roots: &str, nodes: &str) -> Vec<u8> {
    let bin: Vec<u8> = UNIT_CUBE
        .iter()
        .flatten()
        .flat_map(|c| c.to_le_bytes())
        .collect();
    let json = format!(
        r#"{{"asset":{{"version":"2.0"}},"scene":0,"scenes":[{{"nodes":[{roots}]}}],
        "nodes":[{nodes}],
        "meshes":[{{"primitives":[{{"attributes":{{"POSITION":0}}}}]}}],
        "accessors":[{{"bufferView":0,"componentType":5126,"count":8,"type":"VEC3",
            "min":[0,0,0],"max":[1,1,1]}}],
        "bufferViews":[{{"buffer":0,"byteLength":{len}}}],
        "buffers":[{{"byteLength":{len}}}]}}"#,
        len = bin.len(),
        roots = roots
    );
    let mut json = json.into_bytes();
    json.resize(json.len().next_multiple_of(4), b' ');
    let total = 12 + 8 + json.len() + 8 + bin.len();
    let mut out = Vec::with_capacity(total);
    out.extend(b"glTF");
    out.extend(2u32.to_le_bytes());
    out.extend((total as u32).to_le_bytes());
    out.extend((json.len() as u32).to_le_bytes());
    out.extend(b"JSON");
    out.extend(json);
    out.extend((bin.len() as u32).to_le_bytes());
    out.extend(b"BIN\0");
    out.extend(bin);
    out
}

const BOX: &str = r#"{"name":"col_box","mesh":0}"#;
const SPAWN: &str = r#"{"name":"spawn","translation":[1,2,3]}"#;
const FILLER: &str = r#"{"name":"nothing"}"#;

fn nodes(list: [&str; 4]) -> String {
    list.join(",")
}

#[test]
fn the_arena_loads() -> TestResult {
    let level = arena()?;
    assert_eq!(level.meshes().len(), 6);
    assert_eq!(level.world().solids().len(), 10);
    assert_eq!(level.spawns().first(), Vec3::new(0.0, 0.0, 8.0));
    assert_eq!(level.spawns().points().len(), 4);
    assert_eq!(level.pickups().len(), 4);
    Ok(())
}

#[test]
fn the_arena_floor_is_the_first_solid() -> TestResult {
    let level = arena()?;
    let floor = level.world().solids()[0];
    assert_eq!(floor.min(), Vec3::new(-13.0, -1.0, -13.0));
    assert_eq!(floor.max(), Vec3::new(13.0, 0.0, 13.0));
    Ok(())
}

#[test]
fn every_arena_spawn_is_free() -> TestResult {
    let level = arena()?;
    for point in level.spawns().points() {
        Player::try_spawn(*point, level.world(), TickRate::DEFAULT)?;
    }
    Ok(())
}

#[test]
fn every_arena_pickup_stands_in_open_air_and_both_kinds_exist() -> TestResult {
    let level = arena()?;
    for (_, base) in level.pickups() {
        let cube = Aabb::try_new(*base + Vec3::Y * 0.35, Vec3::splat(0.35))?;
        assert!(
            level
                .world()
                .solids()
                .iter()
                .all(|solid| !solid.overlaps(&cube)),
            "pickup at {base} is inside a solid"
        );
    }
    let kinds: Vec<_> = level.pickups().iter().map(|(kind, _)| *kind).collect();
    assert!(kinds.contains(&PickupKind::Health));
    assert!(kinds.contains(&PickupKind::Ammo));
    Ok(())
}

#[test]
fn a_collider_and_a_spawn_make_a_level() -> TestResult {
    let level = load(&glb(&nodes([BOX, SPAWN, FILLER, FILLER])))?;
    assert!(level.meshes().is_empty());
    assert_eq!(level.world().solids().len(), 1);
    assert_eq!(level.spawns().first(), Vec3::new(1.0, 2.0, 3.0));
    Ok(())
}

#[test]
fn a_parent_transform_places_its_children() -> TestResult {
    let parent = r#"{"translation":[10,0,0],"children":[2]}"#;
    let child = r#"{"name":"col_box","mesh":0}"#;
    let level = load(&glb_with_roots("0,1", &[parent, SPAWN, child].join(",")))?;
    assert_eq!(level.world().solids()[0].min(), Vec3::new(10.0, 0.0, 0.0));
    Ok(())
}

#[test]
fn garbage_is_not_a_level() {
    assert!(matches!(load(b"not a glb"), Err(LevelError::Parse(_))));
}

#[test]
fn a_level_without_a_spawn_is_rejected() {
    let result = load(&glb(&nodes([BOX, FILLER, FILLER, FILLER])));
    assert!(matches!(result, Err(LevelError::NoSpawn)));
}

#[test]
fn every_spawn_node_is_kept_in_file_order() -> TestResult {
    let second = r#"{"name":"spawn_1","translation":[4,5,6]}"#;
    let level = load(&glb(&nodes([BOX, SPAWN, second, FILLER])))?;
    assert_eq!(
        level.spawns().points(),
        [Vec3::new(1.0, 2.0, 3.0), Vec3::new(4.0, 5.0, 6.0)]
    );
    Ok(())
}

#[test]
fn pickup_nodes_are_read_by_kind() -> TestResult {
    let health = r#"{"name":"pickup_health_0","translation":[1,0,0]}"#;
    let ammo = r#"{"name":"pickup_ammo_7","translation":[0,0,2]}"#;
    let level = load(&glb(&nodes([BOX, SPAWN, health, ammo])))?;
    assert_eq!(
        level.pickups(),
        [
            (PickupKind::Health, Vec3::new(1.0, 0.0, 0.0)),
            (PickupKind::Ammo, Vec3::new(0.0, 0.0, 2.0)),
        ]
    );
    Ok(())
}

#[test]
fn a_level_without_colliders_is_rejected() {
    let result = load(&glb(&nodes([SPAWN, FILLER, FILLER, FILLER])));
    assert!(matches!(result, Err(LevelError::NoColliders)));
}

#[test]
fn a_turned_collider_is_rejected() {
    let turned = r#"{"name":"col_box","mesh":0,"rotation":[0,0.3826834,0,0.9238795]}"#;
    let result = load(&glb(&nodes([turned, SPAWN, FILLER, FILLER])));
    assert!(matches!(
        result,
        Err(LevelError::Collider {
            source: ColliderError::NotAxisAligned,
            ..
        })
    ));
}

#[test]
fn a_quarter_turned_collider_is_accepted() -> TestResult {
    let quarter = r#"{"name":"col_box","mesh":0,"rotation":[0,0.7071068,0,0.7071068]}"#;
    let level = load(&glb(&nodes([quarter, SPAWN, FILLER, FILLER])))?;
    assert_eq!(level.world().solids().len(), 1);
    Ok(())
}

#[test]
fn a_flat_collider_is_rejected() {
    let flat = r#"{"name":"col_box","mesh":0,"scale":[1,0.0,1]}"#;
    let result = load(&glb(&nodes([flat, SPAWN, FILLER, FILLER])));
    assert!(matches!(result, Err(LevelError::Collider { .. })));
}

#[test]
fn an_error_names_the_node() {
    let flat = r#"{"name":"col_box","mesh":0,"scale":[1,0.0,1]}"#;
    let message = load(&glb(&nodes([flat, SPAWN, FILLER, FILLER])))
        .err()
        .map(|err| err.to_string());
    assert!(message.is_some_and(|m| m.contains("col_box")));
}

#[test]
fn faces_are_shaded_by_orientation() {
    assert_eq!(shade(Vec3::Y), 1.0);
    assert_eq!(shade(Vec3::NEG_Y), 0.45);
    assert_eq!(shade(Vec3::Z), 0.8);
    assert_eq!(shade(Vec3::X), 0.65);
    assert_eq!(shade(Vec3::ZERO), 1.0);
}

/// The three local axes land on distinct world axes, with signs and scales.
fn axis_aligned_matrices() -> impl Strategy<Value = Mat4> {
    (
        prop::sample::select(vec![
            [0, 1, 2],
            [0, 2, 1],
            [1, 0, 2],
            [1, 2, 0],
            [2, 0, 1],
            [2, 1, 0],
        ]),
        prop::array::uniform3(prop::bool::ANY),
        prop::array::uniform3(0.25f32..8.0),
        prop::array::uniform3(-50.0f32..50.0),
    )
        .prop_map(|(perm, flips, scales, shift)| {
            let column = |i: usize| {
                let mut v = Vec3::ZERO;
                v[perm[i]] = if flips[i] { -scales[i] } else { scales[i] };
                v
            };
            Mat4::from_cols(
                column(0).extend(0.0),
                column(1).extend(0.0),
                column(2).extend(0.0),
                Vec3::from(shift).extend(1.0),
            )
        })
}

fn corners(lo: Vec3, size: Vec3) -> Vec<[f32; 3]> {
    UNIT_CUBE
        .iter()
        .map(|&c| (lo + Vec3::from(c) * size).to_array())
        .collect()
}

proptest! {
    #[test]
    fn a_collider_contains_every_placed_vertex(
        world in axis_aligned_matrices(),
        lo in prop::array::uniform3(-20.0f32..20.0),
        size in prop::array::uniform3(0.1f32..20.0),
    ) {
        let points = corners(Vec3::from(lo), Vec3::from(size));
        let solid = bounds(&points, world)
            .map_err(|err| TestCaseError::fail(err.to_string()))?;
        for p in points {
            let placed = world.transform_point3(Vec3::from(p));
            prop_assert!(placed.cmpge(solid.min() - Vec3::splat(1.0e-3)).all());
            prop_assert!(placed.cmple(solid.max() + Vec3::splat(1.0e-3)).all());
        }
    }

    #[test]
    fn an_axis_aligned_transform_keeps_the_volume(
        world in axis_aligned_matrices(),
        lo in prop::array::uniform3(-20.0f32..20.0),
        size in prop::array::uniform3(0.1f32..20.0),
    ) {
        let points = corners(Vec3::from(lo), Vec3::from(size));
        let solid = bounds(&points, world)
            .map_err(|err| TestCaseError::fail(err.to_string()))?;
        let extent = solid.half_extents() * 2.0;
        let expected = Vec3::from(size).element_product() * world.determinant().abs();
        let got = extent.element_product();
        prop_assert!((got - expected).abs() <= expected * 1.0e-3);
    }

    #[test]
    fn a_turn_off_the_axes_is_never_accepted(
        degrees in 5.0f32..85.0,
        lo in prop::array::uniform3(-20.0f32..20.0),
        size in prop::array::uniform3(0.1f32..20.0),
    ) {
        let points = corners(Vec3::from(lo), Vec3::from(size));
        let world = Mat4::from_rotation_y(degrees.to_radians());
        prop_assert_eq!(bounds(&points, world), Err(ColliderError::NotAxisAligned));
    }
}
