use std::num::NonZeroU32;

use render::psx::{Fog, PsxError, PsxSettings};
use render::resolution::RenderResolution;

fn fog(start: f32, end: f32) -> Result<Fog, PsxError> {
    Fog::try_new(start, end, [0.1, 0.2, 0.3])
}

#[test]
fn fog_rejects_bad_ranges() {
    for (start, end) in [
        (-1.0, 10.0),
        (5.0, 5.0),
        (6.0, 5.0),
        (f32::NAN, 10.0),
        (0.0, f32::NAN),
        (0.0, f32::INFINITY),
    ] {
        assert_eq!(fog(start, end), Err(PsxError::FogRange), "{start}..{end}");
    }
}

#[test]
fn fog_rejects_bad_colors() {
    for color in [[1.1, 0.0, 0.0], [0.0, -0.1, 0.0], [0.0, 0.0, f32::NAN]] {
        assert_eq!(Fog::try_new(1.0, 2.0, color), Err(PsxError::FogColor));
    }
}

#[test]
fn settings_reject_bad_dither_strength() {
    for strength in [-0.1, 1.1, f32::NAN, f32::INFINITY] {
        assert_eq!(
            PsxSettings::try_new(None, None, strength),
            Err(PsxError::DitherStrength),
            "strength {strength}"
        );
    }
}

#[test]
fn disabled_settings_pack_every_effect_off() {
    let packed = PsxSettings::disabled().pack(RenderResolution::DEFAULT);
    assert_eq!(packed, [[0.0; 4]; 3]);
}

#[test]
fn pack_scales_the_snap_grid_to_the_target() {
    let Ok(settings) = PsxSettings::try_new(NonZeroU32::new(2), None, 0.0) else {
        panic!("fixture settings are valid");
    };
    let packed = settings.pack(RenderResolution::DEFAULT);
    assert_eq!(packed[2], [640.0, 480.0, 1.0, 0.0]);
}

#[test]
fn pack_stores_fog_start_and_inverse_span() {
    let Ok(fog) = fog(10.0, 30.0) else {
        panic!("fixture fog is valid");
    };
    let Ok(settings) = PsxSettings::try_new(None, Some(fog), 0.5) else {
        panic!("fixture settings are valid");
    };
    let packed = settings.pack(RenderResolution::DEFAULT);
    assert_eq!(packed[0], [0.1, 0.2, 0.3, 0.0]);
    assert_eq!(packed[1], [10.0, 0.05, 1.0, 0.5]);
}

#[test]
fn the_default_look_has_every_effect_on() {
    let settings = PsxSettings::default();
    assert!(settings.snap_steps_per_pixel().is_some());
    assert!(settings.fog().is_some());
    assert!(settings.dither_strength() > 0.0);
}
