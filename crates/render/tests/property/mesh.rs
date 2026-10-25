use glam::Vec3;
use proptest::prelude::*;
use render::mesh::{Mesh, Vertex};

fn any_vertex() -> impl Strategy<Value = Vertex> {
    // Any bit pattern, so NaN and infinities occur.
    (any::<[f32; 3]>(), any::<[f32; 3]>()).prop_map(|(position, color)| Vertex { position, color })
}

proptest! {
    #[test]
    fn an_accepted_mesh_is_always_safe_to_draw(
        vertices in prop::collection::vec(any_vertex(), 0..20),
        indices in prop::collection::vec(0u32..30, 0..60),
    ) {
        if let Ok(mesh) = Mesh::try_new(vertices, indices) {
            prop_assert!(!mesh.vertices().is_empty());
            prop_assert_eq!(mesh.indices().len() % 3, 0);
            prop_assert!(mesh.indices().iter().all(|&i| (i as usize) < mesh.vertices().len()));
            let finite = mesh
                .vertices()
                .iter()
                .all(|v| v.position.iter().chain(&v.color).all(|c| c.is_finite()));
            prop_assert!(finite);
        }
    }

    #[test]
    fn a_cuboid_exists_exactly_when_min_is_below_max(
        min in any::<[f32; 3]>(),
        max in any::<[f32; 3]>(),
    ) {
        let (min, max) = (Vec3::from(min), Vec3::from(max));
        let valid = min.is_finite() && max.is_finite() && min.cmplt(max).all();
        prop_assert_eq!(Mesh::cuboid(min, max, [1.0; 3]).is_some(), valid);
    }
}
