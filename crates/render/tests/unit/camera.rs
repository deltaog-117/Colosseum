use std::f32::consts::{FRAC_PI_2, PI};

use glam::{Vec3, Vec4};
use render::camera::{Camera, PITCH_LIMIT, Projection, ProjectionError};

fn projection() -> Projection {
    match Projection::try_new(FRAC_PI_2, 0.1, 100.0) {
        Ok(projection) => projection,
        Err(err) => panic!("fixture lens is valid: {err}"),
    }
}

fn camera() -> Camera {
    let Some(camera) = Camera::new(Vec3::new(1.0, 2.0, 3.0), projection()) else {
        panic!("fixture position is finite");
    };
    camera
}

#[test]
fn projection_rejects_bad_field_of_view() {
    for fov in [0.0, -1.0, PI, 4.0, f32::NAN, f32::INFINITY] {
        assert_eq!(
            Projection::try_new(fov, 0.1, 10.0),
            Err(ProjectionError::FieldOfView),
            "fov {fov}"
        );
    }
}

#[test]
fn projection_rejects_bad_clip_planes() {
    for (near, far) in [
        (0.0, 10.0),
        (-1.0, 10.0),
        (5.0, 5.0),
        (5.0, 1.0),
        (0.1, f32::NAN),
    ] {
        assert_eq!(
            Projection::try_new(1.0, near, far),
            Err(ProjectionError::ClipPlanes),
            "near {near} far {far}"
        );
    }
}

#[test]
fn camera_rejects_a_non_finite_position() {
    assert!(Camera::new(Vec3::new(f32::NAN, 0.0, 0.0), projection()).is_none());
}

#[test]
fn a_fresh_camera_looks_down_negative_z() {
    let forward = camera().forward();
    assert!((forward - Vec3::NEG_Z).length() < 1e-6, "{forward}");
}

#[test]
fn positive_yaw_turns_left() {
    let mut camera = camera();
    camera.turn(FRAC_PI_2, 0.0);
    assert!((camera.forward() - Vec3::NEG_X).length() < 1e-6);
}

#[test]
fn positive_pitch_looks_up() {
    let mut camera = camera();
    camera.turn(0.0, 0.5);
    assert!(camera.forward().y > 0.0);
}

#[test]
fn pitch_stops_before_straight_up_and_down() {
    let mut camera = camera();
    camera.turn(0.0, 10.0);
    assert_eq!(camera.pitch(), PITCH_LIMIT);
    camera.turn(0.0, -20.0);
    assert_eq!(camera.pitch(), -PITCH_LIMIT);
}

#[test]
fn non_finite_turns_are_ignored() {
    let mut camera = camera();
    camera.turn(0.3, 0.2);
    let before = camera;
    camera.turn(f32::NAN, f32::INFINITY);
    assert_eq!(camera, before);
}

#[test]
fn a_point_ahead_lands_inside_the_clip_volume() {
    let camera = camera();
    let ahead = camera.position() + camera.forward() * 10.0;
    let clip = camera.view_projection(4.0 / 3.0) * Vec4::from((ahead, 1.0));
    let ndc = clip.truncate() / clip.w;
    assert!(ndc.x.abs() < 1e-5 && ndc.y.abs() < 1e-5, "{ndc}");
    assert!((0.0..=1.0).contains(&ndc.z), "{ndc}");
}

#[test]
fn set_view_places_the_camera_exactly() {
    let mut camera = camera();
    camera.set_view(Vec3::new(4.0, 5.0, 6.0), 0.5, -0.25);
    assert_eq!(camera.position(), Vec3::new(4.0, 5.0, 6.0));
    assert_eq!((camera.yaw(), camera.pitch()), (0.5, -0.25));
}

#[test]
fn set_view_ignores_non_finite_input() {
    let mut camera = camera();
    let before = camera;
    camera.set_view(Vec3::new(f32::NAN, 0.0, 0.0), 0.5, 0.0);
    camera.set_view(Vec3::ZERO, f32::INFINITY, 0.0);
    camera.set_view(Vec3::ZERO, 0.0, f32::NAN);
    assert_eq!(camera, before);
}

#[test]
fn set_view_clamps_pitch_and_wraps_yaw() {
    let mut camera = camera();
    camera.set_view(Vec3::ZERO, 3.0 * PI, 10.0);
    assert_eq!(camera.pitch(), PITCH_LIMIT);
    assert!((-PI..=PI).contains(&camera.yaw()));
}
