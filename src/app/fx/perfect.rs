//! A perfect clear: a gold light runs around the board's edge, the empty board glows, and
//! PERFECT CLEAR is spelled out letter by letter.

use macroquad::prelude::*;

use super::{ease_out_cubic, style, text_centered, FxPalette};
use crate::graphics::text::measure_text;

const TRACE: f32 = 0.6;
const LETTERS_AT: f32 = 0.35;
const LETTER_STEP: f32 = 0.05;
const OUT: f32 = 1.7;
pub const DONE: f32 = 2.0;

pub(super) struct PerfectFx {
    pub start: f64,
}

impl PerfectFx {
    pub fn done(&self, now: f64) -> bool {
        (now - self.start) as f32 > DONE
    }

    pub fn draw(&self, board: Rect, cell: f32, now: f64, pal: &FxPalette) {
        let t = (now - self.start) as f32;
        let fade = 1.0 - ((t - OUT) / (DONE - OUT)).clamp(0.0, 1.0);
        let color = pal.perfect;

        // The glow inside the board.
        let glow = ((t - 0.25) / 0.3).clamp(0.0, 1.0) * fade;
        if glow > 0.0 {
            for i in 0..4 {
                let k = i as f32 / 4.0;
                let r = Rect::new(board.x + board.w * k * 0.25, board.y + board.h * k * 0.25, board.w * (1.0 - k * 0.5), board.h * (1.0 - k * 0.5));
                draw_rectangle(r.x, r.y, r.w, r.h, style::alpha(color, 0.035 * glow));
            }
        }

        // The light running around the edge.
        let p = ease_out_cubic((t / TRACE).clamp(0.0, 1.0));
        if p > 0.0 && fade > 0.0 {
            let e = Rect::new(board.x - 3.0, board.y - 3.0, board.w + 6.0, board.h + 6.0);
            let perimeter = 2.0 * (e.w + e.h);
            let width = (cell * 0.12).max(2.0);
            let mut done = 0.0;
            let mut head = vec2(e.x, e.y);
            for (from, to) in [
                (vec2(e.x, e.y), vec2(e.right(), e.y)),
                (vec2(e.right(), e.y), vec2(e.right(), e.bottom())),
                (vec2(e.right(), e.bottom()), vec2(e.x, e.bottom())),
                (vec2(e.x, e.bottom()), vec2(e.x, e.y)),
            ] {
                let len = from.distance(to);
                let left = p * perimeter - done;
                if left <= 0.0 {
                    break;
                }
                let end = from + (to - from) * (left / len).min(1.0);
                draw_line(from.x, from.y, end.x, end.y, width, style::alpha(color, fade));
                head = end;
                done += len;
            }
            if p < 1.0 {
                draw_circle(head.x, head.y, width * 1.6, style::alpha(WHITE, fade));
            }
        }

        // PERFECT / CLEAR.
        let size = cell * 1.05;
        let mut i = 0;
        for (word, row) in [("PERFECT", 0.36), ("CLEAR", 0.36 + 1.35 * cell / board.h)] {
            let chars: Vec<String> = word.chars().map(String::from).collect();
            let widths: Vec<f32> = chars.iter().map(|c| measure_text(c, None, size as u16, 1.0).width).collect();
            let gap = size * 0.12;
            let total: f32 = widths.iter().sum::<f32>() + gap * (chars.len() - 1) as f32;
            let mut x = board.center().x - total / 2.0;
            let y = board.y + board.h * row;
            for (k, ch) in chars.iter().enumerate() {
                let at = LETTERS_AT + i as f32 * LETTER_STEP;
                let q = ((t - at) / 0.15).clamp(0.0, 1.0);
                if q > 0.0 && fade > 0.0 {
                    let rise = (1.0 - ease_out_cubic(q)) * cell * 0.6;
                    let cx = x + widths[k] / 2.0;
                    for (dx, dy) in [(-2.0, 0.0), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0)] {
                        text_centered(ch, cx + dx, y + rise + dy, size, style::alpha(pal.outline, q * fade));
                    }
                    text_centered(ch, cx, y + rise, size, style::alpha(color, q * fade));
                }
                x += widths[k] + gap;
                i += 1;
            }
        }
    }
}
