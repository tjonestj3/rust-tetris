//! The game screen on touch devices: the board as big as the screen allows, Next / Hold / score
//! in compact panels beside it or above it (graphics::view::PlayLayout), the backdrop over the
//! whole screen.
//!
//! The board, pieces and effects are the desktop drawing code, drawn through a camera that maps
//! the canvas's board onto the layout's board rect. The panels, pause screen and gesture hint are
//! drawn here in screen pixels.

use macroquad::prelude::*;

use super::{
    draw_ghost_block_cursor, draw_ghost_throw_animation, draw_legacy_board_with_data, draw_legacy_falling_piece,
    draw_legacy_ghost_piece, draw_line_clear_animation, draw_tetris_celebration,
};
use crate::game::config::{BOARD_WIDTH, BUFFER_HEIGHT, LINES_PER_LEVEL, VISIBLE_HEIGHT};
use crate::game::{Game, GameState};
use crate::graphics::style::{self, alpha, CYAN, GOLD, GREEN, INK, PINK, SOFT_INK, TERM_DIM, TERM_GREEN, VIOLET};
use crate::graphics::text::{draw_text, measure_text};
use crate::graphics::view::{map_camera, screen_camera, HudStyle, Layout, PlayLayout, CANVAS_BOARD};
use crate::tetromino::{Tetromino, TetrominoType};
use super::duo::Callout;
use super::fx::{self, Fx};

const LABEL: Color = SOFT_INK;
const VALUE: Color = INK;

/// The look of the panels and overlays: modern glass, or green terminal text to go with the
/// legacy board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Look {
    Modern,
    Legacy,
}

impl Look {
    fn of(game: &Game) -> Self {
        if game.is_legacy_mode() { Look::Legacy } else { Look::Modern }
    }

    /// `modern` or `legacy`, whichever this look uses.
    fn pick(self, modern: Color, legacy: Color) -> Color {
        match self {
            Look::Modern => modern,
            Look::Legacy => legacy,
        }
    }

    /// A label in `accent` (modern) or dim green.
    fn label(self, accent: Color) -> Color {
        self.pick(accent, TERM_DIM)
    }

    fn value(self) -> Color {
        self.pick(VALUE, TERM_GREEN)
    }
}

/// On and off a few times a second, like a terminal cursor.
fn blink(rate: f64) -> bool {
    (get_time() * rate) as i64 % 2 == 0
}

/// The gestures, for the pause screen and the first-game hint: (gesture, what it does).
const GESTURES: [(&str, &str); 8] = [
    ("TAP", "rotate"),
    ("2-FINGER TAP", "rotate back"),
    ("DRAG", "move"),
    ("DRAG DOWN", "soft drop"),
    ("FLICK DOWN", "hard drop"),
    ("FLICK UP", "hold"),
    ("TAP HOLD", "hold"),
    ("TAP SCORE", "ghost block"),
];

/// What's drawn over the game besides the game itself.
pub(super) struct Overlay<'a> {
    /// 0..1: fades the first-game gesture hint in and out.
    pub hint: f32,
    /// Seconds left while counting back into play (super::duo::PauseFlow).
    pub countdown: Option<f32>,
    /// The last special clear's callout (shown in the open Duo's feed).
    pub callout: &'a Callout,
    /// The effects: hard drop, line clear, JONES'IN!.
    pub fx: &'a Fx,
}

/// Draw the game screen.
pub(super) fn render(game: &Game, background: &Texture2D, layout: &Layout, overlay: &Overlay) {
    let hint = overlay.hint;
    let p = &layout.play;
    let legacy = game.is_legacy_mode();
    let look = Look::of(game);

    let _ = background;
    screen_camera();
    if legacy {
        clear_background(BLACK);
        // The classic look: the desktop drawing, through the board camera.
        map_camera(CANVAS_BOARD, p.board);
        draw_legacy_board_with_data(&game.board);
        if game.is_clearing_lines() {
            draw_line_clear_animation(game);
        } else {
            if let Some(ghost) = game.calculate_ghost_piece() {
                draw_legacy_ghost_piece(&ghost);
            }
            if let Some(ref piece) = game.current_piece {
                draw_legacy_falling_piece(piece);
            }
        }
    } else {
        style::backdrop(get_time() as f32);
        draw_board(game, p, layout.unit, overlay.fx);
        map_camera(CANVAS_BOARD, p.board);
    }
    if game.is_ghost_cursor_visible() {
        draw_ghost_block_cursor(game);
    }
    // The legacy look keeps its stick figure; the other draws a comet (super::fx).
    if legacy && game.is_ghost_throw_active() && game.state != GameState::GameOver {
        draw_ghost_throw_animation(game);
    }

    // The panels.
    screen_camera();
    if game.ghost_block_placement_mode {
        draw_aim_banner(p, layout.unit, look);
    }
    let u = layout.unit;
    let next = panel(p.next, "NEXT", CYAN, p, u, look);
    draw_piece(next, Some(game.next_piece), p.cell, 1.0, look);
    let hold = panel(p.hold, "HOLD", PINK, p, u, look);
    draw_piece(hold, game.held_piece, p.cell, if game.can_hold() { 1.0 } else { 0.35 }, look);
    if game.held_piece.is_none() {
        // The panel is a button (input::touch): say so until something's in it.
        let size = fit_size("TAP TO HOLD", label_size(p, u) * 0.9, hold.w);
        let w = measure_text("TAP TO HOLD", None, size.max(1.0) as u16, 1.0).width;
        let color = look.pick(alpha(PINK, 0.7), TERM_DIM);
        draw_text("TAP TO HOLD", hold.center().x - w / 2.0, hold.center().y + size * 0.35, size, color);
    }
    draw_stats(game, p, u, look, overlay.fx);
    if let Some(feed) = p.feed {
        draw_feed(game, feed, p, u, look, overlay.callout);
    }

    if hint > 0.0 && game.state == GameState::Playing {
        draw_hint(p, u, hint, look);
    }

    // The TETRIS celebration, over everything, at the size of the whole canvas fitted to the screen.
    // Not behind the game-over card: the celebration's clock stops with the game.
    if game.is_tetris_celebration_active() && game.state != GameState::GameOver {
        if legacy {
            map_camera(layout.canvas_src, layout.game);
            draw_tetris_celebration(game);
            screen_camera();
        } else {
            screen_camera();
            fx::jonesin::draw(game, layout, &overlay.fx.palette);
        }
    }

    match game.state {
        GameState::Paused => match overlay.countdown {
            Some(left) => draw_countdown(p, u, left, look),
            None => draw_pause(layout, look),
        },
        GameState::GameOver => {
            screen_camera();
            overlay.fx.draw_game_over(game, layout, overlay.fx.now());
        }
        _ => {}
    }
    screen_camera();
}

/// The board: a dark glass well with a glowing edge, the settled blocks, where the piece will
/// land, and the piece.
fn draw_board(game: &Game, p: &PlayLayout, u: f32, fx: &Fx) {
    let now = fx.now();
    let cell = p.board.w / BOARD_WIDTH as f32;
    // The board kicks with a hard drop and with JONES'IN! (super::fx).
    let kick = fx.board_offset(game, now, cell);
    let b = Rect::new(p.board.x + kick.x, p.board.y + kick.y, p.board.w, p.board.h);
    let pad = (3.0 * u).round();
    let well = Rect::new(b.x - pad, b.y - pad, b.w + 2.0 * pad, b.h + 2.0 * pad);
    let radius = 8.0 * u;
    style::halo(well, radius, 10.0 * u, CYAN, 0.16);
    style::round_rect(well, radius, Color::new(0.06, 0.05, 0.19, 0.86), Color::new(0.10, 0.04, 0.22, 0.90));

    // Faint column stripes and row lines.
    for x in 0..BOARD_WIDTH {
        if x % 2 == 1 {
            draw_rectangle(b.x + x as f32 * cell, b.y, cell, b.h, Color::new(1.0, 1.0, 1.0, 0.025));
        }
    }
    for y in 1..VISIBLE_HEIGHT {
        draw_rectangle(b.x, (b.y + y as f32 * cell).round(), b.w, 1.0, Color::new(0.8, 0.85, 1.0, 0.06));
    }

    fx.draw_under(b, cell, now);
    for y in 0..VISIBLE_HEIGHT {
        for x in 0..BOARD_WIDTH {
            if let Some(color) = game.board.get_cell(x as i32, (y + BUFFER_HEIGHT) as i32).and_then(|c| c.color()) {
                if let Some(look) = fx.cell(game, x, y + BUFFER_HEIGHT, now, cell) {
                    fx::draw_block(b.x + x as f32 * cell, b.y + y as f32 * cell, cell, color, look, &fx.palette);
                }
            }
        }
    }

    if !game.is_clearing_lines() {
        let visible = |(x, y): (i32, i32)| (y >= BUFFER_HEIGHT as i32).then(|| (b.x + x as f32 * cell, b.y + (y - BUFFER_HEIGHT as i32) as f32 * cell));
        if let Some(ghost) = game.calculate_ghost_piece() {
            let color = ghost.color();
            for (x, y) in ghost.absolute_blocks().into_iter().filter_map(visible) {
                style::ghost_block(x, y, cell, color);
            }
        }
        if let Some(ref piece) = game.current_piece {
            let color = piece.color();
            for (x, y) in piece.absolute_blocks().into_iter().filter_map(visible) {
                style::block(x, y, cell, color);
            }
        }
    }

    style::round_outline(well, radius, (2.0 * u).max(1.0), alpha(CYAN, 0.85));
    fx.draw_over(game, b, cell, now);
}

/// A panel's frame and label; returns the area left for its content.
fn panel(r: Rect, label: &str, accent: Color, p: &PlayLayout, u: f32, look: Look) -> Rect {
    let pad = (p.cell * 0.3).max(7.0 * u);
    let size = label_size(p, u);
    if look == Look::Legacy {
        // The label is in the top border.
        let inside = style::term_frame(r, label, size, TERM_GREEN, TERM_GREEN);
        return Rect::new(r.x + pad, inside.y + pad * 0.3, r.w - 2.0 * pad, (inside.h - pad * 0.6).max(0.0));
    }
    style::panel(r, accent, u);
    draw_text(label, r.x + pad, r.y + pad + size * 0.8, size, accent);
    let top = r.y + pad + size + pad * 0.4;
    Rect::new(r.x + pad, top, r.w - 2.0 * pad, r.bottom() - pad - top)
}

fn label_size(p: &PlayLayout, u: f32) -> f32 {
    (p.cell * 0.58).clamp(11.0 * u, 19.0 * u)
}

/// A piece centred in `area`, at most 0.8 of a board cell per block.
fn draw_piece(area: Rect, piece: Option<TetrominoType>, cell: f32, alpha: f32, look: Look) {
    let Some(kind) = piece else { return };
    let blocks = Tetromino::new(kind).blocks;
    let (min_x, max_x) = (blocks.iter().map(|b| b.0).min().unwrap_or(0), blocks.iter().map(|b| b.0).max().unwrap_or(0));
    let (min_y, max_y) = (blocks.iter().map(|b| b.1).min().unwrap_or(0), blocks.iter().map(|b| b.1).max().unwrap_or(0));
    let (cols, rows) = ((max_x - min_x + 1) as f32, (max_y - min_y + 1) as f32);
    let b = (area.w / 4.4).min(area.h / 2.4).min(cell * 0.8).floor().max(2.0);
    let x0 = (area.x + (area.w - cols * b) / 2.0).round();
    let y0 = (area.y + (area.h - rows * b) / 2.0).round();
    let mut color = look.pick(kind.color(), TERM_GREEN);
    color.a *= alpha;
    for (dx, dy) in &blocks {
        let (x, y) = (x0 + (dx - min_x) as f32 * b, y0 + (dy - min_y) as f32 * b);
        match look {
            Look::Modern => style::block(x, y, b, color),
            Look::Legacy => style::term_block(x, y, b, color),
        }
    }
}

/// Text sized to fit `max_w`, no bigger than `size`.
fn fit_size(text: &str, size: f32, max_w: f32) -> f32 {
    let w = measure_text(text, None, size.max(1.0) as u16, 1.0).width;
    if w > max_w && w > 0.0 { size * max_w / w } else { size }
}

fn draw_stats(game: &Game, p: &PlayLayout, u: f32, look: Look, fx: &Fx) {
    let pop = fx.level_pop(fx.now());
    let ghosts = game.ghost_blocks_available;
    let aiming = game.ghost_block_placement_mode;
    // Ghost blocks blink in the legacy look, faster while aiming.
    let lit = blink(if aiming { 5.0 } else { 2.5 });
    let pad = (p.cell * 0.3).max(7.0 * u);
    let label = label_size(p, u);
    let r = match look {
        Look::Modern => {
            let r = p.stats;
            style::panel(r, GOLD, u);
            if ghosts > 0 {
                // The panel is a button (input::touch) while there's a ghost block to use: a
                // violet glow that breathes, brighter while aiming.
                let pulse = (get_time() as f32 * 3.0).sin() * 0.5 + 0.5;
                let strength = if aiming { 0.45 + 0.30 * pulse } else { 0.25 + 0.20 * pulse };
                style::halo(r, 10.0 * u, 10.0 * u, VIOLET, strength);
                style::round_outline(r, 10.0 * u, (2.0 * u).max(1.5), alpha(VIOLET, 0.75 + 0.25 * pulse));
            }
            r
        }
        Look::Legacy => {
            let rule = if ghosts > 0 && !lit { TERM_DIM } else { TERM_GREEN };
            let inside = style::term_frame(p.stats, "STATS", label, rule, TERM_GREEN);
            // Laid out below as if the panel were the part between the borders.
            Rect::new(p.stats.x, inside.y - pad * 0.7, p.stats.w, inside.h + pad * 1.4)
        }
    };
    let inner_w = r.w - 2.0 * pad;
    let score = game.score.to_string();
    let level = game.level().to_string();
    let lines = game.lines_cleared().to_string();

    if p.style == HudStyle::TopStrip {
        // Score big in the middle, level and lines under it.
        let cx = r.center().x;
        let score_size = fit_size(&score, (r.h * 0.42).min(p.cell * 1.3), inner_w);
        let sub = if ghosts > 0 {
            format!("LEVEL {level}   LINES {lines}   GHOST {ghosts}")
        } else {
            format!("LEVEL {level}   LINES {lines}")
        };
        let sub_size = fit_size(&sub, label, inner_w);
        let total = label + score_size + sub_size + pad * 0.6;
        let mut y = r.y + (r.h - total) / 2.0;
        centered("SCORE", cx, y + label * 0.8, label, look.label(GOLD));
        y += label + pad * 0.2;
        centered(&score, cx, y + score_size * 0.8, score_size, look.value());
        y += score_size + pad * 0.4;
        // The level just went up: the line pops in gold, with a ring.
        let sub_color = style::mix(look.pick(GREEN, TERM_GREEN), look.pick(GOLD, TERM_GREEN), pop);
        let sub_big = sub_size * (1.0 + 0.15 * pop);
        centered(&sub, cx, y + sub_big * 0.8, sub_big, sub_color);
        fx.level_ring(vec2(cx, y + sub_size * 0.5), sub_size * 2.0, fx.now());
        return;
    }

    // A column: SCORE over a big number, then LEVEL and LINES side by side.
    let x = r.x + pad;
    let mut y = r.y + pad;
    draw_text("SCORE", x, y + label * 0.8, label, look.label(GOLD));
    y += label + pad * 0.3;
    let score_size = fit_size(&score, p.cell * 1.55, inner_w);
    draw_text(&score, x, y + score_size * 0.8, score_size, look.value());
    y += score_size + pad * 0.9;

    let half = inner_w / 2.0;
    let widest = if level.len() >= lines.len() { &level } else { &lines };
    let value_size = fit_size(widest, p.cell * 1.05, half - pad * 0.5);
    if y + label + value_size <= r.bottom() - pad * 0.5 {
        draw_text("LEVEL", x, y + label * 0.8, label, look.label(GREEN));
        draw_text("LINES", x + half, y + label * 0.8, label, look.label(CYAN));
        y += label + pad * 0.3;
        // The level just went up: the number pops in gold, with a ring.
        let level_size = value_size * (1.0 + 0.35 * pop);
        draw_text(&level, x, y + level_size * 0.8, level_size, style::mix(look.value(), look.pick(GOLD, TERM_GREEN), pop));
        let lw = measure_text(&level, None, value_size as u16, 1.0).width;
        fx.level_ring(vec2(x + lw / 2.0, y + value_size * 0.45), value_size, fx.now());
        draw_text(&lines, x + half, y + value_size * 0.8, value_size, look.value());
        y += value_size + pad * 0.6;

        // Progress to the next level.
        let bar_h = match look {
            Look::Modern => (p.cell * 0.28).max(5.0 * u).round(),
            Look::Legacy => label,
        };
        if y + bar_h <= r.bottom() - pad * 0.5 {
            let done = (game.lines_cleared() % LINES_PER_LEVEL) as f32 / LINES_PER_LEVEL as f32;
            if look == Look::Legacy {
                style::term_bar(x, y + bar_h * 0.8, inner_w, label, done, TERM_GREEN);
            } else {
                let bar = Rect::new(x, y, inner_w, bar_h);
                style::round_rect(bar, bar_h / 2.0, Color::new(1.0, 1.0, 1.0, 0.10), Color::new(1.0, 1.0, 1.0, 0.06));
                if done > 0.0 {
                    let fill = Rect::new(x, y, (inner_w * done).max(bar_h), bar_h);
                    style::round_rect(fill, bar_h / 2.0, style::mix(GREEN, WHITE, 0.25), CYAN);
                }
            }
            y += bar_h + pad * 0.8;
        }
    }
    if ghosts > 0 && y + label <= r.bottom() - pad * 0.3 {
        let text = format!("GHOST BLOCKS {ghosts}");
        draw_text(&text, x, y + label * 0.8, fit_size(&text, label, inner_w), look.pick(VIOLET, TERM_GREEN));
        y += label + pad * 0.3;
        let tip = if game.ghost_block_placement_mode { "TAP TO THROW" } else { "TAP TO AIM" };
        if y + label <= r.bottom() - pad * 0.3 {
            let color = look.pick(style::mix(VIOLET, WHITE, 0.45), if lit { TERM_GREEN } else { TERM_DIM });
            draw_text(tip, x, y + label * 0.8, fit_size(tip, label * 0.85, inner_w), color);
            y += label + pad * 0.6;
        }
    }

    // Time, at the bottom of the panel when it's tall enough.
    let secs = game.game_time.max(0.0) as u64;
    let time = format!("{}:{:02}", secs / 60, secs % 60);
    let time_size = fit_size(&time, p.cell * 0.9, inner_w);
    let time_top = r.bottom() - pad - time_size - label - pad * 0.3;
    if time_top >= y {
        draw_text("TIME", x, time_top + label * 0.8, label, look.label(PINK));
        draw_text(&time, x, time_top + label + pad * 0.3 + time_size * 0.8, time_size, look.value());
    }
}

/// The open Duo's right column under Next: this game's tallies, and the last special clear's
/// callout over the top of them while it's up.
fn draw_feed(game: &Game, r: Rect, p: &PlayLayout, u: f32, look: Look, callout: &Callout) {
    let stats = game.scoring_system.stats();
    let pieces = game.scoring_system.pieces();
    let per_sec = if game.game_time > 1.0 { pieces as f64 / game.game_time } else { 0.0 };
    let mut rows: Vec<(&str, String, Color)> = vec![
        ("PIECES", pieces.to_string(), CYAN),
        ("PER SEC", format!("{per_sec:.2}"), CYAN),
        ("TETRISES", stats.tetrises.to_string(), GOLD),
        ("T-SPINS", stats.t_spins.to_string(), VIOLET),
        ("BEST COMBO", stats.best_combo.to_string(), GREEN),
        ("B2B BEST", stats.b2b_best.to_string(), PINK),
    ];
    if stats.perfect_clears > 0 {
        rows.push(("PERFECT", stats.perfect_clears.to_string(), GOLD));
    }

    let inner = panel(r, "THIS GAME", GREEN, p, u, look);
    let label = label_size(p, u);
    let row_h = (label * 1.75).min(inner.h / rows.len() as f32).max(label * 1.2);
    let mut y = inner.y;
    for (name, value, accent) in &rows {
        if y + row_h > inner.bottom() + 0.5 {
            break;
        }
        let baseline = y + row_h * 0.5 + label * 0.35;
        let value_size = label * 1.1;
        let vw = measure_text(value, None, value_size.max(1.0) as u16, 1.0).width;
        let name_size = fit_size(name, label * 0.9, inner.w - vw - label);
        draw_text(name, inner.x, baseline, name_size, look.label(*accent));
        draw_text(value, inner.right() - vw, baseline, value_size, look.value());
        y += row_h;
    }

    let a = callout.alpha();
    if a <= 0.0 {
        return;
    }
    // The callout: big words in a box over the top of the panel.
    let max_size = (p.cell * 1.0).max(14.0 * u);
    let w = r.w - 2.0 * (4.0 * u);
    let sizes: Vec<f32> = callout.lines.iter().map(|l| fit_size(l, max_size, w * 0.88)).collect();
    let h = sizes.iter().map(|s| s * 1.35).sum::<f32>() + max_size * 0.6;
    let boxr = Rect::new(r.x + 4.0 * u, r.y + 4.0 * u, w, h);
    let radius = 10.0 * u;
    if look == Look::Legacy {
        style::term_box(boxr, alpha(TERM_GREEN, a), 0.95 * a, u);
    } else {
        style::halo(boxr, radius, 10.0 * u, GOLD, 0.35 * a);
        style::round_rect(boxr, radius, Color::new(0.20, 0.12, 0.38, 0.95 * a), Color::new(0.08, 0.04, 0.20, 0.95 * a));
        style::round_outline(boxr, radius, (2.0 * u).max(1.0), alpha(GOLD, 0.9 * a));
    }
    const COLORS: [Color; 3] = [GOLD, CYAN, PINK];
    let mut baseline = boxr.y + max_size * 0.3;
    for (i, (line, size)) in callout.lines.iter().zip(&sizes).enumerate() {
        baseline += size * 1.35;
        let color = look.pick(COLORS[i % COLORS.len()], TERM_GREEN);
        centered(line, boxr.center().x, baseline - size * 0.3, *size, alpha(color, a));
    }
}

/// Counting back into play: the number, big, over the board.
fn draw_countdown(p: &PlayLayout, u: f32, left: f32, look: Look) {
    screen_camera();
    let b = p.board;
    draw_rectangle(b.x, b.y, b.w, b.h, look.pick(Color::new(0.05, 0.02, 0.16, 0.55), Color::new(0.0, 0.0, 0.0, 0.7)));
    let n = left.ceil().max(1.0) as u32;
    let text = n.to_string();
    // Each number pops in big and shrinks a little over its second.
    let t = left.fract();
    let size = p.cell * (4.0 + 1.5 * t);
    let color = look.pick([GREEN, GOLD, PINK][(n as usize).min(3) - 1], TERM_GREEN);
    centered(&text, b.center().x + 3.0 * u, b.center().y + size * 0.35 + 3.0 * u, size, Color::new(0.0, 0.0, 0.0, 0.45));
    centered(&text, b.center().x, b.center().y + size * 0.35, size, color);
    let ready = "GET READY";
    let rs = fit_size(ready, label_size(p, u) * 1.2, b.w * 0.8);
    centered(ready, b.center().x, b.center().y - size * 0.6, rs, look.label(CYAN));
}

fn centered(text: &str, cx: f32, baseline: f32, size: f32, color: Color) {
    let w = measure_text(text, None, size.max(1.0) as u16, 1.0).width;
    draw_text(text, cx - w / 2.0, baseline, size, color);
}

/// While aiming a ghost block: how to throw it, across the top of the board.
fn draw_aim_banner(p: &PlayLayout, u: f32, look: Look) {
    let b = p.board;
    let lines = ["TAP A CELL TO AIM", "TAP IT AGAIN TO THROW"];
    let w = b.w * 0.92;
    let size = fit_size(lines[1], (p.cell * 0.55).max(11.0 * u), w * 0.86);
    let h = size * 3.4;
    let r = Rect::new(b.x + (b.w - w) / 2.0, b.y + p.cell * 0.4, w, h);
    let radius = 10.0 * u;
    if look == Look::Legacy {
        style::term_box(r, TERM_GREEN, 0.9, u);
    } else {
        style::round_rect(r, radius, Color::new(0.13, 0.10, 0.34, 0.88), Color::new(0.06, 0.04, 0.18, 0.88));
        style::round_outline(r, radius, (1.5 * u).max(1.0), alpha(VIOLET, 0.85));
    }
    centered(lines[0], r.center().x, r.y + size * 1.35, size, look.pick(VIOLET, TERM_GREEN));
    centered(lines[1], r.center().x, r.y + size * 2.75, size, look.value());
}

/// The first game's reminder of the gestures, over the lower part of the board.
fn draw_hint(p: &PlayLayout, u: f32, alpha: f32, look: Look) {
    let rows: [(&str, &str); 4] = [("TAP", "rotate"), ("DRAG", "move"), ("FLICK DOWN", "drop"), ("FLICK UP", "hold")];
    let b = p.board;
    let w = b.w * 0.9;
    let size = fit_size("FLICK DOWN   drop", (p.cell * 0.55).max(11.0 * u), w * 0.8);
    let row_h = size * 1.6;
    let h = row_h * rows.len() as f32 + size;
    let r = Rect::new(b.x + (b.w - w) / 2.0, b.y + b.h * 0.58, w, h);
    let radius = 10.0 * u;
    if look == Look::Legacy {
        style::term_box(r, style::alpha(TERM_GREEN, alpha), 0.92 * alpha, u);
    } else {
        style::round_rect(r, radius, Color::new(0.13, 0.10, 0.34, 0.92 * alpha), Color::new(0.06, 0.04, 0.18, 0.92 * alpha));
        style::round_outline(r, radius, (1.5 * u).max(1.0), style::alpha(VIOLET, 0.8 * alpha));
    }
    gesture_rows(&rows, r.center().x, r.y + size * 0.5, row_h, size, alpha, look);
}

/// Gesture / action rows, aligned on a centre line: the gesture to its left, the action to its right.
fn gesture_rows(rows: &[(&str, &str)], cx: f32, top: f32, row_h: f32, size: f32, alpha: f32, look: Look) {
    let gap = size * 0.6;
    let mut label = look.label(CYAN);
    label.a *= alpha;
    let mut value = look.value();
    value.a *= alpha;
    for (i, (gesture, action)) in rows.iter().enumerate() {
        let baseline = top + i as f32 * row_h + row_h * 0.62;
        let gw = measure_text(gesture, None, size as u16, 1.0).width;
        draw_text(gesture, cx - gap / 2.0 - gw, baseline, size, label);
        draw_text(action, cx + gap / 2.0, baseline, size, value);
    }
}

/// Paused: the gestures, with Resume / Menu buttons below (drawn by input::touch).
fn draw_pause(layout: &Layout, look: Look) {
    screen_camera();
    let (s, u) = (layout.safe, layout.unit);
    let shade = look.pick(Color::new(0.05, 0.02, 0.16, 0.80), Color::new(0.0, 0.0, 0.0, 0.86));
    draw_rectangle(0.0, 0.0, layout.screen.w, layout.screen.h, shade);
    let title = fit_size("PAUSED", 46.0 * u, s.w * 0.8);
    title_text("PAUSED", s.center().x, s.y + s.h * 0.17 + title * 0.8, title, u, look);

    // Between the title and the buttons (which start at 80% of the height, minus a button).
    let top = s.y + s.h * 0.17 + title * 1.5;
    let bottom = s.y + s.h * 0.8 - 66.0 * u - 12.0 * u;
    let row_h = ((bottom - top) / GESTURES.len() as f32).min(34.0 * u);
    let size = fit_size("2-FINGER TAP   rotate back", row_h * 0.55, s.w * 0.85);
    let block_h = row_h * GESTURES.len() as f32;
    let block_top = top + ((bottom - top) - block_h) / 2.0;
    let card_w = (s.w * 0.9).min(measure_text("2-FINGER TAP   rotate back", None, size as u16, 1.0).width * 1.25 + 32.0 * u);
    if look == Look::Legacy {
        // Room for the borders above and below the rows.
        let card = Rect::new(s.center().x - card_w / 2.0, block_top - size * 1.4, card_w, block_h + size * 2.8);
        style::term_frame(card, "CONTROLS", size, TERM_GREEN, TERM_GREEN);
    } else {
        let card = Rect::new(s.center().x - card_w / 2.0, block_top - row_h * 0.35, card_w, block_h + row_h * 0.7);
        style::panel(card, VIOLET, u);
    }
    gesture_rows(&GESTURES, s.center().x, block_top, row_h, size, 1.0, look);
}

/// A title in the piece colours, one letter each, with a soft shadow; in the legacy look, green
/// with a blinking cursor after it.
fn title_text(text: &str, cx: f32, baseline: f32, size: f32, u: f32, look: Look) {
    if look == Look::Legacy {
        centered(text, cx, baseline, size, TERM_GREEN);
        if blink(2.0) {
            let w = measure_text(text, None, size.max(1.0) as u16, 1.0).width;
            draw_text("_", cx + w / 2.0, baseline, size, TERM_GREEN);
        }
        return;
    }
    const COLORS: [Color; 6] = [CYAN, GREEN, GOLD, PINK, VIOLET, Color::new(1.0, 0.55, 0.25, 1.0)];
    let w = measure_text(text, None, size.max(1.0) as u16, 1.0).width;
    let mut x = cx - w / 2.0;
    let mut k = 0;
    for ch in text.chars() {
        let s = ch.to_string();
        let cw = measure_text(&s, None, size.max(1.0) as u16, 1.0).width;
        if ch != ' ' {
            draw_text(&s, x + 2.0 * u, baseline + 2.0 * u, size, Color::new(0.0, 0.0, 0.0, 0.45));
            draw_text(&s, x, baseline, size, COLORS[k % COLORS.len()]);
            k += 1;
        }
        x += cw;
    }
}
