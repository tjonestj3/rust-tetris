//! JONES'IN! (a Tetris): the letters slam down one after another, each landing with a punch and a
//! white flash, with a raised edge and a thick outline; when the ! lands the board kicks, the
//! screen flashes, two shockwaves ring out and confetti falls. Then the letters drop away.
//!
//! A back-to-back Tetris (the scoring's last clear says so) runs hot: gold-to-pink letters that
//! shimmer, heat rising behind them, a BACK-TO-BACK banner and an X2 stamp.
//!
//! Everything runs on the game's celebration clock, so it pauses with the game. The letters land
//! with the fanfare (assets/sounds/jonesin.wav): its chord at 0.26 s meets the ! at [`BANG`].

use macroquad::prelude::*;

use super::{ease_in_cubic, ease_out_cubic, hash01, style, FxPalette};
use crate::game::Game;
use crate::game::config::TETRIS_CELEBRATION_TIME;
use crate::graphics::text::{draw_text, measure_text};
use crate::graphics::view::Layout;
use crate::scoring::LineClearType;

const WORD: &str = "JONES'IN!";
/// Seconds between one letter starting to fall and the next.
const STAGGER: f32 = 0.025;
/// Seconds a letter takes to fall.
const FALL: f32 = 0.09;
/// Seconds a letter's landing punch lasts.
const PUNCH: f32 = 0.14;
/// When the ! lands.
pub const BANG: f32 = (WORD.len() - 1) as f32 * STAGGER + FALL;
/// When the letters start dropping away, one after another.
const OUT: f32 = 1.45;
const OUT_STAGGER: f32 = 0.03;
const OUT_TIME: f32 = 0.3;

/// One letter at a moment of the celebration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Letter {
    /// 0 = landed; 1 = still at the top of the screen (fraction of the way left to fall).
    pub above: f32,
    /// Below where it landed, as it drops away (fraction of the screen height).
    pub below: f32,
    pub scale: f32,
    /// 0..1: the white flash as it lands.
    pub flash: f32,
    pub alpha: f32,
}

/// Letter `i` at `t` seconds into the celebration, or `None` before it appears.
pub fn letter(i: usize, t: f32) -> Option<Letter> {
    let start = i as f32 * STAGGER;
    if t < start {
        return None;
    }
    let mut l = Letter { above: 0.0, below: 0.0, scale: 1.0, flash: 0.0, alpha: 1.0 };
    let f = (t - start) / FALL;
    if f < 1.0 {
        l.above = 1.0 - ease_in_cubic(f);
        l.scale = 1.1;
    } else {
        let k = ((t - start - FALL) / PUNCH).clamp(0.0, 1.0);
        l.scale = 1.0 + 0.28 * (1.0 - ease_out_cubic(k));
        l.flash = 1.0 - k;
    }
    let out = OUT + i as f32 * OUT_STAGGER;
    if t > out {
        let o = ((t - out) / OUT_TIME).clamp(0.0, 1.0);
        l.below = ease_in_cubic(o) * 0.7;
        l.alpha = 1.0 - o;
    }
    Some(l)
}

/// The board's kick when the ! lands, as a direction scaled by how hard (up to 1).
pub fn shake(game: &Game) -> Vec2 {
    if !game.is_tetris_celebration_active() {
        return Vec2::ZERO;
    }
    let d = game.tetris_celebration_timer as f32 - BANG;
    if !(0.0..0.35).contains(&d) {
        return Vec2::ZERO;
    }
    let e = (-d * 11.0).exp();
    vec2((d * 63.0).sin(), (d * 51.0).cos() * 0.6) * e
}

/// Whether the Tetris being celebrated was back-to-back.
fn back_to_back(game: &Game) -> bool {
    game.scoring_system
        .last_clear()
        .is_some_and(|c| c.kind == LineClearType::Tetris && c.back_to_back)
}

/// Draw it over everything, in screen pixels.
pub fn draw(game: &Game, layout: &Layout, pal: &FxPalette) {
    if !game.is_tetris_celebration_active() {
        return;
    }
    let t = game.tetris_celebration_timer as f32;
    let hot = back_to_back(game);
    let screen = layout.screen;
    let board = layout.play.board;

    // The word: as big as fits the screen (and no taller than a good part of the board).
    let chars: Vec<String> = WORD.chars().map(String::from).collect();
    let base = (screen.w * 0.9 / (chars.len() as f32 * 0.62)).min(board.h * 0.12).max(12.0);
    let widths: Vec<f32> = chars.iter().map(|c| measure_text(c, None, base as u16, 1.0).width).collect();
    let gap = base * 0.06;
    let total: f32 = widths.iter().sum::<f32>() + gap * (chars.len() - 1) as f32;
    let margin = base * 0.3;
    let cx = (board.center().x).clamp(screen.x + total / 2.0 + margin, screen.right() - total / 2.0 - margin);
    let cy = board.y + board.h * 0.42;

    // Dim the board behind the word while it's up.
    let dim = ((t - 0.1) / 0.15).clamp(0.0, 1.0) * (1.0 - ((t - OUT) / 0.4).clamp(0.0, 1.0));
    if dim > 0.0 {
        draw_rectangle(screen.x, screen.y, screen.w, screen.h, style::alpha(pal.dim, 0.35 * dim));
    }
    if hot {
        draw_heat(screen, cx, cy, total, t, pal);
    }

    let stops = if hot { pal.hot } else { pal.word };
    let mut x = cx - total / 2.0;
    for (i, ch) in chars.iter().enumerate() {
        let w = widths[i];
        let centre = x + w / 2.0;
        x += w + gap;
        let Some(l) = letter(i, t) else { continue };
        let u = i as f32 / (chars.len() - 1) as f32;
        let fill = gradient(&stops, u);
        let size = base * l.scale;
        let wobble = if hot { (t * 40.0 + i as f32).sin() * base * 0.015 } else { 0.0 };
        let top = screen.y - base;
        let y = cy + (top - cy) * l.above + screen.h * l.below;
        draw_letter(ch, centre + wobble, y, size, fill, l, pal);
    }

    // The ! lands: flash, shockwaves, confetti.
    let since = t - BANG;
    if (0.0..0.08).contains(&since) {
        draw_rectangle(screen.x, screen.y, screen.w, screen.h, style::alpha(pal.flash, 0.3 * (1.0 - since / 0.08)));
    }
    ring(cx, cy, since, 0.55, screen.w * 0.5, pal.rings[0], base * 0.12);
    ring(cx, cy, since - 0.08, 0.55, screen.w * 0.38, pal.rings[1], base * 0.08);
    confetti(screen, t, pal, if hot { &pal.hot } else { &pal.word });

    if hot {
        draw_banner(cx, cy - base * 1.05, base, t, screen, pal);
        let x = (cx + total / 2.0 + base * 0.35).min(screen.right() - base * 0.45);
        draw_stamp(x, cy + base * 0.75, base, t, pal);
    }
}

fn gradient(stops: &[Color; 3], u: f32) -> Color {
    let s = u.clamp(0.0, 1.0) * 2.0;
    let i = (s.floor() as usize).min(1);
    style::mix(stops[i], stops[i + 1], s - i as f32)
}

fn draw_letter(ch: &str, cx: f32, cy: f32, size: f32, fill: Color, l: Letter, pal: &FxPalette) {
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

fn ring(cx: f32, cy: f32, since: f32, dur: f32, radius: f32, color: Color, width: f32) {
    if !(0.0..dur).contains(&since) {
        return;
    }
    let p = since / dur;
    let r = radius * ease_out_cubic(p);
    draw_ellipse_lines(cx, cy, r, r * 0.55, 0.0, width * (1.0 - p) + 1.0, style::alpha(color, 1.0 - p));
}

fn confetti(screen: Rect, t: f32, pal: &FxPalette, colors: &[Color; 3]) {
    let (from, until) = (BANG, TETRIS_CELEBRATION_TIME as f32 - 0.2);
    if t < from {
        return;
    }
    let palette = [colors[0], colors[1], colors[2], pal.flash, pal.stamp];
    for i in 0..26u32 {
        let delay = hash01(i, 1) * 0.5;
        let age = t - from - delay;
        if age < 0.0 {
            continue;
        }
        let x0 = screen.x + hash01(i, 2) * screen.w;
        let speed = screen.h * (0.18 + hash01(i, 3) * 0.2);
        let y = screen.y - 20.0 + speed * age + screen.h * 0.12 * age * age;
        let x = x0 + (age * 3.0 + i as f32).sin() * screen.w * 0.02;
        let size = (screen.w * 0.012).max(4.0) * (0.8 + hash01(i, 4) * 0.6);
        let a = if t > until { (1.0 - (t - until) / 0.2).max(0.0) } else { 1.0 };
        let color = palette[(hash01(i, 5) * palette.len() as f32) as usize % palette.len()];
        draw_rectangle_ex(x, y, size, size, DrawRectangleParams {
            offset: vec2(0.5, 0.5),
            rotation: (hash01(i, 6) - 0.5) * 6.0 * age,
            color: style::alpha(color, a),
        });
    }
}

fn draw_heat(screen: Rect, cx: f32, cy: f32, total: f32, t: f32, pal: &FxPalette) {
    let a = ((t - 0.0) / 0.3).clamp(0.0, 1.0) * (1.0 - ((t - OUT) / 0.45).clamp(0.0, 1.0));
    if a <= 0.0 {
        return;
    }
    for i in 0..16u32 {
        let x = cx - total * 0.55 + hash01(i, 11) * total * 1.1;
        let speed = 0.35 + hash01(i, 12) * 0.4;
        let phase = hash01(i, 13);
        let rise = ((t * speed) + phase).fract();
        let y = cy + screen.h * 0.12 - rise * screen.h * 0.4;
        let h = screen.h * 0.05;
        let w = (screen.w * 0.006).max(2.0) * (1.0 + hash01(i, 14));
        for s in 0..4 {
            let k = s as f32 / 4.0;
            draw_rectangle(x, y - h + k * h, w, h / 4.0 + 0.5, style::alpha(pal.heat, 0.35 * a * k));
        }
    }
}

fn draw_banner(cx: f32, cy: f32, base: f32, t: f32, screen: Rect, pal: &FxPalette) {
    let inn = ((t - 0.02) / 0.23).clamp(0.0, 1.0);
    let out = ((t - OUT) / 0.3).clamp(0.0, 1.0);
    if inn <= 0.0 || out >= 1.0 {
        return;
    }
    let x = (screen.x - base * 3.0) + (cx - (screen.x - base * 3.0)) * ease_out_cubic(inn) + ease_in_cubic(out) * screen.w;
    let (w, h) = (base * 4.6, base * 0.56);
    let skew = h * 0.35;
    // A slanted bar: two triangles.
    let (l, r, top, bottom) = (x - w / 2.0, x + w / 2.0, cy - h / 2.0, cy + h / 2.0);
    draw_triangle(vec2(l + skew, top), vec2(r + skew, top), vec2(l - skew, bottom), pal.banner);
    draw_triangle(vec2(r + skew, top), vec2(r - skew, bottom), vec2(l - skew, bottom), pal.banner);
    let label = "BACK-TO-BACK";
    let size = base * 0.36;
    let lw = measure_text(label, None, size as u16, 1.0).width;
    draw_text(label, x - lw / 2.0, cy + size * 0.35, size, pal.banner_ink);
}

fn draw_stamp(x: f32, y: f32, base: f32, t: f32, pal: &FxPalette) {
    let p = ((t - BANG) / 0.14).clamp(0.0, 1.0);
    if p <= 0.0 {
        return;
    }
    let a = (p * 2.0).min(1.0) * (1.0 - ((t - OUT) / 0.3).clamp(0.0, 1.0));
    let scale = 2.4 + (1.0 - 2.4) * ease_out_cubic(p);
    let r = base * 0.42 * scale;
    draw_circle(x, y, r, style::alpha(pal.stamp, a));
    draw_circle_lines(x, y, r, (base * 0.06).max(2.0), style::alpha(pal.banner_ink, a));
    let size = base * 0.4 * scale;
    let w = measure_text("X2", None, size as u16, 1.0).width;
    draw_text("X2", x - w / 2.0, y + size * 0.35, size, style::alpha(pal.banner_ink, a));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bang_lands_with_the_fanfare_chord() {
        // The fanfare's chord comes in at 0.26 s (tools/jonesin_fanfare.py).
        assert!((BANG - 0.26).abs() < 0.05, "BANG {BANG}");
    }

    #[test]
    fn letters_fall_in_order_and_land() {
        assert_eq!(letter(1, 0.0), None, "the second letter waits its turn");
        assert!(letter(0, 0.0).unwrap().above > 0.9);
        for i in 0..WORD.len() {
            let l = letter(i, BANG + PUNCH).unwrap();
            assert_eq!(l.above, 0.0, "letter {i} has landed");
            assert!((l.scale - 1.0).abs() < 1e-3);
        }
    }

    #[test]
    fn all_gone_before_the_celebration_ends() {
        let end = TETRIS_CELEBRATION_TIME as f32;
        for i in 0..WORD.len() {
            assert!(letter(i, end).unwrap().alpha <= 0.0, "letter {i}");
        }
    }
}
