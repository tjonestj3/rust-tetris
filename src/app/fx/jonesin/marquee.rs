//! Marquee (Arcade): a lit sign switches on like a CRT, bulbs chase round its edge, and the
//! letters flicker on one by one like neon tubes; on the chord every bulb flashes and pixel sparks
//! burst from the corners. Back-to-back: BONUS X2 blinks under the sign. Then the sign switches
//! off, collapsing to a line and a dot.

use macroquad::prelude::*;

use super::{chars, centred, ease_out_cubic, hash01, lands, style, Scene, OUT};
use crate::graphics::text::draw_text;

/// The CRT switch-on, and the switch-off (to a line, then to a dot).
const ON: f32 = 0.08;
const OFF_LINE: f32 = 0.14;
const OFF_DOT: f32 = 0.1;

pub(super) fn draw(s: &Scene) {
    let (t, pal, base) = (s.t, s.pal, s.fit.size);
    s.dim(0.55);
    let w = (s.fit.total + base * 1.1).min(s.screen.w * 0.98);
    let h = base * 1.7;
    let sign = Rect::new(s.cx - w / 2.0, s.cy - h / 2.0, w, h);

    // Switching off: the picture squeezes to a bright line, then to a dot.
    let off = t - OUT;
    if off > 0.0 {
        let line = (off / OFF_LINE).clamp(0.0, 1.0);
        let dot = ((off - OFF_LINE) / OFF_DOT).clamp(0.0, 1.0);
        if dot >= 1.0 {
            return;
        }
        let lh = (sign.h * (1.0 - ease_out_cubic(line))).max(2.0 * s.u);
        let lw = sign.w * (1.0 - ease_out_cubic(dot)).max(0.02);
        draw_rectangle(s.cx - lw / 2.0, s.cy - lh / 2.0, lw, lh, style::alpha(pal.flash, 0.9));
        return;
    }

    // Switching on: a line opening up into the sign.
    let on = ease_out_cubic((t / ON).clamp(0.0, 1.0));
    let r = Rect::new(sign.x, s.cy - sign.h * on / 2.0, sign.w, (sign.h * on).max(2.0 * s.u));
    draw_rectangle(r.x, r.y, r.w, r.h, style::alpha(pal.outline, 0.92));
    if on < 1.0 {
        draw_rectangle(r.x, r.y, r.w, r.h, style::alpha(pal.flash, 0.6 * (1.0 - on)));
        return;
    }

    // The frame: two neon tubes.
    let tube = (base * 0.06).max(2.0);
    for (inset, color) in [(0.0, pal.word[2]), (tube * 2.5, pal.word[1])] {
        let f = Rect::new(sign.x + inset, sign.y + inset, sign.w - 2.0 * inset, sign.h - 2.0 * inset);
        draw_rectangle_lines(f.x - tube, f.y - tube, f.w + 2.0 * tube, f.h + 2.0 * tube, tube, style::alpha(color, 0.25));
        draw_rectangle_lines(f.x, f.y, f.w, f.h, tube, color);
    }

    // The bulbs, chasing (and all flashing on the chord).
    let bulbs = bulb_spots(sign, base * 0.34);
    let since = s.since();
    let all = (0.0..0.4).contains(&since) && (since * 20.0).fract() < 0.5;
    let chase = (t * 18.0) as usize;
    let lit_color = if s.hot { pal.hot[0] } else { pal.stamp };
    for (k, p) in bulbs.iter().enumerate() {
        let lit = all || (k + chase) % 3 == 0;
        let rb = (base * 0.065).max(2.0);
        if lit {
            draw_circle(p.x, p.y, rb * 2.2, style::alpha(lit_color, 0.25));
            draw_circle(p.x, p.y, rb, lit_color);
            draw_circle(p.x - rb * 0.3, p.y - rb * 0.3, rb * 0.35, WHITE);
        } else {
            draw_circle(p.x, p.y, rb, style::mix(lit_color, BLACK, 0.7));
        }
    }

    // The letters: dark tubes until they flicker on.
    let baseline = s.cy + base * 0.35;
    for (i, ch) in chars().iter().enumerate() {
        let x = s.letter_x(i) - s.fit.widths[i] / 2.0;
        let since_on = t - lands(i) + 0.05;
        let flicker = since_on < 0.0 || (since_on < 0.09 && hash01(i as u32, (since_on * 60.0) as u32) < 0.45);
        if flicker {
            draw_text(ch, x, baseline, base, style::alpha(s.fill(i), 0.14));
            continue;
        }
        let fill = s.fill(i);
        let glow = (base * 0.045).max(1.5);
        for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
            draw_text(ch, x + dx * glow, baseline + dy * glow, base, style::alpha(fill, 0.35));
        }
        draw_text(ch, x, baseline, base, fill);
        let hot_core = (1.0 - since_on / 0.2).clamp(0.0, 1.0);
        if hot_core > 0.0 {
            draw_text(ch, x, baseline, base, style::alpha(pal.flash, 0.8 * hot_core));
        }
    }

    sparks(s, sign);

    if s.hot && since > 0.0 && (t * 4.0).fract() < 0.7 {
        let size = base * 0.45;
        centred("BONUS X2", s.cx + 2.0 * s.u, sign.bottom() + size * 1.1 + 2.0 * s.u, size, pal.outline);
        centred("BONUS X2", s.cx, sign.bottom() + size * 1.1, size, pal.hot[1]);
    }
}

/// Where the bulbs go: evenly round the sign's edge, about `spacing` apart.
fn bulb_spots(sign: Rect, spacing: f32) -> Vec<Vec2> {
    let e = Rect::new(sign.x - spacing * 0.45, sign.y - spacing * 0.45, sign.w + spacing * 0.9, sign.h + spacing * 0.9);
    let perimeter = 2.0 * (e.w + e.h);
    let n = ((perimeter / spacing.max(1.0)) as usize).clamp(8, 160);
    (0..n)
        .map(|k| {
            let mut d = k as f32 / n as f32 * perimeter;
            if d < e.w {
                return vec2(e.x + d, e.y);
            }
            d -= e.w;
            if d < e.h {
                return vec2(e.right(), e.y + d);
            }
            d -= e.h;
            if d < e.w {
                return vec2(e.right() - d, e.bottom());
            }
            vec2(e.x, e.bottom() - (d - e.w))
        })
        .collect()
}

/// Square pixel sparks bursting out from the sign's corners on the chord.
fn sparks(s: &Scene, sign: Rect) {
    let since = s.since();
    if !(0.0..0.6).contains(&since) {
        return;
    }
    let p = since / 0.6;
    let size = (s.fit.size * 0.09).max(3.0).round();
    let colors = [s.pal.word[0], s.pal.word[1], s.pal.word[2], s.pal.stamp];
    for i in 0..24u32 {
        let corner = [vec2(sign.x, sign.y), vec2(sign.right(), sign.y), vec2(sign.x, sign.bottom()), vec2(sign.right(), sign.bottom())][(i % 4) as usize];
        let out = vec2(if corner.x < s.cx { -1.0 } else { 1.0 }, if corner.y < s.cy { -1.0 } else { 1.0 });
        let a = hash01(i, 41) * std::f32::consts::FRAC_PI_2;
        let dir = vec2(a.cos() * out.x, a.sin() * out.y);
        let dist = s.fit.size * (1.0 + 2.0 * hash01(i, 42)) * ease_out_cubic(p);
        let pos = corner + dir * dist + vec2(0.0, s.screen.h * 0.15 * p * p);
        // Pixel steps, not smooth motion.
        let snap = |v: f32| (v / size).round() * size;
        draw_rectangle(snap(pos.x), snap(pos.y), size, size, style::alpha(colors[(i % 4) as usize], 1.0 - p));
    }
}
