//! Mac OS X (2001, Aqua): a blue desktop with soft swirls and the translucent menu bar along the
//! top, pinstriped windows with the red, yellow and green gel buttons, and blocks like Aqua's
//! glossy gel buttons.

use macroquad::prelude::*;

use crate::graphics::style;
use crate::graphics::text::{draw_text, measure_text, Face};
use crate::graphics::theme::{FxPalette, Theme, ThemeId};

const AQUA: Color = Color::new(0.16, 0.48, 0.93, 1.0);
const GRAPHITE: Color = Color::new(0.42, 0.47, 0.53, 1.0);
const STRIPE_A: Color = Color::new(0.93, 0.93, 0.93, 1.0);
const STRIPE_B: Color = Color::new(0.97, 0.97, 0.97, 1.0);
const EDGE: Color = Color::new(0.55, 0.57, 0.6, 1.0);
const INK: Color = Color::new(0.08, 0.08, 0.1, 1.0);
const RED: Color = Color::new(0.95, 0.25, 0.22, 1.0);
const AMBER: Color = Color::new(1.0, 0.74, 0.12, 1.0);
const LIME: Color = Color::new(0.3, 0.78, 0.22, 1.0);
const PURPLE: Color = Color::new(0.58, 0.36, 0.9, 1.0);
const ORANGE: Color = Color::new(1.0, 0.55, 0.12, 1.0);

pub const THEME: Theme = Theme {
    id: ThemeId::MacOsX,
    name: "MAC OS X",
    terminal: false,
    face: Face::NotoSans,
    window: Some(window),
    accents: [AQUA, RED, ORANGE, LIME, PURPLE],
    label_ink: None,
    value: INK,
    hint: GRAPHITE,
    tip_on: AQUA,
    tip_off: GRAPHITE,
    shade: Color::new(0.9, 0.92, 0.96, 0.82),
    shade_light: Color::new(0.9, 0.92, 0.96, 0.6),
    rainbow: [AQUA, PURPLE, RED, ORANGE, LIME, AQUA],
    surface: [STRIPE_B, STRIPE_A],
    pieces: Some([
        Color::new(0.22, 0.72, 1.0, 1.0),
        Color::new(1.0, 0.84, 0.1, 1.0),
        PURPLE,
        LIME,
        RED,
        Color::new(0.1, 0.38, 0.95, 1.0),
        ORANGE,
        Color::new(0.85, 0.88, 0.95, 1.0),
    ]),
    backdrop,
    panel,
    well,
    well_edge,
    block,
    ghost,
    fx: FxPalette {
        word: [AQUA, PURPLE, RED],
        hot: [AMBER, ORANGE, RED],
        edge: Color::new(0.1, 0.25, 0.55, 1.0),
        outline: WHITE,
        flash: WHITE,
        rings: [AQUA, WHITE],
        dim: Color::new(0.05, 0.15, 0.4, 1.0),
        banner: AQUA,
        banner_ink: WHITE,
        stamp: AMBER,
        heat: ORANGE,
        dust: Color::new(0.8, 0.85, 0.95, 1.0),
        tspin: PURPLE,
        combo: [AQUA, AMBER, RED],
        perfect: AMBER,
        card_edge: AQUA,
        grey: Color::new(0.7, 0.72, 0.76, 1.0),
        card_ink: INK,
        card_label: AQUA,
        chip: STRIPE_B,
        block,
        panel,
    },
};

fn backdrop(time: f32) {
    let (w, h) = (screen_width(), screen_height());
    style::gradient_rect(Rect::new(0.0, 0.0, w, h), Color::new(0.1, 0.33, 0.75, 1.0), Color::new(0.35, 0.62, 0.95, 1.0));
    // Soft light swirls across it.
    for i in 0..5 {
        let k = i as f32;
        let cx = w * (0.1 + 0.25 * k) + (time * 0.05 + k).sin() * w * 0.02;
        let cy = h * (0.75 - 0.12 * k);
        let r = w * (0.55 + 0.1 * k);
        draw_circle_lines(cx, cy, r, w * 0.05, Color::new(1.0, 1.0, 1.0, 0.05));
        draw_circle_lines(cx, cy, r * 0.92, w * 0.02, Color::new(1.0, 1.0, 1.0, 0.06));
    }
    // The menu bar: translucent white, the app's name in bold, its menus.
    let bar_h = (h * 0.028).clamp(20.0, 40.0).round();
    style::gradient_rect(Rect::new(0.0, 0.0, w, bar_h), Color::new(1.0, 1.0, 1.0, 0.88), Color::new(0.92, 0.93, 0.95, 0.88));
    draw_rectangle(0.0, bar_h - 1.0, w, 1.0, Color::new(0.0, 0.0, 0.0, 0.25));
    let size = bar_h * 0.6;
    let mut x = bar_h * 0.6;
    for (i, item) in ["Rust Tetris", "File", "Edit", "View", "Window"].iter().enumerate() {
        draw_text(item, x, bar_h * 0.5 + size * 0.36, size, INK);
        x += measure_text(item, None, size as u16, 1.0).width + bar_h * if i == 0 { 1.0 } else { 0.8 };
        if x > w * 0.85 {
            break;
        }
    }
}

/// Pinstripes across `r` (the window body and title bar).
fn pinstripes(r: Rect) {
    draw_rectangle(r.x, r.y, r.w, r.h, STRIPE_B);
    let mut y = r.y;
    while y < r.bottom() {
        draw_rectangle(r.x, y, r.w, 1.0_f32.min(r.bottom() - y), STRIPE_A);
        y += 2.0;
    }
}

/// A gel circle, the window buttons.
fn gel_dot(x: f32, y: f32, r: f32, color: Color) {
    draw_circle(x, y, r, style::mix(color, BLACK, 0.35));
    draw_circle(x, y, r * 0.86, color);
    draw_circle(x, y + r * 0.25, r * 0.55, style::mix(color, WHITE, 0.35));
    draw_ellipse(x, y - r * 0.42, r * 0.55, r * 0.3, 0.0, Color::new(1.0, 1.0, 1.0, 0.75));
}

/// A pinstriped window: the gel buttons on the left of the title bar, the title in the middle.
fn window(r: Rect, title: &str, _accent: Color, size: f32, u: f32) -> Rect {
    let radius = 6.0 * u;
    style::halo(r, radius, 8.0 * u, BLACK, 0.18);
    style::round_rect(r, radius, STRIPE_B, STRIPE_B);
    let bar_h = (size * 1.55).round();
    let bar = Rect::new(r.x, r.y, r.w, bar_h);
    pinstripes(Rect::new(r.x + radius * 0.5, r.y + 1.0, r.w - radius, bar_h - 1.0));
    style::gradient_rect(Rect::new(r.x + radius * 0.5, r.y + 1.0, r.w - radius, bar_h * 0.5), Color::new(1.0, 1.0, 1.0, 0.5), Color::new(1.0, 1.0, 1.0, 0.0));
    draw_rectangle(r.x, bar.bottom() - 1.0, r.w, 1.0, EDGE);
    let dot = bar_h * 0.26;
    let mut x = r.x + dot * 1.9;
    for c in [RED, AMBER, LIME] {
        gel_dot(x, bar.center().y, dot, c);
        x += dot * 2.6;
    }
    let ts = size * 0.85;
    let tw = measure_text(title, None, ts as u16, 1.0).width;
    let tx = (bar.center().x - tw / 2.0).max(x);
    if tx + tw < bar.right() - 4.0 * u {
        draw_text(title, tx, bar.center().y + ts * 0.36, ts, INK);
    }
    pinstripes(Rect::new(r.x + 1.0, bar.bottom(), r.w - 2.0, r.bottom() - bar.bottom() - radius * 0.5));
    style::round_outline(r, radius, u.max(1.0), EDGE);
    Rect::new(r.x + 1.0, bar.bottom(), r.w - 2.0, (r.bottom() - bar.bottom() - 2.0).max(0.0))
}

/// A plain Aqua panel (buttons, cards, the small boxes): pinstripes, a grey rim.
fn panel(r: Rect, accent: Color, u: f32) {
    let radius = 8.0 * u;
    style::halo(r, radius, 6.0 * u, BLACK, 0.14);
    style::round_rect(r, radius, STRIPE_B, STRIPE_A);
    style::round_rect(Rect::new(r.x + 2.0 * u, r.y + 2.0 * u, r.w - 4.0 * u, r.h * 0.45), radius * 0.8, Color::new(1.0, 1.0, 1.0, 0.6), Color::new(1.0, 1.0, 1.0, 0.0));
    style::round_outline(r, radius, u.max(1.0), style::mix(EDGE, accent, 0.35));
}

/// The board: a pinstriped document with faint rules.
fn well(well: Rect, board: Rect, cell: f32, u: f32) {
    style::round_rect(well, 4.0 * u, WHITE, WHITE);
    pinstripes(Rect::new(well.x + u, well.y + u, well.w - 2.0 * u, well.h - 2.0 * u));
    let cols = (board.w / cell).round() as usize;
    for x in 1..cols {
        draw_rectangle(board.x + x as f32 * cell, board.y, 1.0, board.h, Color::new(0.2, 0.3, 0.5, 0.06));
    }
}

fn well_edge(well: Rect, u: f32) {
    style::halo(well, 4.0 * u, 10.0 * u, BLACK, 0.2);
    style::round_outline(well, 4.0 * u, (1.2 * u).max(1.0), EDGE);
}

/// A gel block: deep colour at the edges, a bright glow low down, a glossy highlight across the
/// top half.
fn block(x: f32, y: f32, size: f32, color: Color) {
    let gap = (size * 0.05).max(1.0);
    let r = Rect::new(x + gap, y + gap, size - 2.0 * gap, size - 2.0 * gap);
    let radius = size * 0.3;
    let a = color.a;
    style::round_rect(r, radius, style::mix(color, BLACK, 0.3).with_alpha(a), style::mix(color, WHITE, 0.3).with_alpha(a));
    style::round_rect(
        Rect::new(r.x + r.w * 0.12, r.y + r.h * 0.5, r.w * 0.76, r.h * 0.42),
        radius * 0.7,
        style::alpha(style::mix(color, WHITE, 0.2), 0.0),
        style::alpha(style::mix(color, WHITE, 0.55), 0.8 * a),
    );
    style::round_rect(
        Rect::new(r.x + r.w * 0.1, r.y + r.h * 0.06, r.w * 0.8, r.h * 0.42),
        radius * 0.8,
        Color::new(1.0, 1.0, 1.0, 0.85 * a),
        Color::new(1.0, 1.0, 1.0, 0.12 * a),
    );
    style::round_outline(r, radius, (size * 0.04).max(1.0), style::mix(color, BLACK, 0.5).with_alpha(0.7 * a));
}

fn ghost(x: f32, y: f32, size: f32, color: Color) {
    let gap = (size * 0.05).max(1.0);
    let r = Rect::new(x + gap, y + gap, size - 2.0 * gap, size - 2.0 * gap);
    style::round_rect(r, size * 0.3, style::alpha(color, 0.2), style::alpha(color, 0.1));
    style::round_outline(r, size * 0.3, (size * 0.05).max(1.0), style::alpha(color, 0.55));
}
