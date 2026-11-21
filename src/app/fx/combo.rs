//! The combo chip: in the board's top corner while clears keep coming. It pops up a step with
//! each one, grows and warms (cyan, gold, pink), and cracks and fades when a piece locks without
//! clearing.

use macroquad::prelude::*;

use super::{style, text_centered, FxPalette};
use crate::game::Game;

/// Seconds of the pop when the count goes up.
const POP: f32 = 0.25;
/// Seconds the crack takes.
const CRACK: f32 = 0.5;

#[derive(Default)]
pub(super) struct ComboFx {
    /// The count shown (the clears in a row before the last one, as the callouts count it).
    n: u32,
    pop_at: f64,
    crack_at: Option<f64>,
}

impl ComboFx {
    /// `clears_before`: the clears scored before this frame.
    pub fn update(&mut self, clears_before: u32, game: &Game, now: f64) {
        let scoring = &game.scoring_system;
        if scoring.clears() < clears_before {
            *self = Self::default();
            return;
        }
        if scoring.clears() > clears_before {
            if let Some(c) = scoring.last_clear() {
                if c.chain >= 1 {
                    self.n = c.chain;
                    self.pop_at = now;
                    self.crack_at = None;
                }
            }
        }
        if self.n > 0 && self.crack_at.is_none() && scoring.current_combo() == 0 {
            self.crack_at = Some(now);
        }
        if self.crack_at.is_some_and(|at| (now - at) as f32 > CRACK) {
            *self = Self::default();
        }
    }

    /// A combo of `n` that popped at `at` (desktop screenshots).
    #[cfg(not(target_os = "ios"))]
    pub fn demo(n: u32, at: f64) -> Self {
        Self { n, pop_at: at, crack_at: None }
    }

    pub fn draw(&self, board: Rect, cell: f32, now: f64, pal: &FxPalette) {
        if self.n == 0 {
            return;
        }
        let pop = 1.0 - ((now - self.pop_at) as f32 / POP).clamp(0.0, 1.0);
        let crack = self.crack_at.map_or(0.0, |at| ((now - at) as f32 / CRACK).clamp(0.0, 1.0));
        let color = heat(&pal.combo, self.n);
        let r = cell * (1.05 + 0.1 * self.n.min(6) as f32) * (1.0 + 0.25 * pop) * (1.0 - 0.2 * crack);
        let c = vec2(board.right() - r - cell * 0.35, board.y + r + cell * 0.35 + crack * crack * cell * 1.5);
        let a = 1.0 - crack;
        draw_circle(c.x, c.y, r, style::alpha(pal.dim, 0.88 * a));
        draw_circle_lines(c.x, c.y, r, (cell * 0.12).max(2.0), style::alpha(color, a));
        text_centered("COMBO", c.x, c.y - r * 0.42, (r * 0.3).max(8.0), style::alpha(color, a));
        text_centered(&self.n.to_string(), c.x, c.y + r * 0.12, r * (0.85 + 0.2 * pop), style::alpha(WHITE, a));
        if crack > 0.0 {
            // A jagged crack across it.
            let w = r * 1.6;
            let mut prev = vec2(c.x - w / 2.0, c.y - r * 0.1);
            for i in 1..=6 {
                let x = c.x - w / 2.0 + w * i as f32 / 6.0;
                let y = c.y - r * 0.1 + if i % 2 == 0 { -r * 0.18 } else { r * 0.18 };
                draw_line(prev.x, prev.y, x, y, (cell * 0.08).max(1.5), style::alpha(pal.outline, a));
                prev = vec2(x, y);
            }
        }
    }
}

/// The chip's colour for a combo of `n`: cyan for a small one, through gold, to pink.
pub fn heat(stops: &[Color; 3], n: u32) -> Color {
    let h = ((n.max(1) - 1) as f32 / 4.0).clamp(0.0, 1.0);
    if h < 0.5 {
        style::mix(stops[0], stops[1], h * 2.0)
    } else {
        style::mix(stops[1], stops[2], (h - 0.5) * 2.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heat_runs_from_the_first_colour_to_the_last() {
        let stops = [RED, GREEN, BLUE];
        assert_eq!(heat(&stops, 1), RED);
        assert_eq!(heat(&stops, 3), GREEN);
        assert_eq!(heat(&stops, 5), BLUE);
        assert_eq!(heat(&stops, 12), BLUE);
    }
}
