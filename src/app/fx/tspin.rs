//! A T-spin clear: the T glows, the filled corners around its centre that made it a T-spin light
//! up one after another (in time with the haptic's three ticks), and T-SPIN DOUBLE (or SINGLE,
//! TRIPLE) sweeps across the board.

use macroquad::prelude::*;

use super::{banner, cell_pos, style, FxPalette};
use crate::game::Game;
use crate::game::config::{BOARD_HEIGHT, BOARD_WIDTH, BUFFER_HEIGHT};

/// Seconds between one corner lighting and the next.
const CORNER_STEP: f32 = 0.08;
/// When the corners fade, and how long it takes.
const CORNERS_OUT: f32 = 0.5;
const FADE: f32 = 0.15;
/// The label's slide in, and when it leaves.
const LABEL_IN: f32 = 0.25;
const LABEL_OUT: f32 = 0.95;
pub const DONE: f32 = LABEL_OUT + 0.3;

pub(super) struct TspinFx {
    cells: Vec<(i32, i32)>,
    corners: Vec<(i32, i32)>,
    lines: usize,
    start: f64,
}

impl TspinFx {
    pub fn new(game: &Game, now: f64) -> Self {
        let cells = game.last_lock_cells.clone();
        let corners = centre(&cells).map(|c| filled_corners(game, c)).unwrap_or_default();
        Self { cells, corners, lines: game.clearing_lines.len(), start: now }
    }

    pub fn done(&self, now: f64) -> bool {
        (now - self.start) as f32 > DONE
    }

    pub fn draw(&self, board: Rect, cell: f32, now: f64, pal: &FxPalette) {
        let t = (now - self.start) as f32;
        // The T glows.
        let glow = (1.0 - t / 0.35).clamp(0.0, 1.0);
        if glow > 0.0 {
            for &(x, y) in &self.cells {
                if y < BUFFER_HEIGHT as i32 {
                    continue;
                }
                let p = cell_pos(board, cell, x as f32, y as f32);
                draw_rectangle_lines(p.x - 1.0, p.y - 1.0, cell + 2.0, cell + 2.0, (cell * 0.12).max(2.0), style::alpha(pal.tspin, glow));
            }
        }
        // The corners light up one by one.
        let fade = 1.0 - ((t - CORNERS_OUT) / FADE).clamp(0.0, 1.0);
        for (i, &(x, y)) in self.corners.iter().enumerate() {
            let on = ((t - i as f32 * CORNER_STEP) / 0.05).clamp(0.0, 1.0) * fade;
            if on <= 0.0 || y < BUFFER_HEIGHT as i32 || y >= (BOARD_HEIGHT + BUFFER_HEIGHT) as i32 || x < 0 || x >= BOARD_WIDTH as i32 {
                continue;
            }
            let p = cell_pos(board, cell, x as f32, y as f32);
            let inset = cell * 0.08;
            draw_rectangle_lines(p.x + inset, p.y + inset, cell - 2.0 * inset, cell - 2.0 * inset, (cell * 0.1).max(2.0), style::alpha(pal.stamp, on));
            draw_rectangle(p.x + inset, p.y + inset, cell - 2.0 * inset, cell - 2.0 * inset, style::alpha(pal.stamp, 0.25 * on));
        }
        let label = match self.lines {
            0 => "T-SPIN",
            1 => "T-SPIN SINGLE",
            2 => "T-SPIN DOUBLE",
            _ => "T-SPIN TRIPLE",
        };
        banner(label, board, board.y + board.h * 0.3, cell, t, LABEL_IN, LABEL_OUT, pal.tspin, WHITE);
    }
}

/// The T's centre: the one cell next to all three others.
pub fn centre(cells: &[(i32, i32)]) -> Option<(i32, i32)> {
    cells.iter().copied().find(|&(x, y)| {
        cells.iter().filter(|&&(ox, oy)| (ox - x).abs() + (oy - y).abs() == 1).count() == 3
    })
}

/// The corners diagonal to `c` that are filled (a wall or the floor counts as filled).
fn filled_corners(game: &Game, (cx, cy): (i32, i32)) -> Vec<(i32, i32)> {
    [(-1, -1), (1, -1), (-1, 1), (1, 1)]
        .into_iter()
        .map(|(dx, dy)| (cx + dx, cy + dy))
        .filter(|&(x, y)| {
            x < 0 || x >= BOARD_WIDTH as i32 || y >= (BOARD_HEIGHT + BUFFER_HEIGHT) as i32
                || game.board.get_cell(x, y).is_some_and(|c| c.color().is_some())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_centre_of_a_t_is_the_cell_touching_the_other_three() {
        // Flat, stem down: centre (4, 8).
        assert_eq!(centre(&[(3, 8), (4, 8), (5, 8), (4, 9)]), Some((4, 8)));
        assert_eq!(centre(&[(4, 7), (4, 8), (4, 9), (5, 8)]), Some((4, 8)));
        // An I has no such cell.
        assert_eq!(centre(&[(0, 5), (1, 5), (2, 5), (3, 5)]), None);
    }
}
