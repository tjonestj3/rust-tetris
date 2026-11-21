//! Desktop 98: a late-90s desktop. A teal background, grey bevelled windows, a sunken black
//! play field, and raised 3D blocks.

use macroquad::prelude::*;

use super::bevel;
use crate::graphics::style;
use crate::graphics::theme::{FxPalette, Theme, ThemeId};

const TEAL: Color = Color::new(0.0, 0.5, 0.5, 1.0);
const FACE: Color = Color::new(0.75, 0.75, 0.75, 1.0);
const HILIGHT: Color = Color::new(0.87, 0.87, 0.87, 1.0);
const SHADOW: Color = Color::new(0.5, 0.5, 0.5, 1.0);
const NAVY: Color = Color::new(0.0, 0.0, 0.5, 1.0);
const MAROON: Color = Color::new(0.5, 0.0, 0.0, 1.0);
const OLIVE: Color = Color::new(0.45, 0.45, 0.0, 1.0);
const GREEN: Color = Color::new(0.0, 0.45, 0.0, 1.0);
const PURPLE: Color = Color::new(0.5, 0.0, 0.5, 1.0);
const BLUE: Color = Color::new(0.06, 0.52, 0.82, 1.0);
const RED: Color = Color::new(1.0, 0.0, 0.0, 1.0);
const YELLOW: Color = Color::new(1.0, 1.0, 0.0, 1.0);
const ORANGE: Color = Color::new(1.0, 0.5, 0.0, 1.0);

pub const THEME: Theme = Theme {
    id: ThemeId::Desktop98,
    name: "DESKTOP 98",
    terminal: false,
    accents: [NAVY, MAROON, OLIVE, GREEN, PURPLE],
    label_ink: None,
    value: BLACK,
    hint: Color::new(0.25, 0.25, 0.25, 1.0),
    tip_on: NAVY,
    tip_off: SHADOW,
    shade: Color::new(0.0, 0.5, 0.5, 0.92),
    shade_light: Color::new(0.0, 0.5, 0.5, 0.75),
    rainbow: [WHITE, YELLOW, Color::new(0.0, 1.0, 1.0, 1.0), WHITE, YELLOW, Color::new(0.0, 1.0, 1.0, 1.0)],
    surface: [FACE, FACE],
    pieces: Some([
        Color::new(0.0, 0.75, 0.75, 1.0),
        Color::new(0.85, 0.85, 0.0, 1.0),
        Color::new(0.6, 0.0, 0.6, 1.0),
        Color::new(0.0, 0.6, 0.0, 1.0),
        Color::new(0.75, 0.0, 0.0, 1.0),
        Color::new(0.0, 0.0, 0.75, 1.0),
        Color::new(0.85, 0.45, 0.0, 1.0),
        FACE,
    ]),
    backdrop,
    panel,
    well,
    well_edge,
    block,
    ghost,
    fx: FxPalette {
        word: [BLUE, RED, YELLOW],
        hot: [YELLOW, ORANGE, RED],
        edge: Color::new(0.25, 0.25, 0.25, 1.0),
        outline: BLACK,
        flash: WHITE,
        rings: [WHITE, YELLOW],
        dim: Color::new(0.0, 0.25, 0.25, 1.0),
        banner: NAVY,
        banner_ink: WHITE,
        stamp: YELLOW,
        heat: ORANGE,
        dust: HILIGHT,
        tspin: PURPLE,
        combo: [NAVY, OLIVE, MAROON],
        perfect: YELLOW,
        card_edge: NAVY,
        grey: SHADOW,
        card_ink: BLACK,
        card_label: NAVY,
        chip: FACE,
        block,
        panel,
    },
};

fn backdrop(_time: f32) {
    clear_background(TEAL);
}

/// A grey window with a raised edge, and a thin title rule in its accent.
fn panel(r: Rect, accent: Color, u: f32) {
    draw_rectangle(r.x, r.y, r.w, r.h, FACE);
    let t = u.round().max(1.0);
    bevel(r, t, WHITE, BLACK, true);
    bevel(Rect::new(r.x + t, r.y + t, r.w - 2.0 * t, r.h - 2.0 * t), t, HILIGHT, SHADOW, true);
    draw_rectangle(r.x + 3.0 * t, r.y + 3.0 * t, r.w - 6.0 * t, 2.0 * t, style::alpha(accent, 0.8));
}

fn well(well: Rect, _board: Rect, _cell: f32, _u: f32) {
    draw_rectangle(well.x, well.y, well.w, well.h, BLACK);
}

/// A sunken field: dark top and left, light bottom and right.
fn well_edge(well: Rect, u: f32) {
    let t = u.round().max(1.0);
    let outer = Rect::new(well.x - 2.0 * t, well.y - 2.0 * t, well.w + 4.0 * t, well.h + 4.0 * t);
    bevel(outer, t, WHITE, SHADOW, false);
    bevel(Rect::new(outer.x + t, outer.y + t, outer.w - 2.0 * t, outer.h - 2.0 * t), t, HILIGHT, BLACK, false);
}

/// A raised 3D block: light edges top and left, dark bottom and right.
fn block(x: f32, y: f32, size: f32, color: Color) {
    let a = color.a;
    draw_rectangle(x, y, size, size, color);
    let e = (size * 0.12).max(1.0).round();
    bevel(Rect::new(x, y, size, size), e, style::mix(color, WHITE, 0.55).with_alpha(a), style::mix(color, BLACK, 0.5).with_alpha(a), true);
}

/// Where the piece lands: a dotted grey outline.
fn ghost(x: f32, y: f32, size: f32, _color: Color) {
    let dot = (size * 0.08).max(1.0).round();
    let n = ((size / (dot * 2.0)) as i32).max(2);
    let step = size / n as f32;
    for i in 0..n {
        let d = i as f32 * step;
        for (px, py) in [(x + d, y), (x + d, y + size - dot), (x, y + d), (x + size - dot, y + d)] {
            draw_rectangle(px, py, dot, dot, HILIGHT);
        }
    }
}
