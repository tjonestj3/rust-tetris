//! Stamp (Warm Paper): a rubber stamp comes down out of the air, faint and big, and presses the
//! word onto the page on the chord: a double ruled border, the word in ink, worn specks where the
//! ink didn't take, a splatter round the edge. Back-to-back: a round X2 stamp follows it. Then
//! the ink fades into the paper.

use macroquad::prelude::*;

use super::{chars, ease_in_cubic, ease_out_cubic, hash01, style, Scene, BANG};
use crate::graphics::text::draw_text;

/// The ink fading away.
const FADE: f32 = 0.4;

pub(super) fn draw(s: &Scene) {
    let (t, pal, base) = (s.t, s.pal, s.fit.size);
    s.dim(0.4);
    let fade = 1.0 - s.leaving(FADE);
    if fade <= 0.0 {
        return;
    }
    let ink = if s.hot { pal.hot[2] } else { pal.word[0] };
    let w = (s.fit.total + base * 0.9).min(s.screen.w * 0.97);
    let h = base * 1.5;
    let home = Rect::new(s.cx - w / 2.0, s.cy - h / 2.0, w, h);

    // Coming down: big and faint, faster as it nears the page, with its shadow gathering under it.
    let since = s.since();
    if since < 0.0 {
        let p = ease_in_cubic((t / BANG).clamp(0.0, 1.0));
        let k = 1.0 + 1.1 * (1.0 - p);
        let r = scaled(home, k);
        let shadow = scaled(home, 1.0 + 0.3 * (1.0 - p));
        style::round_rect(shadow, base * 0.2, style::alpha(pal.outline, 0.12 * p), style::alpha(pal.outline, 0.12 * p));
        stamp(s, r, k, ink, 0.12 + 0.3 * p, false);
        return;
    }
    // Pressed: squashed a touch for a moment.
    let squash = 1.0 - 0.04 * (1.0 - (since / 0.12).clamp(0.0, 1.0));
    stamp(s, scaled(home, squash), squash, ink, fade, true);
    splatter(s, home, ink, fade);

    if s.hot {
        let p = ((since - 0.16) / 0.12).clamp(0.0, 1.0);
        if p > 0.0 {
            let k = 1.0 + 1.2 * (1.0 - ease_out_cubic(p));
            let c = vec2((home.right() - base * 0.2).min(s.screen.right() - base * 0.6), home.bottom() + base * 0.1);
            let r = base * 0.55 * k;
            let a = fade * (0.3 + 0.7 * p);
            let gold = pal.hot[0];
            draw_circle_lines(c.x, c.y, r, (base * 0.08 * k).max(2.0), style::alpha(gold, a));
            draw_circle_lines(c.x, c.y, r * 0.8, (base * 0.03 * k).max(1.0), style::alpha(gold, a));
            super::centred("X2", c.x, c.y, base * 0.55 * k, style::alpha(gold, a));
        }
    }
}

fn scaled(r: Rect, k: f32) -> Rect {
    let c = r.center();
    Rect::new(c.x - r.w * k / 2.0, c.y - r.h * k / 2.0, r.w * k, r.h * k)
}

/// The stamp's face at `r` (its home size times `k`), in `ink` at alpha `a`; `worn` adds the
/// specks where the ink didn't take.
fn stamp(s: &Scene, r: Rect, k: f32, ink: Color, a: f32, worn: bool) {
    let base = s.fit.size;
    let ink = style::alpha(ink, a);
    let radius = base * 0.18 * k;
    style::round_outline(r, radius, (base * 0.09 * k).max(2.0), ink);
    let inset = base * 0.17 * k;
    let inner = Rect::new(r.x + inset, r.y + inset, r.w - 2.0 * inset, r.h - 2.0 * inset);
    style::round_outline(inner, (radius - inset).max(2.0), (base * 0.035 * k).max(1.0), ink);
    let size = base * 0.9 * k;
    for (i, ch) in chars().iter().enumerate() {
        let cx = s.cx + (s.letter_x(i) - s.cx) * 0.9 * k;
        let w = s.fit.widths[i] * 0.9 * k;
        draw_text(ch, cx - w / 2.0, r.center().y + size * 0.35, size, ink);
    }
    if worn {
        for i in 0..46u32 {
            let x = r.x + hash01(i, 51) * r.w;
            let y = r.y + hash01(i, 52) * r.h;
            let d = base * (0.02 + 0.05 * hash01(i, 53));
            draw_circle(x, y, d, style::alpha(s.pal.flash, 0.85 * a));
        }
    }
}

/// Ink flicked out round the edge as it presses.
fn splatter(s: &Scene, r: Rect, ink: Color, a: f32) {
    let p = ease_out_cubic((s.since() / 0.12).clamp(0.0, 1.0));
    let c = r.center();
    for i in 0..14u32 {
        let ang = hash01(i, 61) * std::f32::consts::TAU;
        let dir = vec2(ang.cos(), ang.sin() * 0.55);
        let edge = vec2(r.w / 2.0, r.h / 2.0) * dir.abs().max(vec2(0.2, 0.2));
        let out = s.fit.size * (0.15 + 0.6 * hash01(i, 62)) * p;
        let pos = c + dir.normalize() * (edge.length() + out);
        let d = s.fit.size * (0.03 + 0.07 * hash01(i, 63));
        draw_circle(pos.x, pos.y, d, style::alpha(ink, a * 0.85));
    }
}
