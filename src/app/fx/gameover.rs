//! Game over: the stack greys out row by row from the bottom, then the card rises with the score
//! counting up, and its buttons come on (PLAY AGAIN and MENU, or SAVE SCORE for a high score).
//! The buttons themselves are touch buttons (input::touch, Mode::GameOver), placed in the card
//! from [`crate::input::touch::game_over_card`].

use macroquad::prelude::*;

use super::{ease_out_cubic, style, text_centered, FxPalette};
use crate::game::Game;
use crate::game::config::VISIBLE_HEIGHT;
use crate::graphics::view::Layout;
use crate::input::touch::game_over_card;

/// Seconds between one row greying and the next (bottom up), and one row's fade.
const ROW_STEP: f32 = 0.03;
const ROW_FADE: f32 = 0.15;
/// The card rising, the score counting up, and the buttons coming on.
const CARD_IN: f32 = 0.75;
const CARD_RISE: f32 = 0.4;
const COUNT: f32 = 0.6;
pub const READY: f32 = CARD_IN + CARD_RISE + COUNT + 0.05;

pub(super) struct GameOverFx {
    start: f64,
    pub high_score: bool,
}

impl GameOverFx {
    pub fn new(now: f64, high_score: bool) -> Self {
        Self { start: now, high_score }
    }

    /// Whether the card is up and its buttons can be tapped.
    pub fn ready(&self, now: f64) -> bool {
        (now - self.start) as f32 >= READY
    }

    /// How grey the board's visible row `row` (0 = top) is by now, 0..1.
    pub fn grey(&self, row: usize, now: f64) -> f32 {
        let t = (now - self.start) as f32;
        let at = (VISIBLE_HEIGHT - 1 - row.min(VISIBLE_HEIGHT - 1)) as f32 * ROW_STEP;
        ((t - at) / ROW_FADE).clamp(0.0, 1.0)
    }

    pub fn draw(&self, game: &Game, layout: &Layout, now: f64, pal: &FxPalette) {
        let t = (now - self.start) as f32;
        let u = layout.unit;
        let card = game_over_card(layout);
        let rise = ((t - CARD_IN) / CARD_RISE).clamp(0.0, 1.0);
        if rise <= 0.0 {
            return;
        }
        // Dim the board behind it.
        let s = layout.screen;
        draw_rectangle(s.x, s.y, s.w, s.h, style::alpha(pal.dim, 0.45 * rise));
        let y = card.y + (s.bottom() - card.y) * (1.0 - back(rise));
        let r = Rect::new(card.x, y, card.w, card.h);
        (pal.panel)(r, pal.card_edge, u);
        style::round_outline(r, 10.0 * u, (2.0 * u).max(1.0), style::alpha(pal.card_edge, 0.9));

        // GAME OVER in the word's colours, letter by letter.
        let title = "GAME OVER";
        let size = (r.w / 7.0).min(40.0 * u);
        let n = title.chars().count();
        let step = size * 0.62;
        let x0 = r.center().x - step * (n - 1) as f32 / 2.0;
        for (i, ch) in title.chars().enumerate() {
            if ch == ' ' {
                continue;
            }
            let u_ = i as f32 / (n - 1) as f32;
            let color = if u_ < 0.5 { style::mix(pal.word[0], pal.word[1], u_ * 2.0) } else { style::mix(pal.word[1], pal.word[2], (u_ - 0.5) * 2.0) };
            let cx = x0 + i as f32 * step;
            let cy = r.y + r.h * 0.2;
            text_centered(&ch.to_string(), cx + 2.0 * u, cy + 2.0 * u, size, pal.outline);
            text_centered(&ch.to_string(), cx, cy, size, color);
        }

        let label = if self.high_score { "NEW HIGH SCORE!" } else { "SCORE" };
        text_centered(label, r.center().x, r.y + r.h * 0.4, (size * 0.42).max(10.0), pal.card_label);
        let count = ease_out_cubic(((t - CARD_IN - CARD_RISE) / COUNT).clamp(0.0, 1.0));
        let shown = (game.score as f32 * count).round() as u32;
        text_centered(&shown.to_string(), r.center().x, r.y + r.h * 0.56, size * 0.8, pal.card_ink);
    }
}

/// Ease out with a little overshoot, for the card arriving.
fn back(x: f32) -> f32 {
    let (c1, c3) = (1.70158, 2.70158);
    1.0 + c3 * (x - 1.0).powi(3) + c1 * (x - 1.0).powi(2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bottom_row_greys_first_and_all_are_grey_before_the_card() {
        let fx = GameOverFx::new(0.0, false);
        let early = 0.05;
        assert!(fx.grey(VISIBLE_HEIGHT - 1, early) > 0.0);
        assert_eq!(fx.grey(0, early), 0.0);
        for row in 0..VISIBLE_HEIGHT {
            assert_eq!(fx.grey(row, CARD_IN as f64), 1.0, "row {row}");
        }
    }

    #[test]
    fn buttons_wait_for_the_card() {
        let fx = GameOverFx::new(10.0, false);
        assert!(!fx.ready(10.0 + CARD_IN as f64));
        assert!(fx.ready(10.0 + READY as f64));
    }
}
