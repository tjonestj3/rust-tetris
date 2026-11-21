//! Level up: the level number pops up in gold with a ring around it (drawn by the stats panel,
//! from [`LevelFx::pop`]), the board's grid pulses once, and speed lines run down its sides to say
//! the pieces fall faster now.

use macroquad::prelude::*;

use super::{hash01, style, FxPalette};

/// Seconds the number's pop lasts; the speed lines; the grid's pulse.
const POP: f32 = 0.6;
const LINES: f32 = 0.9;
const PULSE: f32 = 0.5;

pub(super) struct LevelFx {
    pub start: f64,
}

impl LevelFx {
    pub fn done(&self, now: f64) -> bool {
        (now - self.start) as f32 > LINES
    }

    /// 1 when the level has just gone up, falling to 0.
    pub fn pop(&self, now: f64) -> f32 {
        (1.0 - (now - self.start) as f32 / POP).clamp(0.0, 1.0)
    }

    pub fn draw(&self, board: Rect, cell: f32, now: f64, pal: &FxPalette) {
        let t = (now - self.start) as f32;
        // The grid pulses once.
        let pulse = (1.0 - (t - PULSE * 0.3).abs() / (PULSE * 0.7)).clamp(0.0, 1.0);
        if pulse > 0.0 {
            let color = style::alpha(pal.word[0], 0.22 * pulse);
            let cols = (board.w / cell).round() as i32;
            let rows = (board.h / cell).round() as i32;
            for c in 1..cols {
                draw_rectangle(board.x + c as f32 * cell, board.y, 1.0, board.h, color);
            }
            for r in 1..rows {
                draw_rectangle(board.x, board.y + r as f32 * cell, board.w, 1.0, color);
            }
        }
        // Speed lines down both sides.
        if t < LINES {
            let fade = 1.0 - (t / LINES).powi(2);
            for i in 0..12u32 {
                let side = if i % 2 == 0 { board.x - cell * 0.45 } else { board.right() + cell * 0.3 };
                let y = board.y + ((hash01(i, 31) + t * 2.6).fract()) * board.h;
                let h = cell * (1.0 + hash01(i, 32));
                draw_rectangle(side, y, (cell * 0.08).max(1.5), h, style::alpha(pal.word[0], 0.7 * fade));
            }
        }
    }
}

/// The level ring around the number at `c` (drawn by the stats panel).
pub fn ring(c: Vec2, size: f32, pop: f32, pal: &FxPalette) {
    if pop <= 0.0 {
        return;
    }
    let p = 1.0 - pop;
    let r = size * (0.6 + 1.6 * p);
    draw_circle_lines(c.x, c.y, r, (size * 0.12) * pop + 1.0, style::alpha(pal.stamp, pop));
}
