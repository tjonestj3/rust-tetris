//! Dialog (Windows 98): outlines zoom out from the board like a window opening, and on the chord
//! the message box is there: an information icon, JONES'IN! and an OK button. Then it bounces off
//! across the screen, leaving a trail of copies of itself behind, and it's all gone at once when
//! the celebration ends. Back-to-back: the box says so, and its title too.

use macroquad::prelude::*;

use super::{centred, ease_out_cubic, style, Scene, BANG, OUT};
use crate::graphics::text::measure_text;

/// When the box starts bouncing after it opens, how often it leaves a copy, and the most kept.
const BOUNCE_AT: f32 = 0.12;
const TRAIL_STEP: f32 = 0.03;
const TRAIL_MAX: usize = 40;

pub(super) fn draw(s: &Scene) {
    let t = s.t;
    if t >= OUT + 0.05 {
        return;
    }
    let base = s.fit.size * 0.72;
    let small = (s.fit.size * 0.3).max(11.0);
    let word_w = measure_text("JONES'IN!", None, base.max(1.0) as u16, 1.0).width;
    let icon = base * 1.0;
    let w = (word_w + icon + base * 1.6).min(s.screen.w * 0.94);
    let h = small * 1.6 + base * 1.3 + small * 2.6 + if s.hot { small * 1.4 } else { 0.0 };
    let home = vec2(s.cx - w / 2.0, s.cy - h / 2.0);

    // Opening: outlines zooming from the board's middle out to where the box goes.
    if s.since() < 0.0 {
        let from = Rect::new(s.board.center().x - w * 0.05, s.board.center().y - h * 0.05, w * 0.1, h * 0.1);
        let p = (t / BANG).clamp(0.0, 1.0);
        for k in 0..4 {
            let q = ease_out_cubic((p - k as f32 * 0.12).clamp(0.0, 1.0));
            if q <= 0.0 {
                continue;
            }
            let r = lerp_rect(from, Rect::new(home.x, home.y, w, h), q);
            // Light outlines (they open over the dark board), the newest brightest.
            draw_rectangle_lines(r.x, r.y, r.w, r.h, (2.0 * s.u).max(1.0), style::alpha(s.pal.flash, 0.5 + 0.12 * k as f32));
        }
        return;
    }

    // The trail: where the box was every TRAIL_STEP since it started bouncing, then the box.
    let bounce = (s.since() - BOUNCE_AT).max(0.0);
    let copies = ((bounce / TRAIL_STEP) as usize).min(TRAIL_MAX);
    for p in path(home, vec2(w, h), s.screen, bounce, copies) {
        draw_box(s, Rect::new(p.x, p.y, w, h), base, small, icon);
    }
}

/// Where the box's corner is at `copies + 1` moments `TRAIL_STEP` apart, ending `bounce` seconds
/// after it set off from `home`: off to the right and up, falling, bouncing off the floor and
/// the sides.
fn path(home: Vec2, size: Vec2, screen: Rect, bounce: f32, copies: usize) -> Vec<Vec2> {
    let mut want: Vec<f32> = (0..copies).map(|k| bounce - (copies - k) as f32 * TRAIL_STEP).collect();
    want.push(bounce);
    let (mut p, mut v) = (home, vec2(screen.w * 0.7, -screen.h * 0.55));
    let g = screen.h * 3.2;
    let dt = 1.0 / 240.0;
    let mut now = 0.0;
    let mut out = Vec::with_capacity(want.len());
    for at in want {
        while now + dt <= at {
            v.y += g * dt;
            p += v * dt;
            if p.y + size.y > screen.bottom() {
                p.y = screen.bottom() - size.y;
                v.y = -v.y.abs() * 0.78;
            }
            if p.x + size.x > screen.right() {
                p.x = screen.right() - size.x;
                v.x = -v.x.abs();
            } else if p.x < screen.x {
                p.x = screen.x;
                v.x = v.x.abs();
            }
            now += dt;
        }
        out.push(p);
    }
    out
}

fn lerp_rect(a: Rect, b: Rect, k: f32) -> Rect {
    Rect::new(a.x + (b.x - a.x) * k, a.y + (b.y - a.y) * k, a.w + (b.w - a.w) * k, a.h + (b.h - a.h) * k)
}

/// The message box at `r`.
fn draw_box(s: &Scene, r: Rect, base: f32, small: f32, icon: f32) {
    let pal = s.pal;
    let title = if s.hot { "Rust Tetris - Back-to-back" } else { "Rust Tetris" };
    let inner = match pal.window {
        Some(window) => window(r, title, pal.card_edge, small, s.u),
        None => {
            (pal.panel)(r, pal.card_edge, s.u);
            r
        }
    };
    // The information icon: a blue circle with an i.
    let ic = vec2(inner.x + base * 0.4 + icon / 2.0, inner.y + base * 0.35 + icon / 2.0);
    let blue = pal.card_edge;
    draw_circle(ic.x + s.u, ic.y + s.u * 1.5, icon / 2.0, style::alpha(BLACK, 0.4));
    draw_circle(ic.x, ic.y, icon / 2.0, WHITE);
    draw_circle(ic.x, ic.y, icon / 2.0 - (1.5 * s.u).max(1.0), blue);
    centred("i", ic.x, ic.y, icon * 0.75, WHITE);

    let tx = ic.x + icon / 2.0 + base * 0.35;
    let word_w = measure_text("JONES'IN!", None, base.max(1.0) as u16, 1.0).width;
    centred("JONES'IN!", tx + word_w / 2.0, ic.y, base, pal.card_ink);
    let mut y = ic.y + icon / 2.0 + small * 0.6;
    if s.hot {
        let line = "Back-to-back!  Score x2";
        let lw = measure_text(line, None, small as u16, 1.0).width;
        centred(line, tx + lw / 2.0, y + small * 0.5, small, pal.card_ink);
        y += small * 1.4;
    }
    // OK.
    let (bw, bh) = (small * 5.0, small * 1.8);
    let b = Rect::new(inner.center().x - bw / 2.0, y + small * 0.3, bw, bh);
    (pal.panel)(b, pal.card_edge, s.u);
    draw_rectangle_lines(b.x + 3.0 * s.u, b.y + 3.0 * s.u, b.w - 6.0 * s.u, b.h - 6.0 * s.u, 1.0, style::alpha(pal.card_ink, 0.6));
    centred("OK", b.center().x, b.center().y, small, pal.card_ink);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_trail_stays_on_screen_and_ends_where_the_box_is() {
        let screen = Rect::new(0.0, 0.0, 430.0, 932.0);
        let size = vec2(300.0, 160.0);
        let home = vec2(65.0, 300.0);
        let pts = path(home, size, screen, 1.0, 30);
        assert_eq!(pts.len(), 31);
        for p in &pts {
            assert!(p.x >= 0.0 && p.x + size.x <= 430.0 + 0.01, "{p}");
            assert!(p.y + size.y <= 932.0 + 0.01, "{p}");
        }
        assert_eq!(path(home, size, screen, 0.0, 0), vec![home]);
    }
}
