//! JONES'IN! (a Tetris). Each theme plays it its own way (graphics::theme::JonesinStyle), one
//! file per style. They share the timing here: the big moment lands at [`BANG`], with the
//! fanfare's chord (assets/sounds/jonesin.wav, its chord at 0.26 s), the word starts leaving at
//! [`OUT`], and every style is gone before the celebration ends.
//!
//! A back-to-back Tetris (the scoring's last clear says so) runs hot: each style shows it in its
//! own way, in the palette's hot colours.
//!
//! Everything runs on the game's celebration clock, so it pauses with the game.

mod bloom;
mod bounce;
mod dialog;
mod marquee;
mod plaque;
mod slam;
mod stamp;
mod typed;

use macroquad::prelude::*;

use super::{ease_in_cubic, ease_out_bounce, ease_out_cubic, hash01, style, FxPalette};
use crate::game::Game;
use crate::game::config::TETRIS_CELEBRATION_TIME;
use crate::graphics::text::{draw_text, measure_text};
use crate::graphics::theme::JonesinStyle;
use crate::graphics::view::Layout;
use crate::scoring::LineClearType;

const WORD: &str = "JONES'IN!";
/// The word's letters, in order.
const LETTERS: usize = WORD.len();
/// Seconds between one letter starting to fall and the next.
const STAGGER: f32 = 0.025;
/// Seconds a letter takes to fall.
const FALL: f32 = 0.09;
/// Seconds a letter's landing punch lasts.
const PUNCH: f32 = 0.14;
/// When the big moment lands (the !, for the letters that fall).
pub const BANG: f32 = (LETTERS - 1) as f32 * STAGGER + FALL;
/// When the word starts leaving.
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

/// When letter `i` lands (or appears, for the styles that don't drop their letters).
fn lands(i: usize) -> f32 {
    i as f32 * STAGGER + FALL
}

/// The board's kick when the big moment lands, as a direction scaled by how hard (up to 1).
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

/// One frame of the celebration, for a style to draw.
struct Scene<'a> {
    /// Seconds in.
    t: f32,
    /// Back-to-back.
    hot: bool,
    screen: Rect,
    board: Rect,
    /// Where the word's middle goes.
    cx: f32,
    cy: f32,
    /// The word fitted to the screen in the theme's font.
    fit: WordFit,
    pal: &'a FxPalette,
    /// The layout's unit (about a point).
    u: f32,
}

impl Scene<'_> {
    /// Seconds since the big moment (negative before it).
    fn since(&self) -> f32 {
        self.t - BANG
    }

    /// 0 until the word starts leaving, then up to 1 over `dur` seconds.
    fn leaving(&self, dur: f32) -> f32 {
        ((self.t - OUT) / dur).clamp(0.0, 1.0)
    }

    /// The letters' colours, left to right.
    fn stops(&self) -> &[Color; 3] {
        if self.hot {
            &self.pal.hot
        } else {
            &self.pal.word
        }
    }

    /// Letter `i`'s colour.
    fn fill(&self, i: usize) -> Color {
        gradient(self.stops(), i as f32 / (LETTERS - 1) as f32)
    }

    /// The middle of letter `i`'s slot, across, at the fitted size.
    fn letter_x(&self, i: usize) -> f32 {
        let left = self.cx - self.fit.total / 2.0;
        left + self.fit.widths[..i].iter().sum::<f32>() + self.fit.gap * i as f32 + self.fit.widths[i] / 2.0
    }

    /// Dim the screen behind the word, `strength` at most, while it's up.
    fn dim(&self, strength: f32) {
        let a = ((self.t - 0.1) / 0.15).clamp(0.0, 1.0) * (1.0 - ((self.t - OUT) / 0.4).clamp(0.0, 1.0));
        if a > 0.0 {
            let s = self.screen;
            draw_rectangle(s.x, s.y, s.w, s.h, style::alpha(self.pal.dim, strength * a));
        }
    }

    /// The screen flashes for a moment at the big moment.
    fn flash(&self, strength: f32) {
        let since = self.since();
        if (0.0..0.08).contains(&since) {
            let s = self.screen;
            draw_rectangle(s.x, s.y, s.w, s.h, style::alpha(self.pal.flash, strength * (1.0 - since / 0.08)));
        }
    }
}

/// The letters of the word, one string each.
fn chars() -> Vec<String> {
    WORD.chars().map(String::from).collect()
}

/// Draw it over everything, in screen pixels, the way the palette's theme does it.
pub fn draw(game: &Game, layout: &Layout, pal: &FxPalette) {
    if !game.is_tetris_celebration_active() {
        return;
    }
    let screen = layout.screen;
    let board = layout.play.board;
    // The word: as big as fits the screen (and no taller than a good part of the board).
    let fit = fit_word(screen.w, board.h);
    // Over the board when there's room, otherwise the middle of the screen.
    let (lo, hi) = (screen.x + fit.total / 2.0 + fit.margin, screen.right() - fit.total / 2.0 - fit.margin);
    let cx = if lo <= hi { board.center().x.clamp(lo, hi) } else { screen.center().x };
    let s = Scene {
        t: game.tetris_celebration_timer as f32,
        hot: back_to_back(game),
        screen,
        board,
        cx,
        cy: board.y + board.h * 0.42,
        fit,
        pal,
        u: layout.unit,
    };
    match pal.styles.jonesin {
        JonesinStyle::Slam => slam::draw(&s),
        JonesinStyle::Typed => typed::draw(&s),
        JonesinStyle::Shell => typed::draw_shell(&s),
        JonesinStyle::Marquee => marquee::draw(&s),
        JonesinStyle::Dialog => dialog::draw(&s),
        JonesinStyle::Bounce => bounce::draw(&s),
        JonesinStyle::Stamp => stamp::draw(&s),
        JonesinStyle::Plaque => plaque::draw(&s),
        JonesinStyle::Bloom => bloom::draw(&s),
    }
}

/// The word's size and letter widths, in the current font, fitted to a screen `screen_w` wide.
struct WordFit {
    size: f32,
    widths: Vec<f32>,
    gap: f32,
    total: f32,
    margin: f32,
}

fn fit_word(screen_w: f32, board_h: f32) -> WordFit {
    let measure = |size: f32| -> (Vec<f32>, f32, f32) {
        let widths: Vec<f32> = WORD.chars().map(|c| measure_text(&c.to_string(), None, size.max(1.0) as u16, 1.0).width).collect();
        let gap = size * 0.06;
        let total = widths.iter().sum::<f32>() + gap * (widths.len() - 1) as f32;
        (widths, gap, total)
    };
    let mut size = (screen_w * 0.9 / (WORD.len() as f32 * 0.62)).min(board_h * 0.12).max(12.0);
    let (mut widths, mut gap, mut total) = measure(size);
    // Wide fonts: shrink until the word and its margins fit (sizes round to whole pixels, so a
    // second pass catches what the first leaves over).
    for _ in 0..3 {
        let room = screen_w * 0.94 - 2.0 * size * 0.3;
        if total <= room || size <= 6.0 {
            break;
        }
        size = (size * room / total).floor().max(6.0);
        (widths, gap, total) = measure(size);
    }
    WordFit { size, widths, gap, total, margin: size * 0.3 }
}

fn gradient(stops: &[Color; 3], u: f32) -> Color {
    let s = u.clamp(0.0, 1.0) * 2.0;
    let i = (s.floor() as usize).min(1);
    style::mix(stops[i], stops[i + 1], s - i as f32)
}

/// Text centred on (`cx`, `cy`).
fn centred(text: &str, cx: f32, cy: f32, size: f32, color: Color) {
    let w = measure_text(text, None, size.max(1.0) as u16, 1.0).width;
    draw_text(text, cx - w / 2.0, cy + size * 0.35, size, color);
}

/// A shockwave: an ellipse growing out from (`cx`, `cy`) and fading, `since` seconds in.
fn ring(cx: f32, cy: f32, since: f32, dur: f32, radius: f32, color: Color, width: f32) {
    if !(0.0..dur).contains(&since) {
        return;
    }
    let p = since / dur;
    let r = radius * ease_out_cubic(p);
    draw_ellipse_lines(cx, cy, r, r * 0.55, 0.0, width * (1.0 - p) + 1.0, style::alpha(color, 1.0 - p));
}

/// Confetti falling from the top of the screen from the big moment.
fn confetti(s: &Scene) {
    let (from, until) = (BANG, TETRIS_CELEBRATION_TIME as f32 - 0.2);
    let (t, screen, pal) = (s.t, s.screen, s.pal);
    if t < from {
        return;
    }
    let colors = s.stops();
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

/// BACK-TO-BACK on a slanted bar that slides in above the word and off again.
fn back_to_back_banner(s: &Scene, cy: f32) {
    let (t, screen, base, pal, cx) = (s.t, s.screen, s.fit.size, s.pal, s.cx);
    let inn = ((t - 0.02) / 0.23).clamp(0.0, 1.0);
    let out = ((t - OUT) / 0.3).clamp(0.0, 1.0);
    if inn <= 0.0 || out >= 1.0 {
        return;
    }
    let x = (screen.x - base * 3.0) + (cx - (screen.x - base * 3.0)) * ease_out_cubic(inn) + ease_in_cubic(out) * screen.w;
    let (w, h) = (base * 4.6, base * 0.56);
    let skew = h * 0.35;
    let (l, r, top, bottom) = (x - w / 2.0, x + w / 2.0, cy - h / 2.0, cy + h / 2.0);
    draw_triangle(vec2(l + skew, top), vec2(r + skew, top), vec2(l - skew, bottom), pal.banner);
    draw_triangle(vec2(r + skew, top), vec2(r - skew, bottom), vec2(l - skew, bottom), pal.banner);
    centred("BACK-TO-BACK", x, cy, base * 0.36, pal.banner_ink);
}

/// The X2 stamp that thumps down at the big moment.
fn x2_stamp(s: &Scene, x: f32, y: f32) {
    let (t, base, pal) = (s.t, s.fit.size, s.pal);
    let p = ((t - BANG) / 0.14).clamp(0.0, 1.0);
    if p <= 0.0 {
        return;
    }
    let a = (p * 2.0).min(1.0) * (1.0 - ((t - OUT) / 0.3).clamp(0.0, 1.0));
    let scale = 2.4 + (1.0 - 2.4) * ease_out_cubic(p);
    let r = base * 0.42 * scale;
    draw_circle(x, y, r, style::alpha(pal.stamp, a));
    draw_circle_lines(x, y, r, (base * 0.06).max(2.0), style::alpha(pal.banner_ink, a));
    centred("X2", x, y, base * 0.4 * scale, style::alpha(pal.banner_ink, a));
}

/// A block cursor after text, blinking once typing stops.
pub(super) fn cursor(x: f32, baseline: f32, size: f32, color: Color, typing: bool, t: f32) {
    if typing || (t * 3.0).fract() < 0.5 {
        draw_rectangle(x, baseline - size * 0.7, size * 0.5, size * 0.8, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bang_lands_with_the_fanfare_chord() {
        // The fanfare's chord comes in at 0.26 s (tools/jonesin_fanfare.py).
        assert!((BANG - 0.26).abs() < 0.05, "BANG {BANG}");
        assert_eq!(lands(LETTERS - 1), BANG, "the last letter lands on the chord");
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
    fn the_word_fits_the_screen_in_every_font() {
        use crate::graphics::text::{set_face, Face};
        for face in Face::ALL {
            set_face(face);
            for w in [320.0, 390.0, 430.0, 458.0, 568.0, 744.0, 1024.0] {
                let fit = fit_word(w, w * 1.6);
                assert!(fit.total + 2.0 * fit.margin <= w, "{face:?} at {w}: word {} + margins {} > screen", fit.total, fit.margin);
            }
        }
        set_face(Face::Proggy);
    }

    #[test]
    fn all_gone_before_the_celebration_ends() {
        let end = TETRIS_CELEBRATION_TIME as f32;
        for i in 0..WORD.len() {
            assert!(letter(i, end).unwrap().alpha <= 0.0, "letter {i}");
        }
        assert!(OUT + 0.45 < end, "every style's exit fits before the end");
    }

    #[test]
    fn every_theme_picks_a_style_that_draws() {
        // Each theme names one; the match in draw covers them all (a compile error otherwise).
        // The default stays the slam.
        assert_eq!(crate::graphics::theme::DEFAULT_STYLES.jonesin, JonesinStyle::Slam);
        let picks: std::collections::HashSet<_> = crate::graphics::theme::ALL
            .iter()
            .map(|&id| format!("{:?}", crate::graphics::theme::get(id).fx.styles.jonesin))
            .collect();
        assert_eq!(picks.len(), crate::graphics::theme::ALL.len(), "each theme has its own JONES'IN!");
    }
}
