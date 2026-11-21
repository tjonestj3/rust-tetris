//! Slam (Neon): the letters slam down one after another, each landing with a punch and a white
//! flash, with a raised edge and a thick outline; when the ! lands the screen flashes, two
//! shockwaves ring out and confetti falls. Then the letters drop away.
//!
//! Back-to-back: gold-to-pink letters that shimmer, heat rising behind them, a BACK-TO-BACK
//! banner and an X2 stamp.

use macroquad::prelude::*;

use super::{back_to_back_banner, chars, confetti, hash01, letter, ring, style, x2_stamp, Letter, Scene, OUT};
use crate::graphics::text::{draw_text, measure_text};

pub(super) fn draw(s: &Scene) {
    let (t, screen, pal, base) = (s.t, s.screen, s.pal, s.fit.size);
    s.dim(0.35);
    if s.hot {
        draw_heat(s);
    }
    for (i, ch) in chars().iter().enumerate() {
        let Some(l) = letter(i, t) else { continue };
        let size = base * l.scale;
        let wobble = if s.hot { (t * 40.0 + i as f32).sin() * base * 0.015 } else { 0.0 };
        let top = screen.y - base;
        let y = s.cy + (top - s.cy) * l.above + screen.h * l.below;
        draw_letter(ch, s.letter_x(i) + wobble, y, size, s.fill(i), l, s);
    }

    // The ! lands: flash, shockwaves, confetti.
    s.flash(0.3);
    ring(s.cx, s.cy, s.since(), 0.55, screen.w * 0.5, pal.rings[0], base * 0.12);
    ring(s.cx, s.cy, s.since() - 0.08, 0.55, screen.w * 0.38, pal.rings[1], base * 0.08);
    confetti(s);

    if s.hot {
        back_to_back_banner(s, s.cy - base * 1.05);
        let x = (s.cx + s.fit.total / 2.0 + base * 0.35).min(screen.right() - base * 0.45);
        x2_stamp(s, x, s.cy + base * 0.75);
    }
}

fn draw_letter(ch: &str, cx: f32, cy: f32, size: f32, fill: Color, l: Letter, s: &Scene) {
    let pal = s.pal;
    let w = measure_text(ch, None, size as u16, 1.0).width;
    let x = cx - w / 2.0;
    let y = cy + size * 0.35;
    let a = l.alpha;
    // The raised edge, back to front.
    let step = size * 0.018;
    for d in (1..=5).rev() {
        let d = d as f32;
        draw_text(ch, x + d * step * 0.9, y + d * step * 1.1, size, style::alpha(pal.edge, a));
    }
    // A thick outline.
    let o = (size * 0.05).max(2.0);
    for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0), (-0.7, -0.7), (0.7, -0.7), (-0.7, 0.7), (0.7, 0.7)] {
        draw_text(ch, x + dx * o, y + dy * o, size, style::alpha(pal.outline, a));
    }
    draw_text(ch, x, y, size, style::alpha(fill, a));
    if l.flash > 0.0 {
        draw_text(ch, x, y, size, style::alpha(pal.flash, a * l.flash));
    }
}

/// Heat rising behind a back-to-back word.
fn draw_heat(s: &Scene) {
    let (t, screen, total) = (s.t, s.screen, s.fit.total);
    let a = (t / 0.3).clamp(0.0, 1.0) * (1.0 - ((t - OUT) / 0.45).clamp(0.0, 1.0));
    if a <= 0.0 {
        return;
    }
    for i in 0..16u32 {
        let x = s.cx - total * 0.55 + hash01(i, 11) * total * 1.1;
        let speed = 0.35 + hash01(i, 12) * 0.4;
        let phase = hash01(i, 13);
        let rise = ((t * speed) + phase).fract();
        let y = s.cy + screen.h * 0.12 - rise * screen.h * 0.4;
        let h = screen.h * 0.05;
        let w = (screen.w * 0.006).max(2.0) * (1.0 + hash01(i, 14));
        for k in 0..4 {
            let k = k as f32 / 4.0;
            draw_rectangle(x, y - h + k * h, w, h / 4.0 + 0.5, style::alpha(s.pal.heat, 0.35 * a * k));
        }
    }
}
