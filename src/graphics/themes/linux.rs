//! Linux, Ubuntu style: the aubergine to orange desktop, the black top bar (Activities, the
//! clock) and the dock down the left, dark rounded windows with the header bar and the orange
//! close button, flat blocks in the desktop's accent colours, the Ubuntu font.

use macroquad::prelude::*;

use crate::graphics::style;
use crate::graphics::text::{draw_text, measure_text, Face};
use crate::graphics::theme::{FxPalette, Theme, ThemeId, FxStyles, JonesinStyle, LabelStyle, DEFAULT_STYLES};

const ORANGE: Color = Color::new(0.914, 0.329, 0.125, 1.0);
const AUBERGINE: Color = Color::new(0.467, 0.161, 0.325, 1.0);
const DARK_AUBERGINE: Color = Color::new(0.17, 0.0, 0.12, 1.0);
const HEADER: Color = Color::new(0.19, 0.19, 0.19, 1.0);
const BODY: Color = Color::new(0.24, 0.24, 0.24, 1.0);
const BORDER: Color = Color::new(0.1, 0.1, 0.1, 1.0);
const WARM_GREY: Color = Color::new(0.68, 0.65, 0.62, 1.0);
const TEXT: Color = Color::new(0.97, 0.97, 0.97, 1.0);
const BLUE: Color = Color::new(0.0, 0.45, 0.9, 1.0);
const PURPLE: Color = Color::new(0.47, 0.39, 0.85, 1.0);
const GREEN: Color = Color::new(0.29, 0.6, 0.12, 1.0);
const TEAL: Color = Color::new(0.19, 0.51, 0.5, 1.0);
const RED: Color = Color::new(0.85, 0.2, 0.31, 1.0);
const YELLOW: Color = Color::new(0.85, 0.6, 0.0, 1.0);

pub const THEME: Theme = Theme {
    id: ThemeId::Linux,
    name: "LINUX",
    terminal: false,
    face: Face::Ubuntu,
    window: Some(window),
    accents: [BLUE, ORANGE, YELLOW, GREEN, PURPLE],
    label_ink: None,
    value: TEXT,
    hint: WARM_GREY,
    tip_on: ORANGE,
    tip_off: WARM_GREY,
    shade: Color::new(0.1, 0.02, 0.08, 0.86),
    shade_light: Color::new(0.1, 0.02, 0.08, 0.65),
    rainbow: [ORANGE, YELLOW, GREEN, BLUE, PURPLE, RED],
    surface: [HEADER, BODY],
    pieces: Some([TEAL, YELLOW, PURPLE, GREEN, RED, BLUE, ORANGE, WARM_GREY]),
    backdrop,
    panel,
    well,
    well_edge,
    block,
    ghost,
    fx: FxPalette {
        word: [ORANGE, YELLOW, WHITE],
        hot: [YELLOW, ORANGE, RED],
        edge: DARK_AUBERGINE,
        outline: BLACK,
        flash: WHITE,
        rings: [ORANGE, WHITE],
        dim: DARK_AUBERGINE,
        banner: ORANGE,
        banner_ink: WHITE,
        stamp: YELLOW,
        heat: ORANGE,
        dust: WARM_GREY,
        tspin: PURPLE,
        combo: [BLUE, YELLOW, ORANGE],
        perfect: ORANGE,
        card_edge: ORANGE,
        grey: Color::new(0.35, 0.33, 0.33, 1.0),
        card_ink: TEXT,
        card_label: ORANGE,
        chip: HEADER,
        block,
        panel,
        window: Some(window),
        styles: FxStyles {
            jonesin: JonesinStyle::Shell,
            label: LabelStyle::Window,
            ..DEFAULT_STYLES
        },
    },
};

fn backdrop(_time: f32) {
    let (w, h) = (screen_width(), screen_height());
    style::gradient_rect(Rect::new(0.0, 0.0, w, h), DARK_AUBERGINE, AUBERGINE);
    // The warm glow from the lower right, and soft diagonal bands.
    super::soft_circle(w * 1.05, h * 1.0, w * 0.9, style::alpha(ORANGE, 0.9));
    super::soft_circle(w * 0.1, h * 0.1, w * 0.6, style::alpha(Color::new(0.35, 0.0, 0.25, 1.0), 0.8));
    // The top bar: Activities on the left, the clock in the middle.
    let bar_h = (h * 0.03).clamp(20.0, 40.0).round();
    draw_rectangle(0.0, 0.0, w, bar_h, BLACK);
    let size = bar_h * 0.58;
    draw_text("Activities", bar_h * 0.5, bar_h * 0.5 + size * 0.36, size, TEXT);
    let clock = chrono::Local::now().format("%a %H:%M").to_string();
    let cw = measure_text(&clock, None, size as u16, 1.0).width;
    draw_text(&clock, w / 2.0 - cw / 2.0, bar_h * 0.5 + size * 0.36, size, TEXT);
    // The dock down the left: a dark rounded strip with app squares.
    let dock_w = (w * 0.07).clamp(28.0, 64.0);
    let dock = Rect::new(0.0, bar_h, dock_w, h - bar_h);
    draw_rectangle(dock.x, dock.y, dock.w, dock.h, Color::new(0.0, 0.0, 0.0, 0.45));
    let icon = dock_w * 0.66;
    for (i, c) in [ORANGE, BLUE, GREEN, PURPLE, YELLOW, TEAL].iter().enumerate() {
        let y = dock.y + dock_w * 0.3 + i as f32 * (icon + dock_w * 0.25);
        if y + icon > dock.bottom() {
            break;
        }
        style::round_rect(Rect::new((dock_w - icon) / 2.0, y, icon, icon), icon * 0.22, style::mix(*c, WHITE, 0.15), *c);
    }
}

/// A dark rounded window: the header bar with the title in the middle and the orange close
/// button on the right.
fn window(r: Rect, title: &str, _accent: Color, size: f32, u: f32) -> Rect {
    let radius = 10.0 * u;
    style::halo(r, radius, 8.0 * u, BLACK, 0.3);
    style::round_rect(r, radius, BODY, BODY);
    let bar_h = (size * 1.65).round();
    style::round_rect(Rect::new(r.x, r.y, r.w, bar_h + radius), radius, HEADER, HEADER);
    draw_rectangle(r.x, r.y + bar_h, r.w, radius, BODY);
    draw_rectangle(r.x, r.y + bar_h, r.w, 1.0, BORDER);
    let close = bar_h * 0.24;
    let (cx, cy) = (r.right() - bar_h * 0.5, r.y + bar_h * 0.5);
    draw_circle(cx, cy, close, ORANGE);
    let s = close * 0.45;
    draw_line(cx - s, cy - s, cx + s, cy + s, (close * 0.22).max(1.0), WHITE);
    draw_line(cx + s, cy - s, cx - s, cy + s, (close * 0.22).max(1.0), WHITE);
    let ts = size * 0.82;
    let tw = measure_text(title, None, ts as u16, 1.0).width;
    if tw < r.w - bar_h * 1.4 {
        draw_text(title, r.center().x - tw / 2.0, cy + ts * 0.36, ts, TEXT);
    }
    style::round_outline(r, radius, u.max(1.0), BORDER);
    Rect::new(r.x, r.y + bar_h + 1.0, r.w, (r.h - bar_h - 1.0).max(0.0))
}

/// A plain dark panel (buttons, cards, the small boxes).
fn panel(r: Rect, accent: Color, u: f32) {
    let radius = 10.0 * u;
    style::halo(r, radius, 6.0 * u, BLACK, 0.25);
    style::round_rect(r, radius, HEADER, BODY);
    style::round_outline(r, radius, u.max(1.0), style::mix(BORDER, accent, 0.35));
}

fn well(well: Rect, board: Rect, cell: f32, u: f32) {
    style::round_rect(well, 8.0 * u, Color::new(0.12, 0.12, 0.12, 0.95), Color::new(0.09, 0.09, 0.09, 0.95));
    let cols = (board.w / cell).round() as usize;
    let rows = (board.h / cell).round() as usize;
    for x in 1..cols {
        draw_rectangle(board.x + x as f32 * cell, board.y, 1.0, board.h, Color::new(1.0, 1.0, 1.0, 0.04));
    }
    for y in 1..rows {
        draw_rectangle(board.x, board.y + y as f32 * cell, board.w, 1.0, Color::new(1.0, 1.0, 1.0, 0.04));
    }
}

fn well_edge(well: Rect, u: f32) {
    style::round_outline(well, 8.0 * u, (1.5 * u).max(1.0), style::alpha(ORANGE, 0.85));
}

/// A flat rounded block, a touch lighter along the top, a thin darker rim.
fn block(x: f32, y: f32, size: f32, color: Color) {
    let gap = (size * 0.06).max(1.0);
    let r = Rect::new(x + gap, y + gap, size - 2.0 * gap, size - 2.0 * gap);
    let radius = size * 0.18;
    let a = color.a;
    style::round_rect(r, radius, style::mix(color, WHITE, 0.12).with_alpha(a), color);
    draw_rectangle(r.x + radius, r.y + (size * 0.06).max(1.0), r.w - 2.0 * radius, (size * 0.05).max(1.0), Color::new(1.0, 1.0, 1.0, 0.3 * a));
    style::round_outline(r, radius, (size * 0.04).max(1.0), style::mix(color, BLACK, 0.4).with_alpha(a));
}

fn ghost(x: f32, y: f32, size: f32, color: Color) {
    let gap = (size * 0.06).max(1.0);
    let r = Rect::new(x + gap, y + gap, size - 2.0 * gap, size - 2.0 * gap);
    style::round_rect(r, size * 0.18, style::alpha(color, 0.18), style::alpha(color, 0.1));
    style::round_outline(r, size * 0.18, (size * 0.05).max(1.0), style::alpha(color, 0.6));
}
