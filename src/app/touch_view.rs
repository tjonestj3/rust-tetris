//! The game screen on touch devices: the board as big as the screen allows, Next / Hold / score
//! in compact panels beside it or above it (graphics::view::PlayLayout), the backdrop over the
//! whole screen.
//!
//! The board, pieces and effects are the desktop drawing code, drawn through a camera that maps
//! the canvas's board onto the layout's board rect. The panels, pause screen and gesture hint are
//! drawn here in screen pixels.

use macroquad::prelude::*;

use super::{
    draw_enhanced_board_with_data, draw_falling_piece, draw_ghost_block_cursor, draw_ghost_piece,
    draw_ghost_throw_animation, draw_legacy_board_with_data, draw_legacy_falling_piece,
    draw_legacy_ghost_piece, draw_line_clear_animation, draw_tetris_celebration,
};
use crate::game::{Game, GameState};
use crate::graphics::text::{draw_text, measure_text};
use crate::graphics::view::{draw_cover_background, map_camera, screen_camera, HudStyle, Layout, PlayLayout, CANVAS_BOARD};
use crate::tetromino::{Tetromino, TetrominoType};

const PANEL_FILL: Color = Color::new(0.02, 0.03, 0.10, 0.82);
const PANEL_EDGE: Color = Color::new(0.30, 0.85, 1.00, 0.55);
const LABEL: Color = Color::new(0.45, 0.85, 1.00, 0.95);
const VALUE: Color = Color::new(1.00, 1.00, 1.00, 0.97);
const ACCENT: Color = Color::new(1.00, 0.88, 0.25, 1.00);

/// The gestures, for the pause screen and the first-game hint: (gesture, what it does).
const GESTURES: [(&str, &str); 6] = [
    ("TAP", "rotate"),
    ("2-FINGER TAP", "rotate back"),
    ("DRAG", "move"),
    ("DRAG DOWN", "soft drop"),
    ("FLICK DOWN", "hard drop"),
    ("FLICK UP", "hold"),
];

/// Draw the game screen. `hint` (0..1) fades the first-game gesture hint in and out.
pub(super) fn render(game: &Game, background: &Texture2D, layout: &Layout, hint: f32) {
    let p = &layout.play;
    let legacy = game.is_legacy_mode();

    screen_camera();
    if legacy {
        clear_background(BLACK);
    } else {
        draw_cover_background(background, 0.45);
    }

    // The board and everything on it, through the board camera.
    map_camera(CANVAS_BOARD, p.board);
    if legacy {
        draw_legacy_board_with_data(&game.board);
    } else {
        draw_enhanced_board_with_data(&game.board);
    }
    if game.is_clearing_lines() {
        draw_line_clear_animation(game);
    } else {
        if let Some(ghost) = game.calculate_ghost_piece() {
            if legacy { draw_legacy_ghost_piece(&ghost) } else { draw_ghost_piece(&ghost) }
        }
        if let Some(ref piece) = game.current_piece {
            if legacy { draw_legacy_falling_piece(piece) } else { draw_falling_piece(piece) }
        }
    }
    if game.is_ghost_cursor_visible() {
        draw_ghost_block_cursor(game);
    }
    if game.is_ghost_throw_active() {
        draw_ghost_throw_animation(game);
    }

    // The panels.
    screen_camera();
    let u = layout.unit;
    let next = panel(p.next, "NEXT", p, u);
    draw_piece(next, Some(game.next_piece), p.cell, 1.0);
    let hold = panel(p.hold, "HOLD", p, u);
    draw_piece(hold, game.held_piece, p.cell, if game.can_hold() { 1.0 } else { 0.35 });
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

/// A panel's frame and label; returns the area left for its content.
fn panel(r: Rect, label: &str, p: &PlayLayout, u: f32) -> Rect {
    frame(r, u);
    let pad = (p.cell * 0.3).max(6.0 * u);
    let size = label_size(p, u);
    draw_text(label, r.x + pad, r.y + pad + size * 0.8, size, LABEL);
    let top = r.y + pad + size + pad * 0.4;
    Rect::new(r.x + pad, top, r.w - 2.0 * pad, r.bottom() - pad - top)
}

fn frame(r: Rect, u: f32) {
    draw_rectangle(r.x, r.y, r.w, r.h, PANEL_FILL);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, (1.5 * u).max(1.0), PANEL_EDGE);
}

fn label_size(p: &PlayLayout, u: f32) -> f32 {
    (p.cell * 0.5).clamp(10.0 * u, 17.0 * u)
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
        draw_block(x0 + (dx - min_x) as f32 * b, y0 + (dy - min_y) as f32 * b, b, color);
    }
}

/// One block, styled like the board's (a highlight on top, a shade at the bottom).
fn draw_block(x: f32, y: f32, size: f32, color: Color) {
    let inset = (size * 0.04).max(1.0);
    let s = size - 2.0 * inset;
    draw_rectangle(x + inset, y + inset, s, s, color);
    draw_rectangle(x + inset * 2.0, y + inset * 2.0, s - 2.0 * inset, s * 0.18, Color::new(1.0, 1.0, 1.0, 0.3 * color.a));
    draw_rectangle(x + inset * 2.0, y + inset + s * 0.84, s - 2.0 * inset, s * 0.12, Color::new(0.0, 0.0, 0.0, 0.2 * color.a));
}

/// Text sized to fit `max_w`, no bigger than `size`.
fn fit_size(text: &str, size: f32, max_w: f32) -> f32 {
    let w = measure_text(text, None, size.max(1.0) as u16, 1.0).width;
    if w > max_w && w > 0.0 { size * max_w / w } else { size }
}

fn draw_stats(game: &Game, p: &PlayLayout, u: f32) {
    let r = p.stats;
    frame(r, u);
    let pad = (p.cell * 0.3).max(6.0 * u);
    let label = label_size(p, u);
    let inner_w = r.w - 2.0 * pad;
    let score = game.score.to_string();
    let level = game.level().to_string();
    let lines = game.lines_cleared().to_string();

    if p.style == HudStyle::TopStrip {
        // Score big in the middle, level and lines under it.
        let cx = r.center().x;
        let score_size = fit_size(&score, (r.h * 0.42).min(p.cell * 1.3), inner_w);
        let sub = format!("LEVEL {level}   LINES {lines}");
        let sub_size = fit_size(&sub, label, inner_w);
        let total = label + score_size + sub_size + pad * 0.6;
        let mut y = r.y + (r.h - total) / 2.0;
        centered("SCORE", cx, y + label * 0.8, label, LABEL);
        y += label + pad * 0.2;
        centered(&score, cx, y + score_size * 0.8, score_size, VALUE);
        y += score_size + pad * 0.4;
        centered(&sub, cx, y + sub_size * 0.8, sub_size, ACCENT);
        return;
    }

    // A column: SCORE over a big number, then LEVEL and LINES side by side.
    let x = r.x + pad;
    let mut y = r.y + pad;
    draw_text("SCORE", x, y + label * 0.8, label, LABEL);
    y += label + pad * 0.3;
    let score_size = fit_size(&score, p.cell * 1.15, inner_w);
    draw_text(&score, x, y + score_size * 0.8, score_size, VALUE);
    y += score_size + pad * 0.9;

    let half = inner_w / 2.0;
    let widest = if level.len() >= lines.len() { &level } else { &lines };
    let value_size = fit_size(widest, p.cell * 0.85, half - pad * 0.5);
    if y + label + value_size <= r.bottom() - pad * 0.5 {
        draw_text("LEVEL", x, y + label * 0.8, label, LABEL);
        draw_text("LINES", x + half, y + label * 0.8, label, LABEL);
        y += label + pad * 0.3;
        draw_text(&level, x, y + value_size * 0.8, value_size, ACCENT);
        draw_text(&lines, x + half, y + value_size * 0.8, value_size, ACCENT);
        y += value_size + pad * 0.6;
    }
    if game.ghost_blocks_available > 0 && y + label <= r.bottom() - pad * 0.3 {
        let text = format!("GHOST BLOCKS {}", game.ghost_blocks_available);
        draw_text(&text, x, y + label * 0.8, fit_size(&text, label, inner_w), LABEL);
        y += label + pad * 0.6;
    }

    // Time, at the bottom of the panel when it's tall enough.
    let secs = game.game_time.max(0.0) as u64;
    let time = format!("{}:{:02}", secs / 60, secs % 60);
    let time_size = fit_size(&time, p.cell * 0.7, inner_w);
    let time_top = r.bottom() - pad - time_size - label - pad * 0.3;
    if time_top >= y {
        draw_text("TIME", x, time_top + label * 0.8, label, LABEL);
        draw_text(&time, x, time_top + label + pad * 0.3 + time_size * 0.8, time_size, VALUE);
    }
}

fn centered(text: &str, cx: f32, baseline: f32, size: f32, color: Color) {
    let w = measure_text(text, None, size.max(1.0) as u16, 1.0).width;
    draw_text(text, cx - w / 2.0, baseline, size, color);
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
    let mut fill = PANEL_FILL;
    fill.a *= alpha;
    let mut edge = PANEL_EDGE;
    edge.a *= alpha;
    draw_rectangle(r.x, r.y, r.w, r.h, fill);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, (1.5 * u).max(1.0), edge);
    gesture_rows(&rows, r.center().x, r.y + size * 0.5, row_h, size, alpha);
}

/// Gesture / action rows, aligned on a centre line: the gesture to its left, the action to its right.
fn gesture_rows(rows: &[(&str, &str)], cx: f32, top: f32, row_h: f32, size: f32, alpha: f32) {
    let gap = size * 0.6;
    let mut label = LABEL;
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
    draw_rectangle(0.0, 0.0, layout.screen.w, layout.screen.h, Color::new(0.0, 0.0, 0.03, 0.78));
    let title = fit_size("PAUSED", 46.0 * u, s.w * 0.8);
    centered("PAUSED", s.center().x, s.y + s.h * 0.17 + title * 0.8, title, VALUE);

    // Between the title and the buttons (which start at 80% of the height, minus a button).
    let top = s.y + s.h * 0.17 + title * 1.5;
    let bottom = s.y + s.h * 0.8 - 66.0 * u - 12.0 * u;
    let row_h = ((bottom - top) / GESTURES.len() as f32).min(34.0 * u);
    let size = fit_size("2-FINGER TAP   rotate back", row_h * 0.55, s.w * 0.85);
    let block_h = row_h * GESTURES.len() as f32;
    gesture_rows(&GESTURES, s.center().x, top + ((bottom - top) - block_h) / 2.0, row_h, size, 1.0);
}

fn draw_game_over(game: &Game, layout: &Layout) {
    screen_camera();
    let (s, u) = (layout.safe, layout.unit);
    draw_rectangle(0.0, 0.0, layout.screen.w, layout.screen.h, Color::new(0.0, 0.0, 0.03, 0.75));
    let size = fit_size("GAME OVER", 52.0 * u, s.w * 0.85);
    centered("GAME OVER", s.center().x, s.center().y - size * 0.2, size, Color::new(1.0, 0.3, 0.3, 1.0));
    let score = format!("SCORE {}", game.score);
    let small = fit_size(&score, 22.0 * u, s.w * 0.8);
    centered(&score, s.center().x, s.center().y + size * 0.9, small, VALUE);
}
