//! Windows 98: a teal desktop with the grey taskbar along the bottom (Start, the clock), every
//! panel a grey window with the navy title bar and its three buttons, a sunken black play field,
//! and blocks bevelled like the era's buttons, in VGA colours.

use macroquad::prelude::*;

use super::bevel;
use crate::graphics::style;
use crate::graphics::text::{draw_text, measure_text, Face};
use crate::graphics::theme::{FxPalette, Theme, ThemeId, FxStyles, JonesinStyle, LabelStyle, DEFAULT_STYLES};

const TEAL: Color = Color::new(0.0, 0.5, 0.5, 1.0);
const FACE: Color = Color::new(0.75, 0.75, 0.75, 1.0);
const HILIGHT: Color = Color::new(0.87, 0.87, 0.87, 1.0);
const SHADOW: Color = Color::new(0.5, 0.5, 0.5, 1.0);
const NAVY: Color = Color::new(0.0, 0.0, 0.5, 1.0);
const TITLE_RIGHT: Color = Color::new(0.06, 0.52, 0.82, 1.0);
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
    name: "WINDOWS 98",
    terminal: false,
    face: Face::Pixelify,
    window: Some(window),
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
        Color::new(0.9, 0.9, 0.0, 1.0),
        Color::new(0.62, 0.0, 0.62, 1.0),
        Color::new(0.0, 0.66, 0.0, 1.0),
        Color::new(0.8, 0.0, 0.0, 1.0),
        Color::new(0.0, 0.0, 0.8, 1.0),
        Color::new(0.9, 0.48, 0.0, 1.0),
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
        window: Some(window),
        styles: FxStyles {
            jonesin: JonesinStyle::Dialog,
            label: LabelStyle::Window,
            ..DEFAULT_STYLES
        },
    },
};

/// The classic raised edge: white and black outside, light and dark grey inside.
fn raised(r: Rect, t: f32) {
    bevel(r, t, WHITE, BLACK, true);
    bevel(Rect::new(r.x + t, r.y + t, r.w - 2.0 * t, r.h - 2.0 * t), t, HILIGHT, SHADOW, true);
}

fn sunken(r: Rect, t: f32) {
    bevel(r, t, WHITE, SHADOW, false);
    bevel(Rect::new(r.x + t, r.y + t, r.w - 2.0 * t, r.h - 2.0 * t), t, HILIGHT, BLACK, false);
}

fn backdrop(_time: f32) {
    let (w, h) = (screen_width(), screen_height());
    clear_background(TEAL);
    // The taskbar: Start on the left, the clock on the right.
    let bar_h = (h * 0.045).clamp(26.0, 60.0).round();
    let t = (bar_h / 28.0).round().max(1.0);
    let bar = Rect::new(0.0, h - bar_h, w, bar_h);
    draw_rectangle(bar.x, bar.y, bar.w, bar.h, FACE);
    draw_rectangle(bar.x, bar.y, bar.w, t, HILIGHT);
    draw_rectangle(bar.x, bar.y + t, bar.w, t, WHITE);
    let pad = 3.0 * t;
    let size = bar_h * 0.5;
    let start_w = measure_text("Start", None, size as u16, 1.0).width + bar_h * 1.1;
    let start = Rect::new(pad, bar.y + pad, start_w, bar_h - 2.0 * pad);
    draw_rectangle(start.x, start.y, start.w, start.h, FACE);
    raised(start, t);
    // A plain icon (four small panes) and the word.
    let icon = start.h * 0.5;
    let (ix, iy) = (start.x + start.h * 0.3, start.center().y - icon / 2.0);
    for (dx, dy, c) in [(0.0, 0.0, RED), (0.52, 0.0, GREEN), (0.0, 0.52, BLUE), (0.52, 0.52, YELLOW)] {
        draw_rectangle(ix + dx * icon, iy + dy * icon, icon * 0.46, icon * 0.46, c);
    }
    draw_text("Start", ix + icon * 1.3, start.center().y + size * 0.35, size, BLACK);
    let clock = chrono::Local::now().format("%-I:%M %p").to_string();
    let cw = measure_text(&clock, None, size as u16, 1.0).width + bar_h * 0.6;
    let tray = Rect::new(w - pad - cw, bar.y + pad, cw, bar_h - 2.0 * pad);
    bevel(tray, t, WHITE, SHADOW, false);
    draw_text(&clock, tray.x + bar_h * 0.3, tray.center().y + size * 0.35, size, BLACK);
}

/// A grey window: raised edge, the navy title bar with the title and three buttons.
fn window(r: Rect, title: &str, _accent: Color, size: f32, u: f32) -> Rect {
    draw_rectangle(r.x, r.y, r.w, r.h, FACE);
    let t = u.round().max(1.0);
    raised(r, t);
    let bar_h = (size * 1.45).round();
    let bar = Rect::new(r.x + 3.0 * t, r.y + 3.0 * t, r.w - 6.0 * t, bar_h);
    style::hgradient_rect(bar, NAVY, TITLE_RIGHT);
    // Close on the right; minimise and maximise beside it when the bar has room for them and
    // a title.
    let b = (bar_h - 4.0 * t).max(4.0);
    let close_x = bar.right() - 2.0 * t - b;
    let roomy = bar.w >= b * 3.0 + size * 4.0;
    let mut buttons = vec![("x", close_x)];
    if roomy {
        buttons.push(("box", close_x - 2.0 * t - b));
        buttons.push(("_", close_x - 2.0 * t - 2.0 * b));
    }
    let title_right = buttons.last().map(|&(_, x)| x).unwrap_or(bar.right()) - 3.0 * t;
    let mut ts = size * 0.85;
    let room = title_right - bar.x - 4.0 * t;
    let tw = measure_text(title, None, ts as u16, 1.0).width;
    if tw > room {
        ts *= (room / tw).max(0.0);
    }
    if ts >= size * 0.55 {
        draw_text(title, bar.x + 4.0 * t, bar.center().y + ts * 0.36, ts, WHITE);
    }
    for (kind, x) in buttons {
        let br = Rect::new(x, bar.y + 2.0 * t, b, b);
        draw_rectangle(br.x, br.y, br.w, br.h, FACE);
        raised(br, t.min(b / 8.0).max(1.0));
        let (cx, cy, s) = (br.center().x, br.center().y, b * 0.22);
        let lw = (b * 0.1).max(1.0);
        match kind {
            "x" => {
                draw_line(cx - s, cy - s, cx + s, cy + s, lw * 1.4, BLACK);
                draw_line(cx + s, cy - s, cx - s, cy + s, lw * 1.4, BLACK);
            }
            "box" => {
                draw_rectangle_lines(cx - s, cy - s, 2.0 * s, 2.0 * s, lw * 2.0, BLACK);
                draw_rectangle(cx - s, cy - s, 2.0 * s, lw * 1.5, BLACK);
            }
            _ => draw_rectangle(cx - s, cy + s * 0.6, s * 1.6, lw * 1.5, BLACK),
        }
    }
    Rect::new(r.x + 3.0 * t, bar.bottom() + t, r.w - 6.0 * t, (r.bottom() - 3.0 * t - bar.bottom() - t).max(0.0))
}

/// A plain raised panel (buttons, cards, the small boxes).
fn panel(r: Rect, _accent: Color, u: f32) {
    draw_rectangle(r.x, r.y, r.w, r.h, FACE);
    raised(r, u.round().max(1.0));
}

fn well(well: Rect, _board: Rect, _cell: f32, _u: f32) {
    draw_rectangle(well.x, well.y, well.w, well.h, BLACK);
}

/// The play field sits sunken in a grey frame.
fn well_edge(well: Rect, u: f32) {
    let t = u.round().max(1.0);
    let frame = Rect::new(well.x - 6.0 * t, well.y - 6.0 * t, well.w + 12.0 * t, well.h + 12.0 * t);
    // The grey frame round the field, drawn as four bands so the blocks stay uncovered.
    for r in [
        Rect::new(frame.x, frame.y, frame.w, 4.0 * t),
        Rect::new(frame.x, frame.bottom() - 4.0 * t, frame.w, 4.0 * t),
        Rect::new(frame.x, frame.y, 4.0 * t, frame.h),
        Rect::new(frame.right() - 4.0 * t, frame.y, 4.0 * t, frame.h),
    ] {
        draw_rectangle(r.x, r.y, r.w, r.h, FACE);
    }
    raised(frame, t);
    sunken(Rect::new(well.x - 2.0 * t, well.y - 2.0 * t, well.w + 4.0 * t, well.h + 4.0 * t), t);
}

/// A block bevelled like a 3D button: two steps of light along the top and left, two of shadow
/// along the bottom and right.
fn block(x: f32, y: f32, size: f32, color: Color) {
    let a = color.a;
    draw_rectangle(x, y, size, size, color);
    let e = (size * 0.07).max(1.0).round();
    let r = Rect::new(x, y, size, size);
    bevel(r, e, style::mix(color, WHITE, 0.7).with_alpha(a), style::mix(color, BLACK, 0.65).with_alpha(a), true);
    let inner = Rect::new(x + e, y + e, size - 2.0 * e, size - 2.0 * e);
    bevel(inner, e, style::mix(color, WHITE, 0.35).with_alpha(a), style::mix(color, BLACK, 0.3).with_alpha(a), true);
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
