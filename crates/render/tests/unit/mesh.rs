use glam::Vec3;
use render::mesh::{Mesh, MeshError, Vertex};

fn vertex(x: f32) -> Vertex {
    Vertex {
        position: [x, 0.0, 0.0],
        color: [1.0; 3],
    }
}

#[test]
fn a_triangle_is_accepted() {
    let vertices = vec![vertex(0.0), vertex(1.0), vertex(2.0)];
    assert!(Mesh::try_new(vertices, vec![0, 1, 2]).is_ok());
}

#[test]
fn empty_input_is_rejected() {
    assert_eq!(Mesh::try_new(vec![], vec![]), Err(MeshError::Empty));
    assert_eq!(
        Mesh::try_new(vec![vertex(0.0)], vec![]),
        Err(MeshError::Empty)
    );
}

#[test]
fn a_partial_triangle_is_rejected() {
    let vertices = vec![vertex(0.0), vertex(1.0), vertex(2.0)];
    assert_eq!(
        Mesh::try_new(vertices, vec![0, 1]),
        Err(MeshError::PartialTriangle)
    );
}

#[test]
fn an_index_past_the_end_is_rejected() {
    let vertices = vec![vertex(0.0), vertex(1.0), vertex(2.0)];
    assert_eq!(
        Mesh::try_new(vertices, vec![0, 1, 3]),
        Err(MeshError::IndexOutOfBounds)
    );
}

#[test]
fn a_non_finite_component_is_rejected() {
    let mut bad = vertex(0.0);
    bad.color[2] = f32::NAN;
    let vertices = vec![bad, vertex(1.0), vertex(2.0)];
    assert_eq!(
        Mesh::try_new(vertices, vec![0, 1, 2]),
        Err(MeshError::NonFinite)
    );
}

#[test]
fn a_cuboid_has_six_quads() {
    let Some(mesh) = Mesh::cuboid(Vec3::ZERO, Vec3::ONE, [0.5; 3]) else {
        panic!("unit cube is a valid cuboid");
    };
    assert_eq!(mesh.vertices().len(), 24);
    assert_eq!(mesh.indices().len(), 36);
}

#[test]
fn a_flat_cuboid_is_rejected() {
    assert!(Mesh::cuboid(Vec3::ZERO, Vec3::new(1.0, 0.0, 1.0), [1.0; 3]).is_none());
    assert!(Mesh::cuboid(Vec3::ONE, Vec3::ZERO, [1.0; 3]).is_none());
}
