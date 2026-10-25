use std::num::NonZeroU32;

use proptest::prelude::*;
use render::psx::{Fog, PsxSettings};
use render::resolution::RenderResolution;

proptest! {
    #[test]
    fn accepted_fog_always_has_a_positive_finite_span(
        start in -10.0f32..200.0,
        end in -10.0f32..200.0,
    ) {
        if let Ok(fog) = Fog::try_new(start, end, [0.0; 3]) {
            prop_assert!(fog.start() >= 0.0);
            prop_assert!(fog.end() > fog.start());
            prop_assert!((fog.end() - fog.start()).is_finite());
        }
    }

    #[test]
    fn packed_settings_are_always_finite(
        steps in proptest::option::of(1u32..16),
        fogged in proptest::option::of((0.0f32..100.0, 0.001f32..100.0, 0.0f32..=1.0)),
        dither in 0.0f32..=1.0,
        width in 1u32..2000,
        height in 1u32..2000,
    ) {
        let fog = fogged.and_then(|(start, span, c)| Fog::try_new(start, start + span, [c; 3]).ok());
        let Ok(settings) = PsxSettings::try_new(steps.and_then(NonZeroU32::new), fog, dither) else {
            return Err(TestCaseError::fail("in-range dither strength was rejected"));
        };
        let Ok(resolution) = RenderResolution::try_new(width, height) else {
            return Err(TestCaseError::fail("non-zero resolution was rejected"));
        };
        for word in settings.pack(resolution).iter().flatten() {
            prop_assert!(word.is_finite());
        }
    }

    #[test]
    fn rejected_dither_strength_is_exactly_the_out_of_range_values(strength in -2.0f32..3.0) {
        let accepted = PsxSettings::try_new(None, None, strength).is_ok();
        prop_assert_eq!(accepted, (0.0..=1.0).contains(&strength));
    }
}
