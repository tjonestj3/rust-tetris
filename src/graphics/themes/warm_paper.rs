//! Warm Paper: cream paper with a fine grain, ruled like a notebook, flat blocks in muted inks,
//! coral as the accent.

use macroquad::prelude::*;

use super::hash01;
use crate::graphics::style;
use crate::graphics::text::Face;
use crate::graphics::theme::{FxPalette, Theme, ThemeId, FxStyles, JonesinStyle, LabelStyle, DEFAULT_STYLES};

const CREAM: Color = Color::new(0.96, 0.94, 0.90, 1.0);
const PAPER: Color = Color::new(0.985, 0.975, 0.95, 1.0);
const RULE: Color = Color::new(0.86, 0.82, 0.75, 1.0);
const INK: Color = Color::new(0.17, 0.16, 0.15, 1.0);
const WARM_GREY: Color = Color::new(0.6, 0.57, 0.51, 1.0);
const CORAL: Color = Color::new(0.85, 0.47, 0.34, 1.0);
const MUSTARD: Color = Color::new(0.79, 0.62, 0.15, 1.0);
const SAGE: Color = Color::new(0.42, 0.6, 0.36, 1.0);
const SLATE: Color = Color::new(0.35, 0.53, 0.64, 1.0);
const PLUM: Color = Color::new(0.54, 0.42, 0.66, 1.0);
const BRICK: Color = Color::new(0.7, 0.27, 0.2, 1.0);

pub const THEME: Theme = Theme {
    id: ThemeId::WarmPaper,
    name: "WARM PAPER",
    terminal: false,
    face: Face::Lora,
    window: None,
    accents: [SLATE, CORAL, MUSTARD, SAGE, PLUM],
    label_ink: None,
    value: INK,
    hint: WARM_GREY,
    tip_on: CORAL,
    tip_off: WARM_GREY,
    shade: Color::new(0.96, 0.94, 0.90, 0.9),
    shade_light: Color::new(0.96, 0.94, 0.90, 0.72),
    rainbow: [CORAL, MUSTARD, SAGE, SLATE, PLUM, BRICK],
    surface: [PAPER, CREAM],
    pieces: Some([
        SLATE,
        MUSTARD,
        PLUM,
        SAGE,
        CORAL,
        Color::new(0.25, 0.32, 0.5, 1.0),
        Color::new(0.76, 0.45, 0.2, 1.0),
        Color::new(0.75, 0.72, 0.66, 1.0),
    ]),
    backdrop,
    panel,
    well,
    well_edge,
    block,
    ghost,
    fx: FxPalette {
        word: [CORAL, MUSTARD, SAGE],
        hot: [MUSTARD, CORAL, BRICK],
        edge: Color::new(0.78, 0.72, 0.62, 1.0),
        outline: INK,
        flash: PAPER,
        rings: [CORAL, MUSTARD],
        dim: CREAM,
        banner: CORAL,
        banner_ink: PAPER,
        stamp: MUSTARD,
        heat: CORAL,
        dust: WARM_GREY,
        tspin: PLUM,
        combo: [SLATE, MUSTARD, CORAL],
        perfect: MUSTARD,
        card_edge: CORAL,
        grey: Color::new(0.8, 0.77, 0.72, 1.0),
        card_ink: INK,
        card_label: CORAL,
        chip: PAPER,
        block,
        panel,
        window: None,
        styles: FxStyles {
            jonesin: JonesinStyle::Stamp,
            label: LabelStyle::Stamp,
            ..DEFAULT_STYLES
        },
    },
};

fn backdrop(_time: f32) {
    let (w, h) = (screen_width(), screen_height());
    clear_background(CREAM);
    // Paper grain: fixed specks.
    let n = ((w * h) / 900.0) as u32;
    for i in 0..n {
        let x = hash01(i, 7) * w;
        let y = hash01(i, 8) * h;
        let a = 0.03 + hash01(i, 9) * 0.05;
        draw_rectangle(x, y, 1.0, 1.0, style::alpha(INK, a));
    }
}

/// A sheet of paper with a thin rule round it and a coloured tab down its left edge.
fn panel(r: Rect, accent: Color, u: f32) {
    let radius = 6.0 * u;
    style::halo(r, radius, 4.0 * u, Color::new(0.4, 0.3, 0.2, 1.0), 0.05);
    style::round_rect(r, radius, PAPER, PAPER);
    style::round_outline(r, radius, (1.2 * u).max(1.0), RULE);
    draw_rectangle(r.x, r.y + radius, (3.0 * u).round().max(2.0), r.h - 2.0 * radius, accent);
}

/// A notebook page: ruled across at every row.
fn well(well: Rect, board: Rect, cell: f32, u: f32) {
    style::round_rect(well, 4.0 * u, PAPER, PAPER);
    let rows = (board.h / cell).round() as usize;
    for y in 1..rows {
        draw_rectangle(board.x, board.y + y as f32 * cell, board.w, 1.0, style::alpha(RULE, 0.7));
    }
}

fn well_edge(well: Rect, u: f32) {
    style::round_outline(well, 4.0 * u, (1.5 * u).max(1.0), WARM_GREY);
}

/// A flat block in a muted ink: the piece's colour softened towards the paper, a thin darker edge.
fn block(x: f32, y: f32, size: f32, color: Color) {
    let gap = (size * 0.07).max(1.0);
    let r = Rect::new(x + gap, y + gap, size - 2.0 * gap, size - 2.0 * gap);
    let a = color.a;
    let fill = style::mix(style::mix(color, WARM_GREY, 0.25), CREAM, 0.2).with_alpha(a);
    let radius = size * 0.1;
    // A slight ink shadow, as if stamped a touch off.
    let off = (size * 0.05).max(1.0);
    style::round_rect(Rect::new(r.x + off, r.y + off, r.w, r.h), radius, style::alpha(INK, 0.12 * a), style::alpha(INK, 0.12 * a));
    style::round_rect(r, radius, fill, fill);
    // Hand-inked hatching across the lower part.
    let ink = style::mix(fill, INK, 0.3).with_alpha(0.35 * a);
    let lw = (size * 0.035).max(1.0);
    for k in [0.45_f32, 0.65, 0.85] {
        let (x0, y0) = (r.x + r.w * (k - 0.35).max(0.05), r.bottom() - r.h * 0.08);
        let (x1, y1) = (r.x + r.w * k.min(0.95), r.y + r.h * (1.0 - k).max(0.35));
        draw_line(x0, y0, x1, y1, lw, ink);
    }
    style::round_outline(r, radius, (size * 0.05).max(1.0), style::mix(fill, INK, 0.45).with_alpha(a));
}

fn ghost(x: f32, y: f32, size: f32, color: Color) {
    let gap = (size * 0.07).max(1.0);
    let r = Rect::new(x + gap, y + gap, size - 2.0 * gap, size - 2.0 * gap);
    style::round_outline(r, size * 0.1, (size * 0.06).max(1.0), style::alpha(style::mix(color, INK, 0.2), 0.55));
}
