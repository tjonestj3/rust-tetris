//! A label crossing the board (T-SPIN DOUBLE), in the theme's style (graphics::theme::LabelStyle).
//! Every style arrives over `0..arrive` seconds, stays, and is gone 0.3 s after `leave`.

use macroquad::prelude::*;

use super::{ease_in_cubic, ease_out_cubic, style, text_centered, FxPalette};
use crate::graphics::text::{draw_text, measure_text};
use crate::graphics::theme::LabelStyle;

/// Seconds a label takes to leave.
const GONE: f32 = 0.3;

/// Draw `label` across `board` at height `cy`, `t` seconds in. `fill` is the label's colour.
#[allow(clippy::too_many_arguments)]
pub(super) fn draw(label: &str, board: Rect, cy: f32, cell: f32, t: f32, arrive: f32, leave: f32, fill: Color, pal: &FxPalette) {
    let inn = (t / arrive).clamp(0.0, 1.0);
    let out = ((t - leave) / GONE).clamp(0.0, 1.0);
    if t < 0.0 || out >= 1.0 {
        return;
    }
    let size = cell * 0.75;
    let u = (cell / 30.0).max(1.0);
    let w = measure_text(label, None, size as u16, 1.0).width;
    // Slides in from the left and off to the right.
    let slide = |width: f32| {
        let start = board.x - width;
        start + (board.center().x - start) * ease_out_cubic(inn) + ease_in_cubic(out) * board.w * 1.2
    };
    match pal.styles.label {
        LabelStyle::Slant => {
            let (bw, h) = (w + cell * 1.4, cell * 1.2);
            let cx = slide(bw);
            let skew = h * 0.3;
            let (l, r, top, bottom) = (cx - bw / 2.0, cx + bw / 2.0, cy - h / 2.0, cy + h / 2.0);
            let fill = style::alpha(fill, 0.94);
            draw_triangle(vec2(l + skew, top), vec2(r + skew, top), vec2(l - skew, bottom), fill);
            draw_triangle(vec2(r + skew, top), vec2(r - skew, bottom), vec2(l - skew, bottom), fill);
            text_centered(label, cx, cy, size, WHITE);
        }
        LabelStyle::Typed => {
            // "> T-SPIN DOUBLE", typed in and deleted, on a black line across the board.
            let line = format!("> {label}");
            let n = line.chars().count();
            let shown = ((inn * n as f32).floor() as usize).min(n - (out * n as f32).ceil() as usize);
            let text: String = line.chars().take(shown).collect();
            draw_rectangle(board.x, cy - cell * 0.6, board.w, cell * 1.2, style::alpha(pal.chip, 0.9));
            let x = board.x + cell * 0.4;
            let baseline = cy + size * 0.35;
            draw_text(&text, x, baseline, size, fill);
            let end = x + measure_text(&text, None, size as u16, 1.0).width;
            super::jonesin::cursor(end + size * 0.1, baseline, size, fill, inn < 1.0 || out > 0.0, t);
        }
        LabelStyle::Pixel => {
            // Flat pixel text with a hard shadow, blinking on and off as it comes and goes.
            let blinking = inn < 1.0 || out > 0.0;
            if blinking && (t * 16.0).fract() < 0.5 {
                return;
            }
            let shadow = (size * 0.1).max(2.0).round();
            text_centered(label, board.center().x + shadow, cy + shadow, size, pal.outline);
            text_centered(label, board.center().x, cy, size, if blinking { pal.stamp } else { fill });
        }
        LabelStyle::Window => {
            let small = (size * 0.55).max(9.0);
            let (bw, h) = ((w + cell * 1.2).min(board.w * 1.3), small * 1.7 + cell * 1.25);
            let cx = slide(bw);
            let r = Rect::new(cx - bw / 2.0, cy - h / 2.0, bw, h);
            let inner = match pal.window {
                Some(window) => window(r, "Rust Tetris", fill, small, u),
                None => {
                    (pal.panel)(r, fill, u);
                    r
                }
            };
            text_centered(label, inner.center().x, inner.center().y, size, pal.card_ink);
        }
        LabelStyle::Plank => {
            // A wooden plank with a brass rim; brass letters with a dark cut under them.
            let (bw, h) = (w + cell * 1.4, cell * 1.25);
            let cx = slide(bw);
            let r = Rect::new(cx - bw / 2.0, cy - h / 2.0, bw, h);
            style::round_rect(r, cell * 0.12, style::mix(pal.chip, WHITE, 0.1), style::mix(pal.chip, BLACK, 0.3));
            style::round_outline(r, cell * 0.12, (cell * 0.06).max(1.5), pal.stamp);
            text_centered(label, cx + u, cy + u, size, style::alpha(BLACK, 0.6));
            text_centered(label, cx, cy, size, pal.stamp);
        }
        LabelStyle::Stamp => {
            // An ink stamp: pressed down big and faint to its size, fading into the page.
            let k = 1.0 + 0.6 * (1.0 - ease_in_cubic(inn));
            let a = (0.3 + 0.7 * inn) * (1.0 - out);
            let (bw, h) = ((w + cell * 1.0) * k, cell * 1.3 * k);
            let r = Rect::new(board.center().x - bw / 2.0, cy - h / 2.0, bw, h);
            style::round_outline(r, cell * 0.15 * k, (cell * 0.07 * k).max(1.5), style::alpha(fill, a));
            text_centered(label, board.center().x, cy, size * k, style::alpha(fill, a));
        }
        LabelStyle::Pill => {
            // A frosted pill fading in, growing a touch, and fading out.
            let a = inn * (1.0 - out);
            let k = 0.9 + 0.1 * ease_out_cubic(inn);
            let (bw, h) = ((w + cell * 1.2) * k, cell * 1.2 * k);
            let r = Rect::new(board.center().x - bw / 2.0, cy - h / 2.0 - out * cell * 0.4, bw, h);
            style::round_rect(r, h / 2.0, Color::new(1.0, 1.0, 1.0, 0.8 * a), Color::new(1.0, 1.0, 1.0, 0.6 * a));
            style::round_outline(r, h / 2.0, 1.0, style::alpha(fill, 0.5 * a));
            text_centered(label, r.center().x, r.center().y, size * k, style::alpha(fill, a));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::theme::{get, ALL};

    #[test]
    fn chess_calls_a_t_spin_check() {
        let chess = get(crate::graphics::theme::ThemeId::Chess).fx.styles;
        assert_eq!(chess.tspin_names[2], "DOUBLE CHECK!");
        assert_eq!(chess.label, LabelStyle::Plank);
        // Everyone else keeps T-SPIN.
        for id in ALL.iter().filter(|&&id| id != crate::graphics::theme::ThemeId::Chess) {
            assert_eq!(get(*id).fx.styles.tspin_names[0], "T-SPIN", "{id:?}");
        }
    }
}
