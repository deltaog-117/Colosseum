//! The heads-up display: turns the match state into the quads the renderer
//! draws over the scene.

use render::hud::{HudBatch, text_width};
use render::resolution::RenderResolution;
use sim::combat::tuning::MAX_HEALTH;
use sim::rules::Phase;

const MARGIN: f32 = 8.0;
const SMALL: u32 = 1;
const MEDIUM: u32 = 2;
const LARGE: u32 = 3;
/// Height of a glyph at scale 1.
const GLYPH_HEIGHT: f32 = 7.0;

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
const GREY: [f32; 4] = [0.65, 0.65, 0.7, 1.0];
const GREEN: [f32; 4] = [0.3, 0.95, 0.4, 1.0];
const YELLOW: [f32; 4] = [0.98, 0.85, 0.25, 1.0];
const RED: [f32; 4] = [1.0, 0.25, 0.2, 1.0];
const PANEL: [f32; 4] = [0.02, 0.02, 0.05, 0.75];

/// The scoreboard has four columns, so a count is clamped to what fits.
const COUNT_LIMIT: u32 = 9999;
const FRAGS_LIMIT: i64 = 9999;

/// Crosshair arm length, thickness and the gap around the center.
const ARM: f32 = 4.0;
const GAP: f32 = 2.0;

/// Most bot rows the scoreboard shows; the panel must fit the screen.
const MAX_BOT_ROWS: usize = 8;
/// Height of one scoreboard row.
const ROW_HEIGHT: f32 = 12.0;

/// One bot's line on the scoreboard.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ScoreRow {
    pub(crate) kills: u32,
    pub(crate) deaths: u32,
    pub(crate) frags: i64,
}

/// Everything the HUD shows, copied out of the match state.
#[derive(Debug, Clone, Copy)]
pub(crate) struct HudView<'a> {
    /// Zero while dead.
    pub(crate) health: u16,
    pub(crate) ammo: u16,
    pub(crate) seconds_left: u32,
    pub(crate) phase: Phase,
    pub(crate) kills: u32,
    pub(crate) deaths: u32,
    pub(crate) frags: i64,
    /// The scoreboard key is held.
    pub(crate) scoreboard: bool,
    /// The bots' scores, in order, shown under the player's.
    pub(crate) bots: &'a [ScoreRow],
}

/// Rebuilds `batch` for one frame.
pub(crate) fn draw(batch: &mut HudBatch, view: &HudView<'_>) {
    batch.clear();
    let screen = RenderResolution::DEFAULT;
    let (width, height) = (screen.width() as f32, screen.height() as f32);
    let playing = view.phase == Phase::Playing;
    let alive = view.health > 0;

    if alive && playing {
        crosshair(batch, width, height);
    }
    vitals(batch, view, width, height);
    clock(batch, view, width);
    batch.text(
        MARGIN,
        MARGIN,
        SMALL,
        GREY,
        &format!("FRAGS {}", view.frags),
    );

    if !alive && playing {
        centered(batch, width, height * 0.35, LARGE, RED, "YOU DIED");
    }
    if !playing {
        centered(batch, width, 40.0, LARGE, YELLOW, "ROUND OVER");
        let next = format!("NEXT ROUND IN {}", view.seconds_left);
        centered(batch, width, 70.0, SMALL, WHITE, &next);
    }
    // The final score is always up between rounds.
    if view.scoreboard || !playing {
        scoreboard(batch, view, width, height);
    }
}

fn centered(batch: &mut HudBatch, width: f32, y: f32, scale: u32, color: [f32; 4], text: &str) {
    let x = (width - text_width(text, scale) as f32) / 2.0;
    batch.text(x, y, scale, color, text);
}

fn crosshair(batch: &mut HudBatch, width: f32, height: f32) {
    let (cx, cy) = ((width / 2.0).floor(), (height / 2.0).floor());
    batch.rect(cx - GAP - ARM, cy, ARM, 1.0, WHITE);
    batch.rect(cx + GAP + 1.0, cy, ARM, 1.0, WHITE);
    batch.rect(cx, cy - GAP - ARM, 1.0, ARM, WHITE);
    batch.rect(cx, cy + GAP + 1.0, 1.0, ARM, WHITE);
}

/// Health at the bottom left, rockets at the bottom right.
fn vitals(batch: &mut HudBatch, view: &HudView<'_>, width: f32, height: f32) {
    let number_height = GLYPH_HEIGHT * MEDIUM as f32;
    let number_y = height - MARGIN - number_height;
    let label_y = number_y - GLYPH_HEIGHT - 3.0;

    let health_color = match view.health {
        0..=25 => RED,
        26..=50 => YELLOW,
        _ => GREEN,
    };
    batch.text(MARGIN, label_y, SMALL, GREY, "HEALTH");
    batch.text(
        MARGIN,
        number_y,
        MEDIUM,
        health_color,
        &view.health.to_string(),
    );
    // A thin gauge under the number: full at `MAX_HEALTH`.
    let gauge = 48.0;
    let filled = gauge * f32::from(view.health.min(MAX_HEALTH)) / f32::from(MAX_HEALTH);
    let gauge_y = number_y + number_height + 2.0;
    batch.rect(MARGIN, gauge_y, gauge, 2.0, PANEL);
    batch.rect(MARGIN, gauge_y, filled, 2.0, health_color);

    let ammo = view.ammo.to_string();
    let ammo_color = if view.ammo == 0 { RED } else { YELLOW };
    let right = width - MARGIN;
    let label = "ROCKETS";
    batch.text(
        right - text_width(label, SMALL) as f32,
        label_y,
        SMALL,
        GREY,
        label,
    );
    batch.text(
        right - text_width(&ammo, MEDIUM) as f32,
        number_y,
        MEDIUM,
        ammo_color,
        &ammo,
    );
}

/// Round time as `M:SS` at the top center.
fn clock(batch: &mut HudBatch, view: &HudView<'_>, width: f32) {
    let text = format!("{}:{:02}", view.seconds_left / 60, view.seconds_left % 60);
    let color = if view.phase == Phase::Playing && view.seconds_left <= 10 {
        RED
    } else {
        WHITE
    };
    centered(batch, width, MARGIN, MEDIUM, color, &text);
}

fn scoreboard(batch: &mut HudBatch, view: &HudView<'_>, width: f32, height: f32) {
    let bots = &view.bots[..view.bots.len().min(MAX_BOT_ROWS)];
    let panel_w = 200.0;
    let panel_h = 44.0 + ROW_HEIGHT * (1 + bots.len()) as f32;
    let left = ((width - panel_w) / 2.0).floor();
    let top = ((height - panel_h) / 2.0).floor();
    batch.rect(left, top, panel_w, panel_h, PANEL);

    // Column starts, measured from the panel's left edge.
    let columns = [10.0, 60.0, 104.0, 152.0];
    for (offset, head) in columns.iter().zip(["NAME", "KILLS", "DEATHS", "FRAGS"]) {
        batch.text(left + offset, top + 10.0, SMALL, GREY, head);
    }

    let you = ScoreRow {
        kills: view.kills,
        deaths: view.deaths,
        frags: view.frags,
    };
    let rows = std::iter::once(("YOU".to_string(), you)).chain(
        bots.iter()
            .enumerate()
            .map(|(index, bot)| (format!("BOT {}", index + 1), *bot)),
    );
    for (line, (name, row)) in rows.enumerate() {
        let y = top + 30.0 + ROW_HEIGHT * line as f32;
        let kills = row.kills.min(COUNT_LIMIT).to_string();
        let deaths = row.deaths.min(COUNT_LIMIT).to_string();
        let frags = row.frags.clamp(-FRAGS_LIMIT, FRAGS_LIMIT).to_string();
        let cells = [
            name.as_str(),
            kills.as_str(),
            deaths.as_str(),
            frags.as_str(),
        ];
        for (offset, cell) in columns.iter().zip(cells) {
            batch.text(left + offset, y, SMALL, WHITE, cell);
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn view() -> HudView<'static> {
        HudView {
            health: 100,
            ammo: 10,
            seconds_left: 125,
            phase: Phase::Playing,
            kills: 3,
            deaths: 1,
            frags: 2,
            scoreboard: false,
            bots: &[],
        }
    }

    fn quads(view: &HudView<'_>) -> usize {
        let mut batch = HudBatch::new();
        draw(&mut batch, view);
        batch.quad_count()
    }

    /// The crosshair's left arm starts at this corner and no other element does.
    fn has_crosshair(view: &HudView<'_>) -> bool {
        let mut batch = HudBatch::new();
        draw(&mut batch, view);
        batch
            .vertices()
            .iter()
            .any(|v| v.position == [154.0, 120.0])
    }

    #[test]
    fn a_live_player_sees_a_crosshair() {
        assert!(has_crosshair(&view()));
    }

    #[test]
    fn the_dead_have_no_crosshair() {
        let dead = HudView {
            health: 0,
            ..view()
        };
        assert!(!has_crosshair(&dead));
    }

    #[test]
    fn no_crosshair_between_rounds() {
        let over = HudView {
            phase: Phase::Intermission,
            ..view()
        };
        assert!(!has_crosshair(&over));
    }

    #[test]
    fn the_scoreboard_adds_quads_and_shows_between_rounds() {
        let held = HudView {
            scoreboard: true,
            ..view()
        };
        assert!(quads(&held) > quads(&view()));

        let over = HudView {
            phase: Phase::Intermission,
            ..view()
        };
        let over_held = HudView {
            scoreboard: true,
            ..over
        };
        assert_eq!(quads(&over), quads(&over_held));
    }

    #[test]
    fn every_bot_adds_a_row_to_the_scoreboard() {
        let row = ScoreRow {
            kills: 2,
            deaths: 3,
            frags: -1,
        };
        let alone = HudView {
            scoreboard: true,
            ..view()
        };
        let one = HudView {
            bots: &[row],
            ..alone
        };
        let two = HudView {
            bots: &[row, row],
            ..alone
        };
        assert!(quads(&one) > quads(&alone));
        assert!(quads(&two) > quads(&one));
    }

    #[test]
    fn rows_past_the_limit_are_not_drawn() {
        let row = ScoreRow {
            kills: 0,
            deaths: 0,
            frags: 0,
        };
        let rows = [row; MAX_BOT_ROWS + 5];
        let full = HudView {
            scoreboard: true,
            bots: &rows[..MAX_BOT_ROWS],
            ..view()
        };
        let over = HudView {
            bots: &rows,
            ..full
        };
        assert_eq!(quads(&full), quads(&over));
    }

    #[test]
    fn drawing_replaces_the_previous_frame() {
        let mut batch = HudBatch::new();
        draw(&mut batch, &view());
        let first = batch.quad_count();
        draw(&mut batch, &view());
        assert_eq!(batch.quad_count(), first);
    }

    proptest! {
        #[test]
        fn every_vertex_stays_on_the_screen(
            health in 0u16..=u16::MAX,
            ammo in 0u16..=u16::MAX,
            seconds_left in 0u32..=u32::MAX,
            kills in 0u32..=u32::MAX,
            deaths in 0u32..=u32::MAX,
            frags in i64::MIN..=i64::MAX,
            intermission in any::<bool>(),
            scoreboard in any::<bool>(),
            bots in prop::collection::vec((any::<u32>(), any::<u32>(), any::<i64>()), 0..20),
        ) {
            let rows: Vec<ScoreRow> = bots
                .iter()
                .map(|&(kills, deaths, frags)| ScoreRow { kills, deaths, frags })
                .collect();
            let view = HudView {
                health,
                ammo,
                seconds_left,
                phase: if intermission { Phase::Intermission } else { Phase::Playing },
                kills,
                deaths,
                frags,
                scoreboard,
                bots: &rows,
            };
            let mut batch = HudBatch::new();
            draw(&mut batch, &view);

            let screen = RenderResolution::DEFAULT;
            let (w, h) = (screen.width() as f32, screen.height() as f32);
            for vertex in batch.vertices() {
                let [x, y] = vertex.position;
                // The health gauge sits just under the bottom margin.
                prop_assert!((0.0..=w).contains(&x), "x = {x}");
                prop_assert!((0.0..=h).contains(&y), "y = {y}");
            }
        }
    }
}
