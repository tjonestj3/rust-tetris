//! The line clear: the cells pop outward from where the piece landed, shards fly, then the stack
//! above falls into place with a small bounce (over live play: the game waits only for the pop).

use macroquad::prelude::*;

use super::{cell_pos, ease_out_bounce, hash01, style, MIDDLE};
use crate::game::Game;
use crate::game::config::{BOARD_HEIGHT, BOARD_WIDTH, BUFFER_HEIGHT};

/// Seconds between one column popping and the next, outward from the piece.
pub const STAGGER: f32 = 0.017;
/// Seconds one cell takes to swell and vanish.
pub const POP: f32 = 0.16;
/// Seconds the stack takes to fall into place after the lines go.
pub const FALL: f32 = 0.2;
/// Seconds a shard lasts.
const SHARD_LIFE: f32 = 0.75;
const SHARDS_PER_CELL: u32 = 3;

pub(super) struct ClearFx {
    /// The rows that cleared (board rows).
    rows: Vec<usize>,
    /// The column the pop spreads out from: the middle of the piece that completed the lines.
    pub origin: f32,
    /// The cleared cells and their colours, for the shards.
    cells: Vec<(usize, usize, Color)>,
    start: f64,
    /// When the lines went and the stack started to fall.
    pub fall_start: Option<f64>,
    /// For each row after the clear, how many rows it fell.
    shifts: Vec<usize>,
}

impl ClearFx {
    pub fn new(game: &Game, now: f64) -> Self {
        let lock = &game.last_lock_cells;
        let origin = if lock.is_empty() {
            MIDDLE
        } else {
            lock.iter().map(|&(x, _)| x as f32).sum::<f32>() / lock.len() as f32
        };
        let rows = game.clearing_lines.clone();
        let mut cells = Vec::new();
        for &y in &rows {
            for x in 0..BOARD_WIDTH {
                if let Some(color) = game.board.get_cell(x as i32, y as i32).and_then(|c| c.color()) {
                    cells.push((x, y, color));
                }
            }
        }
        let shifts = shifts(&rows, BOARD_HEIGHT + BUFFER_HEIGHT);
        Self { rows, origin, cells, start: now, fall_start: None, shifts }
    }

    /// How far up (in cells) the row now at board row `y` is drawn: it starts where it was before
    /// the clear and drops into place.
    pub fn fall_dy(&self, y: usize, now: f64) -> f32 {
        let Some(start) = self.fall_start else { return 0.0 };
        let p = ((now - start) as f32 / FALL).clamp(0.0, 1.0);
        let shift = self.shifts.get(y).copied().unwrap_or(0) as f32;
        -shift * (1.0 - ease_out_bounce(p))
    }

    pub fn draw_shards(&self, board: Rect, cell: f32, now: f64) {
        let _ = &self.rows;
        for (i, &(x, y, color)) in self.cells.iter().enumerate() {
            let born = self.start as f32 + delay(x as f32, self.origin) + POP * 0.45;
            let age = now as f32 - born;
            if !(0.0..SHARD_LIFE).contains(&age) {
                continue;
            }
            let fade = 1.0 - age / SHARD_LIFE;
            let centre = cell_pos(board, cell, x as f32 + 0.5, y as f32 + 0.5);
            for k in 0..SHARDS_PER_CELL {
                let r1 = hash01(i as u32, k * 2);
                let r2 = hash01(i as u32, k * 2 + 1);
                // In cells per second: out sideways and up, then gravity.
                let v = vec2((r1 - 0.5) * 7.0, -(2.4 + r2 * 5.0));
                let p = centre + (v * age + vec2(0.0, 13.0 * age * age * 0.5)) * cell;
                let size = cell * 0.24 * fade;
                draw_rectangle(p.x - size / 2.0, p.y - size / 2.0, size, size, style::alpha(style::mix(color, WHITE, 0.3), fade));
            }
        }
    }
}

fn delay(x: f32, origin: f32) -> f32 {
    (x - origin).abs().round() * STAGGER
}

/// The pop of the cell in column `x`, `t` seconds into the clear: (scale, whiteness), or `None`
/// once it's gone.
pub fn pop(t: f32, x: f32, origin: f32) -> Option<(f32, f32)> {
    let local = (t - delay(x, origin)) / POP;
    if local <= 0.0 {
        Some((1.0, 0.0))
    } else if local < 0.45 {
        let k = local / 0.45;
        Some((1.0 + 0.18 * k, 0.8 * k))
    } else if local < 1.0 {
        Some((1.18 * (1.0 - (local - 0.45) / 0.55), 0.8))
    } else {
        None
    }
}

/// For each board row after `cleared` rows go, how many rows down it moved.
pub fn shifts(cleared: &[usize], height: usize) -> Vec<usize> {
    let mut out = vec![cleared.len(); height];
    let mut new = height;
    for old in (0..height).rev() {
        if cleared.contains(&old) {
            continue;
        }
        new -= 1;
        out[new] = new - old;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::config::LINE_CLEAR_ANIMATION_TIME;

    #[test]
    fn rows_above_a_clear_fall_by_the_rows_below_them() {
        // 6 rows; rows 4 and 5 (the bottom two) clear.
        assert_eq!(shifts(&[4, 5], 6), vec![2, 2, 2, 2, 2, 2]);
        // Rows 2 and 4 clear: row 5 stays, the old row 3 drops one to 4, rows 0-1 drop two.
        assert_eq!(shifts(&[2, 4], 6), vec![2, 2, 2, 2, 1, 0]);
        assert_eq!(shifts(&[], 4), vec![0, 0, 0, 0]);
    }

    #[test]
    fn every_cell_has_popped_by_the_time_play_resumes() {
        let t = LINE_CLEAR_ANIMATION_TIME as f32;
        for origin in [0.0, 4.5, 9.0] {
            for x in 0..BOARD_WIDTH {
                assert_eq!(pop(t, x as f32, origin), None, "column {x} from {origin}");
            }
        }
    }

    #[test]
    fn the_pop_spreads_out_from_the_piece() {
        // Early on, the piece's column has started and the far edge hasn't.
        assert!(pop(0.03, 4.0, 4.0).unwrap().0 > 1.0);
        assert_eq!(pop(0.03, 9.0, 4.0), Some((1.0, 0.0)));
    }
}
