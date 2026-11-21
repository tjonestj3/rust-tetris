//! Bloom (Glass): a frosted card settles in, and the letters fade in out of a soft blur one after
//! another; on the chord soft colour blooms out behind the card. Back-to-back: the card's rim
//! glows in the hot colours and says so under the word. Then it all drifts up and fades.

use macroquad::prelude::*;

use super::{centred, chars, ease_out_cubic, lands, style, Scene};
use crate::graphics::text::draw_text;
use crate::graphics::themes::soft_circle;

/// A letter's fade in, and the whole thing's fade out.
const IN: f32 = 0.12;
const FADE: f32 = 0.4;

pub(super) fn draw(s: &Scene) {
    let (t, pal, base) = (s.t, s.pal, s.fit.size);
    let fade = 1.0 - s.leaving(FADE);
    if fade <= 0.0 {
        return;
    }
    s.dim(0.25);
    let rise = (1.0 - fade) * base * 0.5;
    let since = s.since();

    // Soft colour blooming out behind the card.
    if since > 0.0 {
        let p = ease_out_cubic((since / 0.7).clamp(0.0, 1.0));
        let a = (1.0 - (since / 1.1).clamp(0.0, 1.0)) * fade;
        for (k, c) in s.stops().iter().enumerate() {
            let dx = (k as f32 - 1.0) * s.fit.total * 0.35;
            soft_circle(s.cx + dx, s.cy - rise, base * (1.2 + 2.6 * p), style::alpha(*c, 0.9 * a));
        }
    }

    // The card.
    let card_in = ease_out_cubic((t / 0.2).clamp(0.0, 1.0));
    let k = 0.88 + 0.12 * card_in;
    let w = (s.fit.total + base * 1.0).min(s.screen.w * 0.97) * k;
    let h = base * if s.hot { 2.1 } else { 1.55 } * k;
    let card = Rect::new(s.cx - w / 2.0, s.cy - base * 0.78 * k - rise, w, h);
    let a = card_in * fade;
    style::halo(card, h * 0.4, base * 0.3, style::mix(pal.word[0], WHITE, 0.3), 0.18 * a);
    style::round_rect(card, h * 0.4, Color::new(1.0, 1.0, 1.0, 0.72 * a), Color::new(1.0, 1.0, 1.0, 0.5 * a));
    if s.hot && since > 0.0 {
        let glow = (since / 0.2).clamp(0.0, 1.0) * a;
        for (j, c) in pal.hot.iter().enumerate() {
            let d = j as f32 * base * 0.05;
            let rim = Rect::new(card.x - d, card.y - d, card.w + 2.0 * d, card.h + 2.0 * d);
            style::round_outline(rim, h * 0.4 + d, (base * 0.04).max(1.5), style::alpha(*c, 0.7 * glow));
        }
    } else {
        style::round_outline(card, h * 0.4, (base * 0.03).max(1.0), Color::new(1.0, 1.0, 1.0, 0.9 * a));
    }

    // The letters, out of a blur: a few faint offset copies closing in, then sharp.
    let baseline = s.cy - rise + base * 0.35;
    for (i, ch) in chars().iter().enumerate() {
        let p = ((t - lands(i) + IN) / IN).clamp(0.0, 1.0);
        if p <= 0.0 {
            continue;
        }
        let x = s.letter_x(i) - s.fit.widths[i] / 2.0;
        let color = s.fill(i);
        let blur = (1.0 - p) * base * 0.12;
        if blur > 0.5 {
            for (dx, dy) in [(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)] {
                draw_text(ch, x + dx * blur, baseline + dy * blur, base, style::alpha(color, 0.22 * p * fade));
            }
        }
        draw_text(ch, x + base * 0.02, baseline + base * 0.04, base, style::alpha(BLACK, 0.12 * p * fade));
        draw_text(ch, x, baseline, base, style::alpha(color, p * fade));
    }
    if s.hot && since > 0.0 {
        let a = (since / 0.2).clamp(0.0, 1.0) * fade;
        centred("BACK-TO-BACK  X2", s.cx, baseline + base * 0.55, base * 0.34, style::alpha(pal.hot[1], a));
    }
}
