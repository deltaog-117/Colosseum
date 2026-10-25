use render::hud::font::{self, ATLAS_HEIGHT, ATLAS_WIDTH, CELL, COLUMNS, SOLID_CELL};
use render::hud::{HudBatch, MAX_QUADS, text_width};

const WHITE: [f32; 4] = [1.0; 4];

#[test]
fn a_rect_is_one_quad_covering_its_area() {
    let mut batch = HudBatch::new();
    batch.rect(10.0, 20.0, 30.0, 40.0, WHITE);

    assert_eq!(batch.quad_count(), 1);
    let corners: Vec<[f32; 2]> = batch.vertices().iter().map(|v| v.position).collect();
    assert_eq!(
        corners,
        [[10.0, 20.0], [40.0, 20.0], [10.0, 60.0], [40.0, 60.0]]
    );
}

#[test]
fn a_rect_samples_inside_the_solid_cell() {
    let mut batch = HudBatch::new();
    batch.rect(0.0, 0.0, 1.0, 1.0, WHITE);

    let [u0, v0, u1, v1] = font::cell_uv(SOLID_CELL);
    for vertex in batch.vertices() {
        assert!(vertex.uv[0] > u0 && vertex.uv[0] < u1);
        assert!(vertex.uv[1] > v0 && vertex.uv[1] < v1);
    }
}

#[test]
fn degenerate_and_non_finite_rects_are_ignored() {
    let mut batch = HudBatch::new();
    batch.rect(0.0, 0.0, 0.0, 5.0, WHITE);
    batch.rect(0.0, 0.0, 5.0, -1.0, WHITE);
    batch.rect(f32::NAN, 0.0, 5.0, 5.0, WHITE);
    batch.rect(0.0, 0.0, f32::INFINITY, 5.0, WHITE);
    batch.rect(0.0, 0.0, 5.0, 5.0, [f32::NAN, 0.0, 0.0, 1.0]);
    assert_eq!(batch.quad_count(), 0);
}

#[test]
fn spaces_advance_the_pen_without_drawing() {
    let mut batch = HudBatch::new();
    batch.text(0.0, 0.0, 1, WHITE, "A B");

    assert_eq!(batch.quad_count(), 2);
    let lefts: Vec<f32> = batch
        .vertices()
        .chunks(4)
        .map(|quad| quad[0].position[0])
        .collect();
    assert_eq!(lefts, [0.0, 12.0]);
}

#[test]
fn scale_multiplies_glyph_size_and_advance() {
    let mut batch = HudBatch::new();
    batch.text(0.0, 0.0, 3, WHITE, "AB");

    let quads: Vec<&[render::hud::HudVertex]> = batch.vertices().chunks(4).collect();
    assert_eq!(quads[0][3].position, [24.0, 24.0]);
    assert_eq!(quads[1][0].position, [18.0, 0.0]);
}

#[test]
fn text_lands_on_whole_pixels() {
    let mut batch = HudBatch::new();
    batch.text(10.4, 20.6, 1, WHITE, "A");
    assert_eq!(batch.vertices()[0].position, [10.0, 21.0]);
}

#[test]
fn scale_zero_draws_nothing() {
    let mut batch = HudBatch::new();
    batch.text(0.0, 0.0, 0, WHITE, "HELLO");
    assert_eq!(batch.quad_count(), 0);
}

#[test]
fn the_batch_stops_at_its_quad_limit() {
    let mut batch = HudBatch::new();
    for _ in 0..MAX_QUADS + 10 {
        batch.rect(0.0, 0.0, 1.0, 1.0, WHITE);
    }
    assert_eq!(batch.quad_count(), MAX_QUADS);
}

#[test]
fn clear_empties_the_batch() {
    let mut batch = HudBatch::new();
    batch.text(0.0, 0.0, 1, WHITE, "HP");
    batch.clear();
    assert_eq!(batch.quad_count(), 0);
    assert!(batch.vertices().is_empty());
}

#[test]
fn text_width_leaves_out_the_trailing_gap() {
    assert_eq!(text_width("", 2), 0);
    assert_eq!(text_width("A", 1), 5);
    assert_eq!(text_width("AB", 1), 11);
    assert_eq!(text_width("AB", 2), 22);
}

#[test]
fn lowercase_shares_the_uppercase_glyph() {
    assert_eq!(font::cell_for('a'), font::cell_for('A'));
}

#[test]
fn an_unknown_character_draws_as_a_question_mark() {
    assert_eq!(font::cell_for('\u{1F680}'), font::cell_for('?'));
    assert_ne!(font::cell_for('?'), font::cell_for('A'));
}

#[test]
fn the_atlas_has_its_declared_size() {
    assert_eq!(font::pixels().len(), (ATLAS_WIDTH * ATLAS_HEIGHT) as usize);
}

#[test]
fn the_solid_cell_is_fully_set_and_space_is_empty() {
    let atlas = font::pixels();
    let cell_sum = |cell: u32| -> u32 {
        let (x0, y0) = ((cell % COLUMNS) * CELL, (cell / COLUMNS) * CELL);
        (0..CELL)
            .flat_map(|dy| (0..CELL).map(move |dx| (x0 + dx, y0 + dy)))
            .map(|(x, y)| u32::from(atlas[(y * ATLAS_WIDTH + x) as usize]))
            .sum()
    };
    assert_eq!(cell_sum(SOLID_CELL), 255 * CELL * CELL);
    assert_eq!(cell_sum(font::cell_for(' ')), 0);
}

#[test]
fn every_digit_and_letter_has_ink() {
    let atlas = font::pixels();
    for ch in ('0'..='9').chain('A'..='Z') {
        let cell = font::cell_for(ch);
        let (x0, y0) = ((cell % COLUMNS) * CELL, (cell / COLUMNS) * CELL);
        let ink = (0..CELL)
            .flat_map(|dy| (0..CELL).map(move |dx| (x0 + dx, y0 + dy)))
            .any(|(x, y)| atlas[(y * ATLAS_WIDTH + x) as usize] != 0);
        assert!(ink, "{ch} has no pixels");
    }
}

#[test]
fn glyphs_have_distinct_shapes() {
    let atlas = font::pixels();
    let shape = |ch: char| -> Vec<u8> {
        let cell = font::cell_for(ch);
        let (x0, y0) = ((cell % COLUMNS) * CELL, (cell / COLUMNS) * CELL);
        (0..CELL)
            .flat_map(|dy| (0..CELL).map(move |dx| (x0 + dx, y0 + dy)))
            .map(|(x, y)| atlas[(y * ATLAS_WIDTH + x) as usize])
            .collect()
    };
    let glyphs: Vec<char> = ('0'..='9').chain('A'..='Z').collect();
    for (i, a) in glyphs.iter().enumerate() {
        for b in &glyphs[i + 1..] {
            assert_ne!(shape(*a), shape(*b), "{a} and {b} look the same");
        }
    }
}
