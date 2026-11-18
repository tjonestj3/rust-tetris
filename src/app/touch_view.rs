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
use crate::graphics::style::{self, alpha, CYAN, GOLD, GREEN, INK, PINK, SOFT_INK, VIOLET};
use crate::graphics::text::{draw_text, measure_text};
use crate::graphics::view::{map_camera, screen_camera, HudStyle, Layout, PlayLayout, CANVAS_BOARD};
use crate::tetromino::{Tetromino, TetrominoType};

const LABEL: Color = SOFT_INK;
const VALUE: Color = INK;

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

/// Draw the game screen. `hint` (0..1) fades the first-game gesture hint in and out.
pub(super) fn render(game: &Game, background: &Texture2D, layout: &Layout, hint: f32) {
    let p = &layout.play;
    let legacy = game.is_legacy_mode();

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
        draw_board(game, p, layout.unit);
        if game.is_clearing_lines() {
            map_camera(CANVAS_BOARD, p.board);
            draw_line_clear_animation(game);
        }
        map_camera(CANVAS_BOARD, p.board);
    }
    if game.is_ghost_cursor_visible() {
        draw_ghost_block_cursor(game);
    }
    if game.is_ghost_throw_active() {
        draw_ghost_throw_animation(game);
    }

    // The panels.
    screen_camera();
    if game.ghost_block_placement_mode {
        draw_aim_banner(p, layout.unit);
    }
    let u = layout.unit;
    let next = panel(p.next, "NEXT", CYAN, p, u);
    draw_piece(next, Some(game.next_piece), p.cell, 1.0);
    let hold = panel(p.hold, "HOLD", PINK, p, u);
    draw_piece(hold, game.held_piece, p.cell, if game.can_hold() { 1.0 } else { 0.35 });
    if game.held_piece.is_none() {
        // The panel is a button (input::touch): say so until something's in it.
        let size = fit_size("TAP TO HOLD", label_size(p, u) * 0.9, hold.w);
        let w = measure_text("TAP TO HOLD", None, size.max(1.0) as u16, 1.0).width;
        draw_text("TAP TO HOLD", hold.center().x - w / 2.0, hold.center().y + size * 0.35, size, alpha(PINK, 0.7));
    }
    draw_stats(game, p, u);

    if hint > 0.0 && game.state == GameState::Playing {
        draw_hint(p, u, hint);
    }

    // The TETRIS celebration, over everything, at the size of the whole canvas fitted to the screen.
    if game.is_tetris_celebration_active() {
        map_camera(layout.canvas_src, layout.game);
        draw_tetris_celebration(game);
        screen_camera();
    }

    match game.state {
        GameState::Paused => draw_pause(layout),
        GameState::GameOver => draw_game_over(game, layout),
        _ => {}
    }
    screen_camera();
}

/// The board: a dark glass well with a glowing edge, the settled blocks, where the piece will
/// land, and the piece.
fn draw_board(game: &Game, p: &PlayLayout, u: f32) {
    let b = p.board;
    let cell = b.w / BOARD_WIDTH as f32;
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

    for y in 0..VISIBLE_HEIGHT {
        for x in 0..BOARD_WIDTH {
            if let Some(color) = game.board.get_cell(x as i32, (y + BUFFER_HEIGHT) as i32).and_then(|c| c.color()) {
                style::block(b.x + x as f32 * cell, b.y + y as f32 * cell, cell, color);
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
}

/// A panel's frame and label; returns the area left for its content.
fn panel(r: Rect, label: &str, accent: Color, p: &PlayLayout, u: f32) -> Rect {
    style::panel(r, accent, u);
    let pad = (p.cell * 0.3).max(7.0 * u);
    let size = label_size(p, u);
    draw_text(label, r.x + pad, r.y + pad + size * 0.8, size, accent);
    let top = r.y + pad + size + pad * 0.4;
    Rect::new(r.x + pad, top, r.w - 2.0 * pad, r.bottom() - pad - top)
}

fn label_size(p: &PlayLayout, u: f32) -> f32 {
    (p.cell * 0.58).clamp(11.0 * u, 19.0 * u)
}

/// A piece centred in `area`, at most 0.8 of a board cell per block.
fn draw_piece(area: Rect, piece: Option<TetrominoType>, cell: f32, alpha: f32) {
    let Some(kind) = piece else { return };
    let blocks = Tetromino::new(kind).blocks;
    let (min_x, max_x) = (blocks.iter().map(|b| b.0).min().unwrap_or(0), blocks.iter().map(|b| b.0).max().unwrap_or(0));
    let (min_y, max_y) = (blocks.iter().map(|b| b.1).min().unwrap_or(0), blocks.iter().map(|b| b.1).max().unwrap_or(0));
    let (cols, rows) = ((max_x - min_x + 1) as f32, (max_y - min_y + 1) as f32);
    let b = (area.w / 4.4).min(area.h / 2.4).min(cell * 0.8).floor().max(2.0);
    let x0 = (area.x + (area.w - cols * b) / 2.0).round();
    let y0 = (area.y + (area.h - rows * b) / 2.0).round();
    let mut color = kind.color();
    color.a *= alpha;
    for (dx, dy) in &blocks {
        style::block(x0 + (dx - min_x) as f32 * b, y0 + (dy - min_y) as f32 * b, b, color);
    }
}

/// Text sized to fit `max_w`, no bigger than `size`.
fn fit_size(text: &str, size: f32, max_w: f32) -> f32 {
    let w = measure_text(text, None, size.max(1.0) as u16, 1.0).width;
    if w > max_w && w > 0.0 { size * max_w / w } else { size }
}

fn draw_stats(game: &Game, p: &PlayLayout, u: f32) {
    let r = p.stats;
    style::panel(r, GOLD, u);
    let ghosts = game.ghost_blocks_available;
    if ghosts > 0 {
        // The panel is a button (input::touch) while there's a ghost block to use: a violet
        // glow that breathes, brighter while aiming.
        let aiming = game.ghost_block_placement_mode;
        let pulse = (get_time() as f32 * 3.0).sin() * 0.5 + 0.5;
        let strength = if aiming { 0.45 + 0.30 * pulse } else { 0.25 + 0.20 * pulse };
        style::halo(r, 10.0 * u, 10.0 * u, VIOLET, strength);
        style::round_outline(r, 10.0 * u, (2.0 * u).max(1.5), alpha(VIOLET, 0.75 + 0.25 * pulse));
    }
    let pad = (p.cell * 0.3).max(7.0 * u);
    let label = label_size(p, u);
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
        centered("SCORE", cx, y + label * 0.8, label, GOLD);
        y += label + pad * 0.2;
        centered(&score, cx, y + score_size * 0.8, score_size, VALUE);
        y += score_size + pad * 0.4;
        centered(&sub, cx, y + sub_size * 0.8, sub_size, GREEN);
        return;
    }

    // A column: SCORE over a big number, then LEVEL and LINES side by side.
    let x = r.x + pad;
    let mut y = r.y + pad;
    draw_text("SCORE", x, y + label * 0.8, label, GOLD);
    y += label + pad * 0.3;
    let score_size = fit_size(&score, p.cell * 1.55, inner_w);
    draw_text(&score, x, y + score_size * 0.8, score_size, VALUE);
    y += score_size + pad * 0.9;

    let half = inner_w / 2.0;
    let widest = if level.len() >= lines.len() { &level } else { &lines };
    let value_size = fit_size(widest, p.cell * 1.05, half - pad * 0.5);
    if y + label + value_size <= r.bottom() - pad * 0.5 {
        draw_text("LEVEL", x, y + label * 0.8, label, GREEN);
        draw_text("LINES", x + half, y + label * 0.8, label, CYAN);
        y += label + pad * 0.3;
        draw_text(&level, x, y + value_size * 0.8, value_size, VALUE);
        draw_text(&lines, x + half, y + value_size * 0.8, value_size, VALUE);
        y += value_size + pad * 0.6;

        // Progress to the next level.
        let bar_h = (p.cell * 0.28).max(5.0 * u).round();
        if y + bar_h <= r.bottom() - pad * 0.5 {
            let done = (game.lines_cleared() % LINES_PER_LEVEL) as f32 / LINES_PER_LEVEL as f32;
            let bar = Rect::new(x, y, inner_w, bar_h);
            style::round_rect(bar, bar_h / 2.0, Color::new(1.0, 1.0, 1.0, 0.10), Color::new(1.0, 1.0, 1.0, 0.06));
            if done > 0.0 {
                let fill = Rect::new(x, y, (inner_w * done).max(bar_h), bar_h);
                style::round_rect(fill, bar_h / 2.0, style::mix(GREEN, WHITE, 0.25), CYAN);
            }
            y += bar_h + pad * 0.8;
        }
    }
    if ghosts > 0 && y + label <= r.bottom() - pad * 0.3 {
        let text = format!("GHOST BLOCKS {ghosts}");
        draw_text(&text, x, y + label * 0.8, fit_size(&text, label, inner_w), VIOLET);
        y += label + pad * 0.3;
        let tip = if game.ghost_block_placement_mode { "TAP TO THROW" } else { "TAP TO AIM" };
        if y + label <= r.bottom() - pad * 0.3 {
            draw_text(tip, x, y + label * 0.8, fit_size(tip, label * 0.85, inner_w), style::mix(VIOLET, WHITE, 0.45));
            y += label + pad * 0.6;
        }
    }

    // Time, at the bottom of the panel when it's tall enough.
    let secs = game.game_time.max(0.0) as u64;
    let time = format!("{}:{:02}", secs / 60, secs % 60);
    let time_size = fit_size(&time, p.cell * 0.9, inner_w);
    let time_top = r.bottom() - pad - time_size - label - pad * 0.3;
    if time_top >= y {
        draw_text("TIME", x, time_top + label * 0.8, label, PINK);
        draw_text(&time, x, time_top + label + pad * 0.3 + time_size * 0.8, time_size, VALUE);
    }
}

fn centered(text: &str, cx: f32, baseline: f32, size: f32, color: Color) {
    let w = measure_text(text, None, size.max(1.0) as u16, 1.0).width;
    draw_text(text, cx - w / 2.0, baseline, size, color);
}

/// While aiming a ghost block: how to throw it, across the top of the board.
fn draw_aim_banner(p: &PlayLayout, u: f32) {
    let b = p.board;
    let lines = ["TAP A CELL TO AIM", "TAP IT AGAIN TO THROW"];
    let w = b.w * 0.92;
    let size = fit_size(lines[1], (p.cell * 0.55).max(11.0 * u), w * 0.86);
    let h = size * 3.4;
    let r = Rect::new(b.x + (b.w - w) / 2.0, b.y + p.cell * 0.4, w, h);
    let radius = 10.0 * u;
    style::round_rect(r, radius, Color::new(0.13, 0.10, 0.34, 0.88), Color::new(0.06, 0.04, 0.18, 0.88));
    style::round_outline(r, radius, (1.5 * u).max(1.0), alpha(VIOLET, 0.85));
    centered(lines[0], r.center().x, r.y + size * 1.35, size, VIOLET);
    centered(lines[1], r.center().x, r.y + size * 2.75, size, VALUE);
}

/// The first game's reminder of the gestures, over the lower part of the board.
fn draw_hint(p: &PlayLayout, u: f32, alpha: f32) {
    let rows: [(&str, &str); 4] = [("TAP", "rotate"), ("DRAG", "move"), ("FLICK DOWN", "drop"), ("FLICK UP", "hold")];
    let b = p.board;
    let w = b.w * 0.9;
    let size = fit_size("FLICK DOWN   drop", (p.cell * 0.55).max(11.0 * u), w * 0.8);
    let row_h = size * 1.6;
    let h = row_h * rows.len() as f32 + size;
    let r = Rect::new(b.x + (b.w - w) / 2.0, b.y + b.h * 0.58, w, h);
    let radius = 10.0 * u;
    style::round_rect(r, radius, Color::new(0.13, 0.10, 0.34, 0.92 * alpha), Color::new(0.06, 0.04, 0.18, 0.92 * alpha));
    style::round_outline(r, radius, (1.5 * u).max(1.0), style::alpha(VIOLET, 0.8 * alpha));
    gesture_rows(&rows, r.center().x, r.y + size * 0.5, row_h, size, alpha);
}

/// Gesture / action rows, aligned on a centre line: the gesture to its left, the action to its right.
fn gesture_rows(rows: &[(&str, &str)], cx: f32, top: f32, row_h: f32, size: f32, alpha: f32) {
    let gap = size * 0.6;
    let mut label = CYAN;
    label.a *= alpha;
    let mut value = VALUE;
    value.a *= alpha;
    for (i, (gesture, action)) in rows.iter().enumerate() {
        let baseline = top + i as f32 * row_h + row_h * 0.62;
        let gw = measure_text(gesture, None, size as u16, 1.0).width;
        draw_text(gesture, cx - gap / 2.0 - gw, baseline, size, label);
        draw_text(action, cx + gap / 2.0, baseline, size, value);
    }
}

/// Paused: the gestures, with Resume / Menu buttons below (drawn by input::touch).
fn draw_pause(layout: &Layout) {
    screen_camera();
    let (s, u) = (layout.safe, layout.unit);
    draw_rectangle(0.0, 0.0, layout.screen.w, layout.screen.h, Color::new(0.05, 0.02, 0.16, 0.80));
    let title = fit_size("PAUSED", 46.0 * u, s.w * 0.8);
    title_text("PAUSED", s.center().x, s.y + s.h * 0.17 + title * 0.8, title, u);

    // Between the title and the buttons (which start at 80% of the height, minus a button).
    let top = s.y + s.h * 0.17 + title * 1.5;
    let bottom = s.y + s.h * 0.8 - 66.0 * u - 12.0 * u;
    let row_h = ((bottom - top) / GESTURES.len() as f32).min(34.0 * u);
    let size = fit_size("2-FINGER TAP   rotate back", row_h * 0.55, s.w * 0.85);
    let block_h = row_h * GESTURES.len() as f32;
    let block_top = top + ((bottom - top) - block_h) / 2.0;
    let card_w = (s.w * 0.9).min(measure_text("2-FINGER TAP   rotate back", None, size as u16, 1.0).width * 1.25 + 32.0 * u);
    let card = Rect::new(s.center().x - card_w / 2.0, block_top - row_h * 0.35, card_w, block_h + row_h * 0.7);
    style::panel(card, VIOLET, u);
    gesture_rows(&GESTURES, s.center().x, block_top, row_h, size, 1.0);
}

fn draw_game_over(game: &Game, layout: &Layout) {
    screen_camera();
    let (s, u) = (layout.safe, layout.unit);
    draw_rectangle(0.0, 0.0, layout.screen.w, layout.screen.h, Color::new(0.05, 0.02, 0.16, 0.78));
    let size = fit_size("GAME OVER", 52.0 * u, s.w * 0.85);
    let card_w = (s.w * 0.86).min(size * 7.0);
    let card = Rect::new(s.center().x - card_w / 2.0, s.center().y - size * 1.6, card_w, size * 3.4);
    style::panel(card, PINK, u);
    title_text("GAME OVER", s.center().x, card.y + size * 1.35, size, u);
    centered("SCORE", s.center().x, card.y + size * 2.15, fit_size("SCORE", 16.0 * u, card_w), GOLD);
    let score = game.score.to_string();
    let big = fit_size(&score, 30.0 * u, card_w * 0.8);
    centered(&score, s.center().x, card.y + size * 2.15 + big * 1.1, big, VALUE);
}

/// A title in the piece colours, one letter each, with a soft shadow.
fn title_text(text: &str, cx: f32, baseline: f32, size: f32, u: f32) {
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
