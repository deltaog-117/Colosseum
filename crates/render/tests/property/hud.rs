use proptest::prelude::*;
use render::hud::font::{self, ATLAS_HEIGHT, ATLAS_WIDTH, CELL, COLUMNS};
use render::hud::{HudBatch, MAX_QUADS, text_width};

const WHITE: [f32; 4] = [1.0; 4];

proptest! {
    #[test]
    fn any_character_maps_to_a_cell_inside_the_atlas(ch in any::<char>()) {
        let cell = font::cell_for(ch);
        prop_assert!((cell % COLUMNS + 1) * CELL <= ATLAS_WIDTH);
        prop_assert!((cell / COLUMNS + 1) * CELL <= ATLAS_HEIGHT);
    }

    #[test]
    fn any_text_keeps_the_batch_whole_and_finite(
        x in -1.0e6f32..1.0e6,
        y in -1.0e6f32..1.0e6,
        scale in 0u32..8,
        text in ".{0,80}",
    ) {
        let mut batch = HudBatch::new();
        batch.text(x, y, scale, WHITE, &text);

        prop_assert_eq!(batch.vertices().len() % 4, 0);
        prop_assert!(batch.quad_count() <= MAX_QUADS);
        for vertex in batch.vertices() {
            prop_assert!(vertex.position.iter().all(|v| v.is_finite()));
            prop_assert!(vertex.uv.iter().all(|v| (0.0..=1.0).contains(v)));
        }
    }

    #[test]
    fn a_batch_never_outgrows_its_limit(count in 0usize..MAX_QUADS * 2) {
        let mut batch = HudBatch::new();
        for _ in 0..count {
            batch.rect(0.0, 0.0, 2.0, 2.0, WHITE);
        }
        prop_assert_eq!(batch.quad_count(), count.min(MAX_QUADS));
    }

    #[test]
    fn only_visible_characters_make_quads(text in "[A-Z0-9 ]{0,60}", scale in 1u32..6) {
        let mut batch = HudBatch::new();
        batch.text(0.0, 0.0, scale, WHITE, &text);
        prop_assert_eq!(batch.quad_count(), text.chars().filter(|c| *c != ' ').count());
    }

    #[test]
    fn longer_text_is_never_narrower(text in ".{0,40}", extra in ".{1,10}", scale in 1u32..8) {
        let longer = format!("{text}{extra}");
        prop_assert!(text_width(&longer, scale) > text_width(&text, scale));
    }

    #[test]
    fn text_width_scales_linearly(text in ".{0,40}", scale in 1u32..8) {
        prop_assert_eq!(text_width(&text, scale), text_width(&text, 1) * scale);
    }

    #[test]
    fn the_last_glyph_ends_at_the_measured_width(text in "[A-Z0-9]{1,30}", scale in 1u32..6) {
        let mut batch = HudBatch::new();
        batch.text(0.0, 0.0, scale, WHITE, &text);

        // The quad is a full cell wide, but the glyph's five columns end
        // where `text_width` says.
        let Some(last) = batch.vertices().chunks(4).next_back() else {
            return Err(TestCaseError::fail("a non-empty text has quads"));
        };
        let ink_end = last[0].position[0] + (5 * scale) as f32;
        prop_assert_eq!(ink_end, text_width(&text, scale) as f32);
    }
}
