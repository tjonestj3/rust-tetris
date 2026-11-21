//! Glass: light and frosted. A soft pastel background with drifting blurs of colour, frosted white
//! cards, soft rounded blocks without hard outlines, dark text.

use macroquad::prelude::*;

use super::soft_circle;
use crate::graphics::style;
use crate::graphics::text::Face;
use crate::graphics::theme::{FxPalette, Theme, ThemeId};

const BLUE: Color = Color::new(0.04, 0.52, 1.0, 1.0);
const PINK: Color = Color::new(1.0, 0.22, 0.37, 1.0);
const ORANGE: Color = Color::new(1.0, 0.62, 0.04, 1.0);
const GREEN: Color = Color::new(0.19, 0.78, 0.35, 1.0);
const PURPLE: Color = Color::new(0.69, 0.32, 0.87, 1.0);
const TEAL: Color = Color::new(0.19, 0.69, 0.78, 1.0);
const RED: Color = Color::new(1.0, 0.27, 0.23, 1.0);
const INK: Color = Color::new(0.11, 0.11, 0.12, 1.0);
const GREY: Color = Color::new(0.56, 0.56, 0.58, 1.0);

pub const THEME: Theme = Theme {
    id: ThemeId::Glass,
    name: "GLASS",
    terminal: false,
    face: Face::Inter,
    window: None,
    accents: [BLUE, PINK, ORANGE, GREEN, PURPLE],
    label_ink: None,
    value: INK,
    hint: GREY,
    tip_on: PURPLE,
    tip_off: GREY,
    shade: Color::new(0.97, 0.97, 0.99, 0.75),
    shade_light: Color::new(0.97, 0.97, 0.99, 0.55),
    rainbow: [BLUE, PURPLE, PINK, ORANGE, GREEN, TEAL],
    surface: [Color::new(1.0, 1.0, 1.0, 1.0), Color::new(0.95, 0.95, 0.98, 1.0)],
    pieces: Some([
        Color::new(0.35, 0.78, 0.98, 1.0),
        Color::new(1.0, 0.8, 0.0, 1.0),
        PURPLE,
        GREEN,
        RED,
        BLUE,
        ORANGE,
        Color::new(0.85, 0.87, 0.95, 1.0),
    ]),
    backdrop,
    panel,
    well,
    well_edge,
    block,
    ghost,
    fx: FxPalette {
        word: [BLUE, PURPLE, PINK],
        hot: [ORANGE, PINK, RED],
        edge: Color::new(0.72, 0.75, 0.88, 1.0),
        outline: INK,
        flash: WHITE,
        rings: [BLUE, PINK],
        dim: WHITE,
        banner: PINK,
        banner_ink: WHITE,
        stamp: ORANGE,
        heat: ORANGE,
        dust: Color::new(0.6, 0.6, 0.7, 1.0),
        tspin: PURPLE,
        combo: [BLUE, ORANGE, PINK],
        perfect: ORANGE,
        card_edge: BLUE,
        grey: Color::new(0.78, 0.78, 0.82, 1.0),
        card_ink: INK,
        card_label: ORANGE,
        chip: WHITE,
        block,
        panel,
    },
};

fn backdrop(time: f32) {
    let (w, h) = (screen_width(), screen_height());
    style::gradient_rect(Rect::new(0.0, 0.0, w, h), Color::new(0.91, 0.93, 0.99, 1.0), Color::new(0.98, 0.92, 0.96, 1.0));
    let drift = |speed: f32, phase: f32| (time * speed + phase).sin();
    soft_circle(w * (0.22 + 0.04 * drift(0.11, 0.0)), h * (0.24 + 0.03 * drift(0.07, 1.0)), w * 0.55, style::alpha(BLUE, 0.9));
    soft_circle(w * (0.85 + 0.04 * drift(0.09, 2.0)), h * (0.72 + 0.03 * drift(0.08, 3.0)), w * 0.6, style::alpha(PINK, 0.8));
    soft_circle(w * (0.6 + 0.05 * drift(0.06, 4.0)), h * (0.45 + 0.04 * drift(0.1, 5.0)), w * 0.35, style::alpha(PURPLE, 0.6));
}

/// A frosted card: white, mostly see-through, a bright rim, a soft shadow.
fn panel(r: Rect, accent: Color, u: f32) {
    let radius = 14.0 * u;
    style::halo(r, radius, 8.0 * u, Color::new(0.2, 0.2, 0.35, 1.0), 0.05);
    style::round_rect(r, radius, Color::new(1.0, 1.0, 1.0, 0.7), Color::new(1.0, 1.0, 1.0, 0.5));
    style::round_outline(r, radius, (1.2 * u).max(1.0), style::alpha(style::mix(accent, WHITE, 0.55), 0.8));
}

fn well(well: Rect, board: Rect, cell: f32, u: f32) {
    let radius = 12.0 * u;
    style::round_rect(well, radius, Color::new(1.0, 1.0, 1.0, 0.45), Color::new(1.0, 1.0, 1.0, 0.32));
    let cols = (board.w / cell).round() as usize;
    let rows = (board.h / cell).round() as usize;
    for x in 1..cols {
        draw_rectangle(board.x + x as f32 * cell, board.y, 1.0, board.h, Color::new(0.1, 0.1, 0.3, 0.05));
    }
    for y in 1..rows {
        draw_rectangle(board.x, board.y + y as f32 * cell, board.w, 1.0, Color::new(0.1, 0.1, 0.3, 0.05));
    }
}

fn well_edge(well: Rect, u: f32) {
    style::round_outline(well, 12.0 * u, (1.5 * u).max(1.0), Color::new(1.0, 1.0, 1.0, 0.95));
}

/// A soft rounded block: lighter at the top, a gentle shine, a faint shadow under it.
fn block(x: f32, y: f32, size: f32, color: Color) {
    let gap = (size * 0.06).max(1.0);
    let r = Rect::new(x + gap, y + gap, size - 2.0 * gap, size - 2.0 * gap);
    let radius = size * 0.28;
    let a = color.a;
    style::round_rect(Rect::new(r.x, r.y + size * 0.05, r.w, r.h), radius, Color::new(0.1, 0.1, 0.3, 0.10 * a), Color::new(0.1, 0.1, 0.3, 0.0));
    // See-through: the board shows a little through it.
    style::round_rect(r, radius, style::mix(color, WHITE, 0.35).with_alpha(0.88 * a), color.with_alpha(0.82 * a));
    // Light gathering at the bottom, as through glass.
    style::round_rect(
        Rect::new(r.x + r.w * 0.1, r.bottom() - r.h * 0.38, r.w * 0.8, r.h * 0.32),
        radius * 0.7,
        style::mix(color, WHITE, 0.6).with_alpha(0.0),
        style::mix(color, WHITE, 0.6).with_alpha(0.55 * a),
    );
    style::round_rect(Rect::new(r.x + r.w * 0.15, r.y + r.h * 0.1, r.w * 0.7, r.h * 0.28), radius * 0.6, Color::new(1.0, 1.0, 1.0, 0.5 * a), Color::new(1.0, 1.0, 1.0, 0.0));
    // A crisp bright rim.
    style::round_outline(r, radius, (size * 0.035).max(1.0), Color::new(1.0, 1.0, 1.0, 0.7 * a));
}

fn ghost(x: f32, y: f32, size: f32, color: Color) {
    let gap = (size * 0.06).max(1.0);
    let r = Rect::new(x + gap, y + gap, size - 2.0 * gap, size - 2.0 * gap);
    let radius = size * 0.28;
    style::round_rect(r, radius, style::alpha(color, 0.22), style::alpha(color, 0.14));
    style::round_outline(r, radius, (size * 0.05).max(1.0), style::alpha(color, 0.5));
}
