//! Bounce (Mac OS X): each letter on a glossy gel tile that drops in and bounces like an icon in
//! the dock, with a reflection under the row; the last one hits on the chord, and a magnifying
//! wave runs along the row. Back-to-back: a gel pill says so and a red gel dot says X2. Then the
//! tiles are sucked down to the bottom of the screen, one after another.

use macroquad::prelude::*;

use super::{centred, chars, ease_in_cubic, ease_out_bounce, ring, style, Scene, LETTERS, OUT, STAGGER};

/// Seconds a tile's drop and bounces take; its first touch down comes at 4/11 of that.
const DROP: f32 = 0.25;
/// Seconds a tile takes to be sucked away, and between one tile and the next.
const AWAY: f32 = 0.3;
const AWAY_STAGGER: f32 = 0.03;

pub(super) fn draw(s: &Scene) {
    let (t, pal, screen) = (s.t, s.pal, s.screen);
    s.dim(0.3);
    let gap_k = 0.08;
    let tile = (s.fit.size * 1.05).min(screen.w * 0.94 / (LETTERS as f32 + (LETTERS - 1) as f32 * gap_k));
    let gap = tile * gap_k;
    let row = LETTERS as f32 * tile + (LETTERS - 1) as f32 * gap;
    let left = s.cx - row / 2.0;
    let top = screen.y - tile * 1.5;
    let since = s.since();

    // The magnifying wave along the row after the chord.
    let wave = (since - 0.15) / 0.6;
    let wave_x = left + row * wave;

    for (i, ch) in chars().iter().enumerate() {
        let start = i as f32 * STAGGER;
        if t < start {
            continue;
        }
        let home = vec2(left + i as f32 * (tile + gap) + tile / 2.0, s.cy);
        let mut c = vec2(home.x, top + (home.y - top) * ease_out_bounce(((t - start) / DROP).clamp(0.0, 1.0)));
        let mut size = tile;
        if (0.0..1.0).contains(&wave) {
            let d = (home.x - wave_x) / (tile * 1.4);
            let grow = 0.28 * (-d * d).exp();
            size *= 1.0 + grow;
            c.y -= tile * grow * 0.5;
        }
        // Sucked away to the bottom of the screen.
        let away = ease_in_cubic(((t - OUT - i as f32 * AWAY_STAGGER) / AWAY).clamp(0.0, 1.0));
        if away >= 1.0 {
            continue;
        }
        if away > 0.0 {
            let sink = vec2(screen.center().x, screen.bottom());
            c += (sink - c) * away;
            size *= 1.0 - 0.85 * away;
        }
        let color = s.fill(i);
        // The reflection, then the tile.
        let landed = (t - start) / DROP >= 4.0 / 11.0;
        if away == 0.0 && landed {
            gel(vec2(c.x, home.y + tile + gap * 0.5), size, color, 0.16);
        }
        gel(c, size, color, 1.0);
        centred(ch, c.x + size * 0.03, c.y + size * 0.04, size * 0.66, style::alpha(BLACK, 0.35));
        centred(ch, c.x, c.y, size * 0.66, WHITE);
    }

    ring(s.cx, s.cy, since, 0.5, screen.w * 0.45, pal.rings[1], s.fit.size * 0.06);

    if s.hot && since > 0.0 && t < OUT + 0.2 {
        let p = ((since) / 0.2).clamp(0.0, 1.0);
        let fade = 1.0 - s.leaving(0.2);
        let size = s.fit.size * 0.38;
        let w = size * 7.5;
        let pill = Rect::new(s.cx - w / 2.0, s.cy - tile * 1.25 - size * 0.9 + (1.0 - p) * size, w, size * 1.8);
        gel_rect(pill, pill.h / 2.0, pal.banner, fade * p);
        centred("BACK-TO-BACK", pill.center().x, pill.center().y, size, style::alpha(WHITE, fade * p));
        let dot = vec2((left + row - tile * 0.2).min(screen.right() - tile * 0.45), s.cy - tile * 0.85);
        let r = tile * 0.4 * (1.0 + 0.6 * (1.0 - p));
        gel_rect(Rect::new(dot.x - r, dot.y - r, 2.0 * r, 2.0 * r), r, pal.hot[2], fade * p);
        centred("X2", dot.x, dot.y, r * 0.9, style::alpha(WHITE, fade * p));
    }
}

/// A square gel tile centred on `c`.
fn gel(c: Vec2, size: f32, color: Color, a: f32) {
    gel_rect(Rect::new(c.x - size / 2.0, c.y - size / 2.0, size, size), size * 0.24, color, a);
}

/// Aqua gel: deep at the top, glowing low down, a glossy highlight over the top half.
fn gel_rect(r: Rect, radius: f32, color: Color, a: f32) {
    if a <= 0.0 || r.w < 1.0 {
        return;
    }
    style::round_rect(r, radius, style::mix(color, BLACK, 0.25).with_alpha(a), style::mix(color, WHITE, 0.3).with_alpha(a));
    style::round_rect(
        Rect::new(r.x + r.w * 0.12, r.y + r.h * 0.5, r.w * 0.76, r.h * 0.42),
        radius * 0.7,
        style::alpha(style::mix(color, WHITE, 0.2), 0.0),
        style::alpha(style::mix(color, WHITE, 0.6), 0.8 * a),
    );
    style::round_rect(
        Rect::new(r.x + r.w * 0.08, r.y + r.h * 0.05, r.w * 0.84, r.h * 0.44),
        radius * 0.85,
        Color::new(1.0, 1.0, 1.0, 0.85 * a),
        Color::new(1.0, 1.0, 1.0, 0.1 * a),
    );
    style::round_outline(r, radius, (r.w * 0.03).max(1.0), style::mix(color, BLACK, 0.5).with_alpha(0.7 * a));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_last_tile_first_touches_down_on_the_chord() {
        let touch = (LETTERS - 1) as f32 * STAGGER + DROP * 4.0 / 11.0;
        assert!((touch - super::super::BANG).abs() < 0.01, "touch {touch}");
    }

    #[test]
    fn every_tile_is_gone_before_the_end() {
        let gone = OUT + (LETTERS - 1) as f32 * AWAY_STAGGER + AWAY;
        assert!(gone < crate::game::config::TETRIS_CELEBRATION_TIME as f32, "{gone}");
    }
}
