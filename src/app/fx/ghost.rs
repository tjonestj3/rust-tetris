//! The ghost block's throw: a reticle pulses on the target cell, then the block comes in from
//! above the board as a comet with an ember trail. When it lands: an impact ring, embers, a short
//! kick of the board, and the cells beside it jiggle.

use macroquad::prelude::*;

use super::{cell_pos, ease_in_cubic, ease_out_cubic, hash01, style, FxPalette};
use crate::game::Game;

/// Fractions of the throw: the reticle until the comet sets off, the comet until it lands.
const AIM_END: f32 = 0.5;
const COMET_START: f32 = 0.4;
/// Seconds of the impact.
const IMPACT: f32 = 0.45;
const TRAIL: usize = 12;

/// The ghost block's colour (the game places it in this colour: Game::finish_ghost_throw).
const GHOST: Color = Color::new(0.8, 0.8, 1.0, 1.0);

/// The throw in flight, drawn from the game's own progress (it pauses with the game).
pub fn draw_throw(game: &Game, board: Rect, cell: f32, now: f64, pal: &FxPalette) {
    if !game.is_ghost_throw_active() || game.state == crate::game::GameState::GameOver {
        return;
    }
    let Some((progress, _, _)) = game.get_ghost_throw_info() else { return };
    let p = progress as f32;
    let (tx, ty) = game.ghost_throw_target;
    let target = cell_pos(board, cell, tx as f32, ty as f32);

    if p < AIM_END {
        let pulse = 0.5 + 0.5 * (now as f32 * 18.0).sin();
        let fade = 1.0 - ((p - (AIM_END - 0.1)) / 0.1).clamp(0.0, 1.0);
        let m = cell * (0.12 + 0.1 * pulse);
        let len = cell * 0.32;
        let color = style::alpha(pal.card_edge, (0.6 + 0.4 * pulse) * fade);
        let w = (cell * 0.09).max(2.0);
        for (qx, qy) in [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)] {
            let x = target.x + qx * cell + if qx > 0.0 { m } else { -m };
            let y = target.y + qy * cell + if qy > 0.0 { m } else { -m };
            let dx = if qx > 0.0 { -len } else { len };
            let dy = if qy > 0.0 { -len } else { len };
            draw_line(x, y, x + dx, y, w, color);
            draw_line(x, y, x, y + dy, w, color);
        }
    }

    if p >= COMET_START {
        let from = vec2(board.right() + cell, board.y - cell * 3.0);
        let k = ease_in_cubic(((p - COMET_START) / (1.0 - COMET_START)).clamp(0.0, 1.0));
        for i in (0..=TRAIL).rev() {
            let q = (k - i as f32 * 0.035).max(0.0);
            let at = from + (target - from) * q;
            let r = cell * 0.42 * (1.0 - i as f32 / (TRAIL as f32 + 1.0));
            let color = if i == 0 { style::mix(pal.heat, WHITE, 0.7) } else { style::alpha(style::mix(pal.heat, pal.stamp, i as f32 / TRAIL as f32), 0.55 * (1.0 - i as f32 / (TRAIL as f32 + 1.0))) };
            draw_circle(at.x + cell / 2.0, at.y + cell / 2.0, r, color);
        }
        let head = from + (target - from) * k;
        (pal.block)(head.x, head.y, cell, GHOST);
    }
}

/// The landing, on the frame the throw ends.
pub(super) struct ImpactFx {
    pub start: f64,
    pub cell: (i32, i32),
}

impl ImpactFx {
    pub fn age(&self, now: f64) -> f32 {
        (now - self.start) as f32
    }

    pub fn done(&self, now: f64) -> bool {
        self.age(now) > IMPACT
    }

    /// The jiggle of the cell at (`x`, `y`): a scale around 1, for the cells beside the landing.
    pub fn jiggle(&self, x: i32, y: i32, now: f64) -> f32 {
        let (cx, cy) = self.cell;
        let beside = (x - cx).abs() + (y - cy).abs() == 1;
        let t = self.age(now);
        if !beside || !(0.0..0.4).contains(&t) {
            return 1.0;
        }
        1.0 + (t * 40.0).sin() * 0.12 * (1.0 - t / 0.4)
    }

    /// The board's kick, as a direction scaled by how hard (up to 1).
    pub fn shake(&self, now: f64) -> Vec2 {
        let t = self.age(now);
        if !(0.0..0.3).contains(&t) {
            return Vec2::ZERO;
        }
        let e = (-t * 12.0).exp();
        vec2((t * 70.0).sin(), (t * 55.0).cos()) * e
    }

    pub fn draw(&self, board: Rect, cell: f32, now: f64, pal: &FxPalette) {
        let t = self.age(now);
        if !(0.0..IMPACT).contains(&t) {
            return;
        }
        let c = cell_pos(board, cell, self.cell.0 as f32 + 0.5, self.cell.1 as f32 + 0.5);
        let p = t / IMPACT;
        let r = cell * 3.2 * ease_out_cubic(p);
        draw_ellipse_lines(c.x, c.y, r, r * 0.6, 0.0, (cell * 0.2) * (1.0 - p) + 1.0, style::alpha(pal.heat, 1.0 - p));
        // A flash on the block itself.
        let flash = (1.0 - t / 0.2).clamp(0.0, 1.0);
        if flash > 0.0 {
            draw_rectangle(c.x - cell / 2.0, c.y - cell / 2.0, cell, cell, style::alpha(WHITE, 0.7 * flash));
        }
        for i in 0..14u32 {
            let v = vec2((hash01(i, 21) - 0.5) * 8.0, -(3.0 + hash01(i, 22) * 5.0));
            let at = c + (v * t + vec2(0.0, 18.0 * t * t * 0.5)) * cell;
            let size = (cell * 0.1).max(2.0);
            draw_rectangle(at.x, at.y, size, size, style::alpha(style::mix(pal.heat, pal.stamp, hash01(i, 23)), 1.0 - p));
        }
    }
}
