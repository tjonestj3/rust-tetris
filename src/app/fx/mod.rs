//! The touch screen's effects: the hard drop's streak and dip, the line clear's pop and fall, and
//! JONES'IN! (see the Animation Lab mockups). One file per effect; every colour comes from an
//! [`FxPalette`], which a theme will supply (docs/superpowers/specs/2026-11-20-themes-design.md).
//!
//! The app tells [`Fx`] what the game looked like before each frame ([`Fx::observe`], the same
//! moments the haptics and sounds use), and the board drawing asks it how to draw each cell and
//! what to draw under and over the blocks. Nothing here changes the game.

mod clear;
mod drop;
pub(super) mod jonesin;

use macroquad::prelude::*;

use crate::game::Game;
use crate::game::config::{BOARD_WIDTH, BUFFER_HEIGHT};
use crate::graphics::style::{self, CYAN, GOLD, PINK, VIOLET};

/// Every colour the effects draw with.
#[derive(Debug, Clone, Copy)]
pub(super) struct FxPalette {
    /// JONES'IN!'s letters, left to right.
    pub word: [Color; 3],
    /// The letters for a back-to-back Tetris.
    pub hot: [Color; 3],
    /// The letters' raised edge, and their outline.
    pub edge: Color,
    pub outline: Color,
    /// The screen flash, the shockwave rings and the dim behind the word.
    pub flash: Color,
    pub rings: [Color; 2],
    pub dim: Color,
    /// BACK-TO-BACK's banner and its text; the X2 stamp.
    pub banner: Color,
    pub banner_ink: Color,
    pub stamp: Color,
    /// The heat rising behind a back-to-back word.
    pub heat: Color,
    /// Dust where a hard-dropped piece lands.
    pub dust: Color,
}

impl Default for FxPalette {
    /// Neon, the look the game has today.
    fn default() -> Self {
        let orange = Color::new(1.0, 0.54, 0.24, 1.0);
        Self {
            word: [CYAN, PINK, GOLD],
            hot: [GOLD, orange, PINK],
            edge: Color::new(0.16, 0.08, 0.38, 1.0),
            outline: Color::new(0.04, 0.02, 0.12, 1.0),
            flash: WHITE,
            rings: [CYAN, PINK],
            dim: Color::new(0.04, 0.02, 0.12, 1.0),
            banner: PINK,
            banner_ink: Color::new(0.08, 0.02, 0.18, 1.0),
            stamp: GOLD,
            heat: orange,
            dust: style::mix(VIOLET, WHITE, 0.7),
        }
    }
}

/// How to draw one settled cell this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct CellFx {
    /// Pixels to move it (falling into place after a clear).
    pub dy: f32,
    /// 1 = normal size; popping cells swell, then shrink away.
    pub scale: f32,
    /// 0..1 towards white.
    pub white: f32,
}

impl Default for CellFx {
    fn default() -> Self {
        Self { dy: 0.0, scale: 1.0, white: 0.0 }
    }
}

#[derive(Default)]
pub(super) struct Fx {
    pub palette: FxPalette,
    drop: Option<drop::DropFx>,
    clear: Option<clear::ClearFx>,
}

impl Fx {
    /// Note what happened since the frame began: `hard_drops` and `was_clearing` are from before
    /// the frame's input and update (app::FeltState). `now` is `get_time()`.
    pub fn observe(&mut self, hard_drops: u32, was_clearing: bool, game: &Game, now: f64) {
        if game.hard_drops != hard_drops {
            if let Some(mark) = &game.last_hard_drop {
                self.drop = Some(drop::DropFx::new(mark, now));
            }
        }
        if !was_clearing && game.is_clearing_lines() {
            self.clear = Some(clear::ClearFx::new(game, now));
        }
        if was_clearing && !game.is_clearing_lines() {
            if let Some(c) = &mut self.clear {
                c.fall_start = Some(now);
            }
        }
    }

    /// Forget everything (a new game).
    pub fn reset(&mut self) {
        self.drop = None;
        self.clear = None;
    }

    /// How far to move the whole board this frame: the dip after a hard drop and the kick when
    /// JONES'IN!'s last letter lands.
    pub fn board_offset(&self, game: &Game, now: f64, cell: f32) -> Vec2 {
        let mut d = vec2(0.0, 0.0);
        if let Some(drop) = &self.drop {
            d.y += drop::dip((now - drop.start) as f32) * cell * 0.14;
        }
        d + jonesin::shake(game) * cell * 0.22
    }

    /// How to draw the settled cell at board column `x`, row `y` (board rows, counting the hidden
    /// ones), or `None` to leave it out (popped).
    pub fn cell(&self, game: &Game, x: usize, y: usize, now: f64, cell: f32) -> Option<CellFx> {
        let Some(clear) = &self.clear else { return Some(CellFx::default()) };
        if game.is_clearing_lines() {
            if game.clearing_lines.contains(&y) {
                let t = game.get_clear_animation_progress() as f32 * crate::game::config::LINE_CLEAR_ANIMATION_TIME as f32;
                return clear::pop(t, x as f32, clear.origin).map(|(scale, white)| CellFx { scale, white, ..Default::default() });
            }
            return Some(CellFx::default());
        }
        Some(CellFx { dy: clear.fall_dy(y, now) * cell, ..Default::default() })
    }

    /// Under the blocks: the hard drop's streak.
    pub fn draw_under(&self, board: Rect, cell: f32, now: f64) {
        if let Some(drop) = &self.drop {
            drop.draw_streak(board, cell, now);
        }
    }

    /// Over the blocks: the clear's shards and the hard drop's dust.
    pub fn draw_over(&self, board: Rect, cell: f32, now: f64) {
        if let Some(clear) = &self.clear {
            clear.draw_shards(board, cell, now);
        }
        if let Some(drop) = &self.drop {
            drop.draw_dust(board, cell, now, self.palette.dust);
        }
    }
}

/// A settled block, drawn the way `fx` says.
pub(super) fn draw_block(x: f32, y: f32, cell: f32, color: Color, fx: CellFx) {
    let size = cell * fx.scale;
    let color = if fx.white > 0.0 { style::mix(color, WHITE, fx.white) } else { color };
    style::block(x + (cell - size) / 2.0, y + fx.dy + (cell - size) / 2.0, size, color);
}

/// The screen position of board column `x`, row `y` (board rows, counting the hidden ones).
fn cell_pos(board: Rect, cell: f32, x: f32, y: f32) -> Vec2 {
    vec2(board.x + x * cell, board.y + (y - BUFFER_HEIGHT as f32) * cell)
}

/// A number in 0..1 that's the same every time for the same `a` and `b` (for scattering shards
/// and confetti without keeping any state).
fn hash01(a: u32, b: u32) -> f32 {
    let mut h = a.wrapping_mul(0x9E37_79B1) ^ b.wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0xFFFF) as f32 / 65536.0
}

fn ease_out_cubic(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(3)
}

fn ease_in_cubic(x: f32) -> f32 {
    x * x * x
}

fn ease_out_bounce(x: f32) -> f32 {
    let (n, d) = (7.5625, 2.75);
    if x < 1.0 / d {
        n * x * x
    } else if x < 2.0 / d {
        let x = x - 1.5 / d;
        n * x * x + 0.75
    } else if x < 2.5 / d {
        let x = x - 2.25 / d;
        n * x * x + 0.9375
    } else {
        let x = x - 2.625 / d;
        n * x * x + 0.984375
    }
}

/// The middle of the board, as a column (for an effect with nowhere better to start from).
const MIDDLE: f32 = (BOARD_WIDTH as f32 - 1.0) / 2.0;
