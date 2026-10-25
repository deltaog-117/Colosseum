use glam::Vec3;
use proptest::prelude::*;
use render::camera::{Camera, PITCH_LIMIT, Projection};
use std::f32::consts::PI;

fn camera() -> Camera {
    let Ok(projection) = Projection::try_new(1.4, 0.1, 200.0) else {
        panic!("fixture lens is valid");
    };
    let Some(camera) = Camera::new(Vec3::new(0.0, 1.7, 0.0), projection) else {
        panic!("fixture position is finite");
    };
    camera
}

proptest! {
    #[test]
    fn angles_stay_in_range_for_any_turns(
        // Any bit pattern, so NaN, infinities and extremes all occur.
        turns in prop::collection::vec((any::<f32>(), any::<f32>()), 0..50),
    ) {
        let mut camera = camera();
        for (yaw, pitch) in turns {
            camera.turn(yaw, pitch);
            prop_assert!((-PI..=PI).contains(&camera.yaw()), "yaw {}", camera.yaw());
            prop_assert!(
                (-PITCH_LIMIT..=PITCH_LIMIT).contains(&camera.pitch()),
                "pitch {}",
                camera.pitch()
            );
        }
    }

    #[test]
    fn forward_is_a_unit_vector_and_the_matrix_is_finite(
        yaw in -1.0e4f32..1.0e4,
        pitch in -1.0e4f32..1.0e4,
        aspect in 0.1f32..10.0,
    ) {
        let mut camera = camera();
        camera.turn(yaw, pitch);
        prop_assert!((camera.forward().length() - 1.0).abs() < 1e-4);
        prop_assert!(camera.view_projection(aspect).to_cols_array().iter().all(|c| c.is_finite()));
    }

    #[test]
    fn projection_accepts_exactly_the_valid_lenses(
        fov in any::<f32>(),
        near in any::<f32>(),
        far in any::<f32>(),
    ) {
        let valid = fov.is_finite() && fov > 0.0 && fov < PI
            && near.is_finite() && far.is_finite() && 0.0 < near && near < far;
        prop_assert_eq!(Projection::try_new(fov, near, far).is_ok(), valid);
    }
}
