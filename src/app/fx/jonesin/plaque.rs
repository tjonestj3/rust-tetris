//! Plaque (Chess): a wooden plaque with a brass rim and four screws drops into place, and the
//! letters are carved and gilded one after another, the last on the chord, when a glint of light
//! runs across the brass and wood chips fall. Back-to-back: the banner and the X2 seal. Then the
//! plaque is lifted away.

use macroquad::prelude::*;

use super::{back_to_back_banner, chars, ease_in_cubic, ease_out_cubic, hash01, lands, style, x2_stamp, Scene};
use crate::graphics::text::draw_text;

/// The plaque's drop into place, and its lift away.
const DROP: f32 = 0.16;
const LIFT: f32 = 0.35;

pub(super) fn draw(s: &Scene) {
    let (t, pal, base, screen) = (s.t, s.pal, s.fit.size, s.screen);
    s.dim(0.45);
    let w = (s.fit.total + base * 1.3).min(screen.w * 0.97);
    let h = base * 1.75;
    let home_y = s.cy - h / 2.0;
    let drop = ease_out_cubic((t / DROP).clamp(0.0, 1.0));
    let lift = ease_in_cubic(s.leaving(LIFT));
    let y = (screen.y - h) + (home_y - (screen.y - h)) * drop - (home_y - screen.y + h) * lift;
    let r = Rect::new(s.cx - w / 2.0, y, w, h);
    let dy = y - home_y;

    // The wood: a dark-to-darker board with grain running along it.
    let wood = pal.chip;
    style::halo(r, base * 0.14, base * 0.2, BLACK, 0.35);
    style::round_rect(r, base * 0.14, style::mix(wood, WHITE, 0.08), style::mix(wood, BLACK, 0.35));
    for i in 0..9u32 {
        let gy = r.y + r.h * (0.1 + 0.8 * i as f32 / 8.0) + hash01(i, 71) * base * 0.06;
        let steps = 12;
        for k in 0..steps {
            let x0 = r.x + base * 0.12 + (r.w - base * 0.24) * k as f32 / steps as f32;
            let x1 = r.x + base * 0.12 + (r.w - base * 0.24) * (k + 1) as f32 / steps as f32;
            let wave = |x: f32| ((x - r.x) / r.w * 9.0 + i as f32).sin() * base * 0.025;
            draw_line(x0, gy + wave(x0), x1, gy + wave(x1), (base * 0.015).max(1.0), style::alpha(BLACK, 0.14 + 0.08 * hash01(i, 72)));
        }
    }

    // The brass: a rim, an inner rule and the screws.
    let brass = pal.stamp;
    style::round_outline(r, base * 0.14, (base * 0.06).max(2.0), brass);
    let inset = base * 0.14;
    let inner = Rect::new(r.x + inset, r.y + inset, r.w - 2.0 * inset, r.h - 2.0 * inset);
    style::round_outline(inner, base * 0.06, (base * 0.02).max(1.0), style::alpha(brass, 0.7));
    for (sx, sy) in [(inner.x, inner.y), (inner.right(), inner.y), (inner.x, inner.bottom()), (inner.right(), inner.bottom())] {
        let rr = base * 0.065;
        draw_circle(sx, sy, rr, brass);
        draw_circle(sx - rr * 0.25, sy - rr * 0.25, rr * 0.4, style::mix(brass, WHITE, 0.5));
        draw_line(sx - rr * 0.7, sy + rr * 0.7, sx + rr * 0.7, sy - rr * 0.7, (rr * 0.3).max(1.0), style::mix(brass, BLACK, 0.5));
    }

    // The letters, carved (a dark cut above, a light edge below) and gilded.
    let size = base * 0.92;
    let baseline = s.cy + dy + size * 0.35;
    let cut = (size * 0.035).max(1.0);
    for (i, ch) in chars().iter().enumerate() {
        let since = t - lands(i);
        if since < 0.0 {
            continue;
        }
        let x = s.letter_x(i) - s.fit.widths[i] * 0.92 / 2.0;
        draw_text(ch, x - cut, baseline - cut, size, style::alpha(BLACK, 0.6));
        draw_text(ch, x + cut, baseline + cut, size, style::mix(wood, WHITE, 0.25));
        draw_text(ch, x, baseline, size, s.fill(i));
        let shine = (1.0 - since / 0.15).clamp(0.0, 1.0);
        if shine > 0.0 {
            draw_text(ch, x, baseline, size, style::alpha(pal.flash, 0.8 * shine));
        }
    }

    glint(s, r);
    chips(s, r);

    if s.hot {
        back_to_back_banner(s, r.y - base * 0.45);
        let x = (r.right() - base * 0.1).min(screen.right() - base * 0.45);
        x2_stamp(s, x, r.bottom());
    }
}

/// A band of light running across the plaque after the chord.
fn glint(s: &Scene, r: Rect) {
    let p = s.since() / 0.4;
    if !(0.0..1.0).contains(&p) {
        return;
    }
    let bw = r.h * 0.5;
    let skew = r.h * 0.35;
    let x = r.x + skew + (r.w - bw - 2.0 * skew) * ease_out_cubic(p);
    let c = style::alpha(s.pal.flash, 0.28 * (1.0 - p));
    let (top, bottom) = (r.y + s.fit.size * 0.06, r.bottom() - s.fit.size * 0.06);
    draw_triangle(vec2(x + skew, top), vec2(x + skew + bw, top), vec2(x - skew, bottom), c);
    draw_triangle(vec2(x + skew + bw, top), vec2(x - skew + bw, bottom), vec2(x - skew, bottom), c);
}

/// Wood chips falling from under the plaque after the chord.
fn chips(s: &Scene, r: Rect) {
    let since = s.since();
    if !(0.0..0.7).contains(&since) {
        return;
    }
    let p = since / 0.7;
    for i in 0..14u32 {
        let x = r.x + hash01(i, 81) * r.w;
        let vx = (hash01(i, 82) - 0.5) * s.fit.size * 2.0;
        let y = r.bottom() + s.screen.h * 0.5 * since * since + s.fit.size * 0.6 * since;
        let size = s.fit.size * (0.05 + 0.06 * hash01(i, 83));
        draw_rectangle_ex(x + vx * since, y, size, size * 0.6, DrawRectangleParams {
            offset: vec2(0.5, 0.5),
            rotation: hash01(i, 84) * 6.0 + since * 8.0,
            color: style::alpha(style::mix(s.pal.chip, WHITE, 0.3), 1.0 - p),
        });
    }
}
