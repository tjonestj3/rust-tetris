//! The hard drop: a fading streak down the columns the piece fell through, a dip of the board
//! that springs back, and dust where it touched down.

use macroquad::prelude::*;

use super::{cell_pos, ease_out_cubic, style};
use crate::game::state::HardDropMark;

/// Seconds the streak takes to fade.
const STREAK: f32 = 0.3;
/// Seconds the dip takes to settle.
const DIP: f32 = 0.4;
/// Seconds the dust lasts.
const DUST: f32 = 0.45;

pub(super) struct DropFx {
    /// The piece's cells where it started, and how far it fell.
    start_cells: Vec<(i32, i32)>,
    distance: i32,
    color: Color,
    pub start: f64,
}

impl DropFx {
    pub fn new(mark: &HardDropMark, now: f64) -> Self {
        Self { start_cells: mark.start.clone(), distance: mark.distance, color: mark.piece_type.color(), start: now }
    }

    pub fn draw_streak(&self, board: Rect, cell: f32, now: f64) {
        let age = (now - self.start) as f32;
        if self.distance <= 0 || !(0.0..STREAK).contains(&age) {
            return;
        }
        let fade = 1.0 - age / STREAK;
        for x in columns(&self.start_cells) {
            let top = self.start_cells.iter().filter(|c| c.0 == x).map(|c| c.1).min().unwrap_or(0);
            let bottom = self.start_cells.iter().filter(|c| c.0 == x).map(|c| c.1).max().unwrap_or(0) + self.distance;
            let from = cell_pos(board, cell, x as f32 + 0.2, top as f32).y.max(board.y);
            let to = cell_pos(board, cell, x as f32, bottom as f32 + 0.5).y;
            if to <= from {
                continue;
            }
            // Brighter towards the bottom, in slices (macroquad has no gradient fill).
            let slices = 8;
            let h = (to - from) / slices as f32;
            for i in 0..slices {
                let a = 0.55 * fade * (i as f32 + 1.0) / slices as f32;
                draw_rectangle(board.x + (x as f32 + 0.2) * cell, from + i as f32 * h, cell * 0.6, h + 0.5, style::alpha(self.color, a));
            }
        }
    }

    pub fn draw_dust(&self, board: Rect, cell: f32, now: f64, color: Color) {
        let age = (now - self.start) as f32;
        if self.distance <= 0 || !(0.0..DUST).contains(&age) {
            return;
        }
        let p = ease_out_cubic(age / DUST);
        let landed: Vec<(i32, i32)> = self.start_cells.iter().map(|&(x, y)| (x, y + self.distance)).collect();
        let low = landed.iter().map(|c| c.1).max().unwrap_or(0);
        let left = landed.iter().filter(|c| c.1 == low).map(|c| c.0).min().unwrap_or(0);
        let right = landed.iter().filter(|c| c.1 == low).map(|c| c.0).max().unwrap_or(0) + 1;
        let floor = cell_pos(board, cell, 0.0, low as f32 + 1.0).y;
        for (edge, dir) in [(left, -1.0), (right, 1.0)] {
            let x = board.x + edge as f32 * cell;
            for k in 0..3 {
                let k = k as f32;
                let r = cell * (0.16 - k * 0.04) * (1.0 - p) + 1.0;
                draw_circle(x + dir * (cell * 0.2 + p * cell * 0.9 + k * cell * 0.25), floor - k * cell * 0.1 - p * cell * 0.2, r, style::alpha(color, 0.55 * (1.0 - p)));
            }
        }
    }
}

/// The board's dip `age` seconds after the drop, as a fraction of its depth (down positive).
pub fn dip(age: f32) -> f32 {
    if !(0.0..DIP).contains(&age) {
        return 0.0;
    }
    (-age * 9.0).exp() * (age * 28.0).cos()
}

fn columns(cells: &[(i32, i32)]) -> Vec<i32> {
    let mut xs: Vec<i32> = cells.iter().map(|c| c.0).collect();
    xs.sort_unstable();
    xs.dedup();
    xs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dip_goes_down_first_and_settles() {
        assert!(dip(0.0) > 0.9);
        assert!(dip(DIP - 0.01).abs() < 0.05);
        assert_eq!(dip(DIP + 0.1), 0.0);
        assert_eq!(dip(-0.1), 0.0);
    }

    #[test]
    fn one_streak_per_column() {
        assert_eq!(columns(&[(3, 0), (4, 0), (5, 0), (4, 1)]), vec![3, 4, 5]);
    }
}
