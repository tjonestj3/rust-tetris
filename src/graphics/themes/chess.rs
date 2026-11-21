//! Chess: a wooden chessboard. Walnut behind everything, the board checkered in two woods, blocks
//! as lacquered wooden tiles, brass and ivory for the frames and text.

use macroquad::prelude::*;

use super::{bevel, hash01};
use crate::graphics::style;
use crate::graphics::theme::{FxPalette, Theme, ThemeId};

const WALNUT: Color = Color::new(0.16, 0.10, 0.06, 1.0);
const DARK_WOOD: Color = Color::new(0.23, 0.14, 0.09, 1.0);
const EBONY: Color = Color::new(0.1, 0.07, 0.05, 1.0);
const BRASS: Color = Color::new(0.79, 0.64, 0.35, 1.0);
const IVORY: Color = Color::new(0.95, 0.91, 0.81, 1.0);
const OXBLOOD: Color = Color::new(0.66, 0.28, 0.2, 1.0);
const MOSS: Color = Color::new(0.52, 0.64, 0.36, 1.0);
const HEATHER: Color = Color::new(0.64, 0.52, 0.72, 1.0);
const MAPLE: Color = Color::new(0.83, 0.69, 0.49, 1.0);
const LIGHT_SQUARE: Color = Color::new(0.55, 0.40, 0.26, 1.0);
const DARK_SQUARE: Color = Color::new(0.45, 0.31, 0.19, 1.0);
const EMBER: Color = Color::new(0.9, 0.5, 0.2, 1.0);

pub const THEME: Theme = Theme {
    id: ThemeId::Chess,
    name: "CHESS",
    terminal: false,
    accents: [IVORY, OXBLOOD, BRASS, MOSS, HEATHER],
    label_ink: None,
    value: IVORY,
    hint: Color::new(0.79, 0.64, 0.35, 0.75),
    tip_on: BRASS,
    tip_off: Color::new(0.5, 0.4, 0.25, 1.0),
    shade: Color::new(0.1, 0.06, 0.03, 0.88),
    shade_light: Color::new(0.1, 0.06, 0.03, 0.7),
    rainbow: [IVORY, BRASS, OXBLOOD, MOSS, HEATHER, MAPLE],
    surface: [DARK_WOOD, WALNUT],
    pieces: Some([
        Color::new(0.93, 0.88, 0.76, 1.0),
        BRASS,
        OXBLOOD,
        Color::new(0.45, 0.55, 0.3, 1.0),
        Color::new(0.55, 0.2, 0.15, 1.0),
        Color::new(0.22, 0.16, 0.12, 1.0),
        Color::new(0.6, 0.4, 0.22, 1.0),
        MAPLE,
    ]),
    backdrop,
    panel,
    well,
    well_edge,
    block,
    ghost,
    fx: FxPalette {
        word: [IVORY, BRASS, OXBLOOD],
        hot: [BRASS, EMBER, OXBLOOD],
        edge: EBONY,
        outline: EBONY,
        flash: IVORY,
        rings: [BRASS, IVORY],
        dim: EBONY,
        banner: BRASS,
        banner_ink: EBONY,
        stamp: BRASS,
        heat: EMBER,
        dust: MAPLE,
        tspin: OXBLOOD,
        combo: [IVORY, BRASS, OXBLOOD],
        perfect: BRASS,
        card_edge: BRASS,
        grey: Color::new(0.35, 0.3, 0.25, 1.0),
        card_ink: IVORY,
        card_label: BRASS,
        chip: DARK_WOOD,
        block,
        panel,
    },
};

fn backdrop(_time: f32) {
    let (w, h) = (screen_width(), screen_height());
    style::gradient_rect(Rect::new(0.0, 0.0, w, h), WALNUT, Color::new(0.09, 0.055, 0.03, 1.0));
    // Grain: long, faint, gently wavy lines.
    for i in 0..48u32 {
        let y0 = hash01(i, 3) * h;
        let amp = 2.0 + hash01(i, 4) * 6.0;
        let a = 0.03 + hash01(i, 5) * 0.04;
        let mut prev = vec2(0.0, y0);
        let steps = 24;
        for s in 1..=steps {
            let x = w * s as f32 / steps as f32;
            let y = y0 + (x / w * 6.0 + i as f32).sin() * amp;
            draw_line(prev.x, prev.y, x, y, 1.0, Color::new(0.9, 0.7, 0.5, a));
            prev = vec2(x, y);
        }
    }
}

/// A dark wooden frame with a brass edge and an inner groove.
fn panel(r: Rect, accent: Color, u: f32) {
    let radius = 4.0 * u;
    style::halo(r, radius, 6.0 * u, BLACK, 0.18);
    style::round_rect(r, radius, DARK_WOOD, WALNUT);
    style::round_outline(r, radius, (2.0 * u).max(1.0), style::mix(BRASS, accent, 0.25));
    let i = 4.0 * u;
    style::round_outline(Rect::new(r.x + i, r.y + i, r.w - 2.0 * i, r.h - 2.0 * i), radius, u.max(1.0), Color::new(0.38, 0.24, 0.14, 1.0));
}

/// The board, checkered in two woods (low contrast, so the tiles stand out).
fn well(well: Rect, board: Rect, cell: f32, _u: f32) {
    draw_rectangle(well.x, well.y, well.w, well.h, EBONY);
    let cols = (board.w / cell).round() as usize;
    let rows = (board.h / cell).round() as usize;
    for y in 0..rows {
        for x in 0..cols {
            let c = if (x + y) % 2 == 0 { LIGHT_SQUARE } else { DARK_SQUARE };
            draw_rectangle(board.x + x as f32 * cell, board.y + y as f32 * cell, cell, cell, c);
        }
    }
}

fn well_edge(well: Rect, u: f32) {
    draw_rectangle_lines(well.x - u, well.y - u, well.w + 2.0 * u, well.h + 2.0 * u, (2.0 * u).max(1.0) * 2.0, EBONY);
    draw_rectangle_lines(well.x, well.y, well.w, well.h, (1.5 * u).max(1.0) * 2.0, BRASS);
}

/// A lacquered wooden tile: the piece's colour stained into wood, bevelled, with grain.
fn block(x: f32, y: f32, size: f32, color: Color) {
    let gap = (size * 0.05).max(1.0);
    let r = Rect::new(x + gap, y + gap, size - 2.0 * gap, size - 2.0 * gap);
    let a = color.a;
    let base = style::mix(color, Color::new(0.45, 0.28, 0.15, 1.0), 0.35).with_alpha(a);
    draw_rectangle(r.x, r.y, r.w, r.h, base);
    let e = (size * 0.1).max(1.0).round();
    bevel(r, e, style::mix(base, WHITE, 0.3).with_alpha(a), style::mix(base, BLACK, 0.45).with_alpha(a), true);
    let grain = style::mix(base, BLACK, 0.25).with_alpha(0.35 * a);
    for k in [0.38, 0.66] {
        draw_line(r.x + e, r.y + r.h * k, r.right() - e, r.y + r.h * (k + 0.06), 1.0, grain);
    }
}

fn ghost(x: f32, y: f32, size: f32, _color: Color) {
    let gap = (size * 0.08).max(1.0);
    draw_rectangle_lines(x + gap, y + gap, size - 2.0 * gap, size - 2.0 * gap, (size * 0.08).max(1.0) * 2.0, style::alpha(BRASS, 0.6));
}
