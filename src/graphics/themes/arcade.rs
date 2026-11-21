//! Arcade: a coin-op cabinet's screen. Black with scanlines and a neon grid running to a glowing
//! horizon, flat 8-bit blocks with hard highlights, double-ruled neon frames.

use macroquad::prelude::*;

use super::hash01;
use crate::graphics::style;
use crate::graphics::text::Face;
use crate::graphics::theme::{FxPalette, Theme, ThemeId};

const CYAN: Color = Color::new(0.0, 0.94, 1.0, 1.0);
const PINK: Color = Color::new(1.0, 0.17, 0.84, 1.0);
const GOLD: Color = Color::new(1.0, 0.9, 0.0, 1.0);
const GREEN: Color = Color::new(0.22, 1.0, 0.08, 1.0);
const VIOLET: Color = Color::new(0.62, 0.3, 1.0, 1.0);
const ORANGE: Color = Color::new(1.0, 0.5, 0.0, 1.0);
const RED: Color = Color::new(1.0, 0.18, 0.18, 1.0);
const NIGHT: Color = Color::new(0.02, 0.01, 0.05, 1.0);

pub const THEME: Theme = Theme {
    id: ThemeId::Arcade,
    name: "ARCADE",
    terminal: false,
    face: Face::PressStart,
    window: None,
    accents: [CYAN, PINK, GOLD, GREEN, VIOLET],
    label_ink: None,
    value: WHITE,
    hint: Color::new(1.0, 0.17, 0.84, 0.75),
    tip_on: Color::new(0.8, 0.65, 1.0, 1.0),
    tip_off: Color::new(0.8, 0.65, 1.0, 1.0),
    shade: Color::new(0.0, 0.0, 0.0, 0.85),
    shade_light: Color::new(0.0, 0.0, 0.0, 0.6),
    rainbow: [GOLD, PINK, CYAN, GREEN, VIOLET, ORANGE],
    surface: [Color::new(0.06, 0.02, 0.12, 1.0), NIGHT],
    pieces: Some([
        Color::new(0.0, 0.94, 1.0, 1.0),
        Color::new(1.0, 0.9, 0.0, 1.0),
        Color::new(1.0, 0.17, 0.84, 1.0),
        Color::new(0.22, 1.0, 0.08, 1.0),
        Color::new(1.0, 0.18, 0.18, 1.0),
        Color::new(0.25, 0.42, 1.0, 1.0),
        Color::new(1.0, 0.5, 0.0, 1.0),
        WHITE,
    ]),
    backdrop,
    panel,
    well,
    well_edge,
    block,
    ghost,
    fx: FxPalette {
        word: [GOLD, PINK, CYAN],
        hot: [GOLD, ORANGE, RED],
        edge: Color::new(0.3, 0.0, 0.4, 1.0),
        outline: BLACK,
        flash: WHITE,
        rings: [CYAN, PINK],
        dim: BLACK,
        banner: PINK,
        banner_ink: BLACK,
        stamp: GOLD,
        heat: ORANGE,
        dust: CYAN,
        tspin: VIOLET,
        combo: [CYAN, GOLD, PINK],
        perfect: GOLD,
        card_edge: CYAN,
        grey: Color::new(0.25, 0.25, 0.3, 1.0),
        card_ink: WHITE,
        card_label: GOLD,
        chip: BLACK,
        block,
        panel,
    },
};

fn backdrop(time: f32) {
    let (w, h) = (screen_width(), screen_height());
    clear_background(NIGHT);
    // A glow along the horizon, and a neon grid running towards it.
    let horizon = h * 0.62;
    style::gradient_rect(Rect::new(0.0, horizon - h * 0.12, w, h * 0.12), style::alpha(PINK, 0.0), style::alpha(PINK, 0.18));
    style::gradient_rect(Rect::new(0.0, horizon, w, h - horizon), style::alpha(VIOLET, 0.16), style::alpha(VIOLET, 0.04));
    let grid = style::alpha(PINK, 0.22);
    let scroll = (time * 0.25).fract();
    for i in 0..14 {
        let k = (i as f32 + scroll) / 14.0;
        let y = horizon + (h - horizon) * k * k;
        draw_rectangle(0.0, y, w, 1.0, grid);
    }
    for i in -10..=10 {
        let x = w / 2.0 + i as f32 * w / 10.0;
        draw_line(w / 2.0 + i as f32 * w / 40.0, horizon, x * 1.0 + (x - w / 2.0) * 1.5, h, 1.0, grid);
    }
    // Stars above the horizon.
    for i in 0..40u32 {
        let x = hash01(i, 1) * w;
        let y = hash01(i, 2) * horizon * 0.9;
        let tw = 0.4 + 0.6 * ((time * 2.0 + i as f32).sin() * 0.5 + 0.5);
        draw_rectangle(x, y, 2.0, 2.0, Color::new(1.0, 1.0, 1.0, 0.35 * tw));
    }
    // Scanlines over it all.
    let mut y = 0.0;
    while y < h {
        draw_rectangle(0.0, y, w, 1.0, Color::new(0.0, 0.0, 0.0, 0.28));
        y += 3.0;
    }
}

/// A double-ruled neon frame with square corners and corner pips.
fn panel(r: Rect, accent: Color, u: f32) {
    draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.0, 0.0, 0.0, 0.88));
    style::halo(r, 2.0 * u, 6.0 * u, accent, 0.12);
    let t = (2.0 * u).round().max(1.0);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, t * 2.0, accent);
    let i = 4.0 * u;
    draw_rectangle_lines(r.x + i, r.y + i, r.w - 2.0 * i, r.h - 2.0 * i, (u).max(1.0) * 2.0, style::alpha(accent, 0.45));
    let pip = 3.0 * u;
    for (x, y) in [(r.x, r.y), (r.right() - pip, r.y), (r.x, r.bottom() - pip), (r.right() - pip, r.bottom() - pip)] {
        draw_rectangle(x, y, pip, pip, WHITE);
    }
}

fn well(well: Rect, board: Rect, cell: f32, _u: f32) {
    draw_rectangle(well.x, well.y, well.w, well.h, Color::new(0.0, 0.0, 0.0, 0.92));
    // A dot at each grid corner.
    let cols = (board.w / cell).round() as usize;
    let rows = (board.h / cell).round() as usize;
    for y in 1..rows {
        for x in 1..cols {
            draw_rectangle(board.x + x as f32 * cell - 0.5, board.y + y as f32 * cell - 0.5, 1.5, 1.5, style::alpha(CYAN, 0.25));
        }
    }
}

fn well_edge(well: Rect, u: f32) {
    style::halo(well, 2.0 * u, 10.0 * u, PINK, 0.25);
    draw_rectangle_lines(well.x, well.y, well.w, well.h, (3.0 * u).round().max(2.0) * 2.0, PINK);
}

/// A flat 8-bit block: the colour, a hard light edge top and left, a dark one bottom and right,
/// and a glint in the corner.
fn block(x: f32, y: f32, size: f32, color: Color) {
    let gap = (size * 0.06).max(1.0).round();
    let r = Rect::new(x + gap, y + gap, size - 2.0 * gap, size - 2.0 * gap);
    let a = color.a;
    draw_rectangle(r.x, r.y, r.w, r.h, color);
    let e = (size * 0.12).max(1.0).round();
    super::bevel(r, e, style::mix(color, WHITE, 0.5).with_alpha(a), style::mix(color, BLACK, 0.5).with_alpha(a), true);
    draw_rectangle(r.x + e, r.y + e, e, e, Color::new(1.0, 1.0, 1.0, 0.85 * a));
    // A hard black outline, like sprite art.
    let o = (size * 0.06).max(1.0).round();
    let ink = Color::new(0.0, 0.0, 0.0, 0.9 * a);
    draw_rectangle(r.x - o, r.y - o, r.w + 2.0 * o, o, ink);
    draw_rectangle(r.x - o, r.bottom(), r.w + 2.0 * o, o, ink);
    draw_rectangle(r.x - o, r.y, o, r.h, ink);
    draw_rectangle(r.right(), r.y, o, r.h, ink);
}

/// Where the piece lands: corner brackets in its colour.
fn ghost(x: f32, y: f32, size: f32, color: Color) {
    let t = (size * 0.08).max(1.0);
    let l = size * 0.3;
    let c = style::alpha(color, 0.75);
    let (x0, y0, x1, y1) = (x + t, y + t, x + size - t, y + size - t);
    for (cx, cy, dx, dy) in [(x0, y0, 1.0, 1.0), (x1, y0, -1.0, 1.0), (x0, y1, 1.0, -1.0), (x1, y1, -1.0, -1.0)] {
        draw_line(cx, cy, cx + dx * l, cy, t, c);
        draw_line(cx, cy, cx, cy + dy * l, t, c);
    }
}
