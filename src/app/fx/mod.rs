//! The touch screen's effects: the hard drop's streak and dip, the line clear's pop and fall, and
//! JONES'IN! (see the Animation Lab mockups). One file per effect; every colour comes from an
//! [`FxPalette`], which a theme will supply (docs/superpowers/specs/2026-11-20-themes-design.md).
//!
//! The app tells [`Fx`] what the game looked like before each frame ([`Fx::observe`], the same
//! moments the haptics and sounds use), and the board drawing asks it how to draw each cell and
//! what to draw under and over the blocks. Nothing here changes the game.

mod clear;
mod combo;
mod drop;
mod gameover;
mod ghost;
pub(super) mod jonesin;
mod label;
mod level;
mod perfect;
mod tspin;

use macroquad::prelude::*;

use crate::game::{Game, GameState};
use crate::game::config::{BOARD_WIDTH, BUFFER_HEIGHT};
use crate::graphics::style;
use crate::graphics::text::{draw_text, measure_text};
use crate::graphics::view::Layout;

pub(super) use crate::graphics::theme::FxPalette;

/// How to draw one settled cell this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct CellFx {
    /// Pixels to move it (falling into place after a clear).
    pub dy: f32,
    /// 1 = normal size; popping cells swell, then shrink away.
    pub scale: f32,
    /// 0..1 towards white.
    pub white: f32,
    /// 0..1 towards the palette's grey (a finished game).
    pub grey: f32,
}

impl Default for CellFx {
    fn default() -> Self {
        Self { dy: 0.0, scale: 1.0, white: 0.0, grey: 0.0 }
    }
}

/// What the game looked like before a frame's input and update (from app::FeltState).
#[derive(Debug, Clone, Copy)]
pub(super) struct Before {
    pub hard_drops: u32,
    pub clearing: bool,
    pub clears: u32,
    /// A ghost block was in flight.
    pub throwing: bool,
    pub level: u32,
}

#[derive(Default)]
pub(super) struct Fx {
    pub palette: FxPalette,
    drop: Option<drop::DropFx>,
    clear: Option<clear::ClearFx>,
    tspin: Option<tspin::TspinFx>,
    combo: combo::ComboFx,
    perfect: Option<perfect::PerfectFx>,
    game_over: Option<gameover::GameOverFx>,
    impact: Option<ghost::ImpactFx>,
    level: Option<level::LevelFx>,
    /// A clock that stands still (desktop `--freeze`, for screenshots).
    frozen_at: Option<f64>,
}

impl Fx {
    /// The effects' clock: `get_time()`, unless frozen.
    pub fn now(&self) -> f64 {
        self.frozen_at.unwrap_or_else(get_time)
    }

    /// Stop the effects' clock at `at` (desktop `--freeze`).
    pub fn freeze(&mut self, at: f64) {
        self.frozen_at = Some(at);
    }

    /// Note what happened since the frame began. `now` is `get_time()`.
    pub fn observe(&mut self, before: Before, game: &Game, now: f64) {
        let (hard_drops, was_clearing) = (before.hard_drops, before.clearing);
        if game.lock_was_t_spin && !was_clearing && game.is_clearing_lines() {
            self.tspin = Some(tspin::TspinFx::new(game, now));
        }
        self.combo.update(before.clears, game, now);
        if game.scoring_system.clears() > before.clears && game.scoring_system.last_clear().is_some_and(|c| c.perfect) {
            self.perfect = Some(perfect::PerfectFx { start: now });
        }
        if game.state != GameState::GameOver {
            self.game_over = None;
        }
        if self.tspin.as_ref().is_some_and(|f| f.done(now)) {
            self.tspin = None;
        }
        if self.perfect.as_ref().is_some_and(|f| f.done(now)) {
            self.perfect = None;
        }
        // A throw that landed (not one cut off by a game over and a new game).
        let (tx, ty) = game.ghost_throw_target;
        let landed = game.board.get_cell(tx, ty).is_some_and(|c| c.color().is_some());
        if before.throwing && !game.is_ghost_throw_active() && game.state != GameState::GameOver && landed {
            self.impact = Some(ghost::ImpactFx { start: now, cell: game.ghost_throw_target });
        }
        if self.impact.as_ref().is_some_and(|f| f.done(now)) {
            self.impact = None;
        }
        if game.level() > before.level && before.level > 0 {
            self.level = Some(level::LevelFx { start: now });
        }
        if self.level.as_ref().is_some_and(|f| f.done(now)) {
            self.level = None;
        }
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
        let (palette, frozen_at) = (self.palette, self.frozen_at);
        *self = Self { palette, frozen_at, ..Default::default() };
    }

    /// The game has ended (called every frame it's over; starts the greying and the card once).
    /// `high_score`: whether the score makes the leaderboard.
    pub fn game_over(&mut self, now: f64, high_score: bool) {
        if self.game_over.is_none() {
            self.game_over = Some(gameover::GameOverFx::new(now, high_score));
        }
    }

    /// Desktop screenshots (`--fx combo|perfect|tspin|over|level|impact|throw`): set an effect up part way through.
    #[cfg(not(target_os = "ios"))]
    pub fn demo(&mut self, which: &str, game: &Game, now: f64) {
        match which {
            "combo" => self.combo = combo::ComboFx::demo(3, now - 1.0),
            "perfect" => self.perfect = Some(perfect::PerfectFx { start: now - 0.9 }),
            "tspin" => {
                let mut g = game.clone();
                let y = (crate::game::config::BOARD_HEIGHT + BUFFER_HEIGHT) as i32 - 4;
                g.last_lock_cells = vec![(3, y), (4, y), (5, y), (4, y + 1)];
                g.clearing_lines = vec![y as usize, y as usize + 1];
                self.tspin = Some(tspin::TspinFx::new(&g, now - 0.3));
            }
            "over" => self.game_over = Some(gameover::GameOverFx::new(now - 2.5, game.score > 0)),
            "level" => self.level = Some(level::LevelFx { start: now - 0.12 }),
            "impact" => {
                let y = (crate::game::config::BOARD_HEIGHT + BUFFER_HEIGHT) as i32 - 1;
                self.impact = Some(ghost::ImpactFx { start: now - 0.1, cell: (2, y) });
            }
            _ => {}
        }
    }

    /// Once the game-over card is up: `Some(high_score)`, and its buttons can be tapped.
    pub fn game_over_card(&self, now: f64) -> Option<bool> {
        self.game_over.as_ref().filter(|g| g.ready(now)).map(|g| g.high_score)
    }

    /// The game-over card (over everything, in screen pixels).
    pub fn draw_game_over(&self, game: &Game, layout: &Layout, now: f64) {
        if let Some(g) = &self.game_over {
            g.draw(game, layout, now, &self.palette);
        }
    }

    /// How far to move the whole board this frame: the dip after a hard drop and the kick when
    /// JONES'IN!'s last letter lands.
    pub fn board_offset(&self, game: &Game, now: f64, cell: f32) -> Vec2 {
        let mut d = vec2(0.0, 0.0);
        if let Some(drop) = &self.drop {
            d.y += drop::dip((now - drop.start) as f32) * cell * 0.14;
        }
        if let Some(impact) = &self.impact {
            d += impact.shake(now) * cell * 0.18;
        }
        if game.state == GameState::GameOver {
            return d;
        }
        d + jonesin::shake(game) * cell * 0.22
    }

    /// 1 just after the level goes up, falling to 0 (the stats panel's level number pops).
    pub fn level_pop(&self, now: f64) -> f32 {
        self.level.as_ref().map_or(0.0, |l| l.pop(now))
    }

    /// The ring around the level number as it pops.
    pub fn level_ring(&self, c: Vec2, size: f32, now: f64) {
        level::ring(c, size, self.level_pop(now), &self.palette);
    }

    /// How to draw the settled cell at board column `x`, row `y` (board rows, counting the hidden
    /// ones), or `None` to leave it out (popped).
    pub fn cell(&self, game: &Game, x: usize, y: usize, now: f64, cell: f32) -> Option<CellFx> {
        if let Some(g) = &self.game_over {
            return Some(CellFx { grey: g.grey(y.saturating_sub(BUFFER_HEIGHT), now), ..Default::default() });
        }
        let Some(clear) = &self.clear else {
            let scale = self.impact.as_ref().map_or(1.0, |i| i.jiggle(x as i32, y as i32, now));
            return Some(CellFx { scale, ..Default::default() });
        };
        if game.is_clearing_lines() {
            if game.clearing_lines.contains(&y) {
                let t = game.get_clear_animation_progress() as f32 * crate::game::config::LINE_CLEAR_ANIMATION_TIME as f32;
                return clear::pop(t, x as f32, clear.origin).map(|(scale, white)| CellFx { scale, white, ..Default::default() });
            }
            return Some(CellFx::default());
        }
        let scale = self.impact.as_ref().map_or(1.0, |i| i.jiggle(x as i32, y as i32, now));
        Some(CellFx { dy: clear.fall_dy(y, now) * cell, scale, ..Default::default() })
    }

    /// Under the blocks: the hard drop's streak.
    pub fn draw_under(&self, board: Rect, cell: f32, now: f64) {
        if let Some(drop) = &self.drop {
            drop.draw_streak(board, cell, now);
        }
    }

    /// Over the blocks: the clear's shards, the hard drop's dust, the T-spin, the combo chip, a
    /// perfect clear, a ghost block's throw and landing, and a level up.
    pub fn draw_over(&self, game: &Game, board: Rect, cell: f32, now: f64) {
        ghost::draw_throw(game, board, cell, now, &self.palette);
        if let Some(i) = &self.impact {
            i.draw(board, cell, now, &self.palette);
        }
        if let Some(l) = &self.level {
            l.draw(board, cell, now, &self.palette);
        }
        if let Some(clear) = &self.clear {
            clear.draw_shards(board, cell, now);
        }
        if let Some(drop) = &self.drop {
            drop.draw_dust(board, cell, now, self.palette.dust);
        }
        if let Some(t) = &self.tspin {
            t.draw(board, cell, now, &self.palette);
        }
        if self.game_over.is_none() {
            self.combo.draw(board, cell, now, &self.palette);
        }
        if let Some(p) = &self.perfect {
            p.draw(board, cell, now, &self.palette);
        }
    }
}

/// A settled block, drawn the way `fx` says.
pub(super) fn draw_block(x: f32, y: f32, cell: f32, color: Color, fx: CellFx, pal: &FxPalette) {
    let size = cell * fx.scale;
    let color = if fx.grey > 0.0 { style::mix(color, pal.grey, fx.grey * 0.85) } else { color };
    let color = if fx.white > 0.0 { style::mix(color, WHITE, fx.white) } else { color };
    (pal.block)(x + (cell - size) / 2.0, y + fx.dy + (cell - size) / 2.0, size, color);
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

/// Text centred on (`cx`, `cy`).
fn text_centered(text: &str, cx: f32, cy: f32, size: f32, color: Color) {
    let w = measure_text(text, None, size.max(1.0) as u16, 1.0).width;
    draw_text(text, cx - w / 2.0, cy + size * 0.35, size, color);
}
