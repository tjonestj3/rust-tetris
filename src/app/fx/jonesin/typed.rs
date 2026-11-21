//! Typed (Terminal): a command is typed at a prompt, then the word in big block text with a
//! cursor; its last letter lands on the chord, the word flashes inverted and scanlines roll down
//! the screen. Back-to-back prints a second line. Then it's deleted, letter by letter.
//!
//! Shell (Linux): the same in a terminal window that pops up: a coloured prompt, the command,
//! the word printed big.

use macroquad::prelude::*;

use super::{chars, cursor, ease_in_cubic, ring, style, Scene, BANG, LETTERS, OUT};
use crate::graphics::text::{draw_text, measure_text};

/// The command is typed by then; the word's letters follow, the last at [`BANG`].
const COMMAND_TYPED: f32 = 0.1;
/// How long deleting the word takes.
const DELETE: f32 = 0.3;

/// How many of `len` characters are showing: typed in from `from` to `to`, deleted from OUT.
fn shown(t: f32, from: f32, to: f32, len: usize) -> usize {
    let typed = (((t - from) / (to - from)).clamp(0.0, 1.0) * len as f32).floor() as usize;
    let deleted = (((t - OUT) / DELETE).clamp(0.0, 1.0) * len as f32).ceil() as usize;
    typed.min(len.saturating_sub(deleted))
}

/// The word's letters showing now, the last landing on the chord.
fn word_shown(t: f32) -> usize {
    shown(t, COMMAND_TYPED, BANG, LETTERS)
}

fn draw_prefix(text: &str, x: f32, baseline: f32, size: f32, len: usize, color: Color) -> f32 {
    let part: String = text.chars().take(len).collect();
    draw_text(&part, x, baseline, size, color);
    x + measure_text(&part, None, size.max(1.0) as u16, 1.0).width
}

pub(super) fn draw(s: &Scene) {
    let (t, pal, base) = (s.t, s.pal, s.fit.size);
    s.dim(0.7);
    let left = s.cx - s.fit.total / 2.0;
    let small = (base * 0.34).max(10.0);
    let baseline = s.cy + base * 0.35;
    let fade = 1.0 - s.leaving(DELETE);

    // The command.
    let command = "> tetris --lines 4";
    let n = shown(t, 0.0, COMMAND_TYPED, command.len()).max(if t < OUT { 0 } else { command.len() });
    draw_prefix(command, left, s.cy - base * 0.75, small, n, style::alpha(pal.word[0], 0.7 * fade));

    // The word, inverted for a moment when it's done.
    let shown = word_shown(t);
    let inverted = (0.0..0.12).contains(&s.since());
    if inverted {
        draw_rectangle(left - base * 0.1, s.cy - base * 0.55, s.fit.total + base * 0.2, base * 1.05, pal.word[0]);
    }
    for (i, ch) in chars().iter().enumerate().take(shown) {
        let w = s.fit.widths[i];
        let color = if inverted { pal.outline } else { s.fill(i) };
        draw_text(ch, s.letter_x(i) - w / 2.0, baseline, base, color);
    }
    let after = if shown == 0 { left } else { s.letter_x(shown - 1) + s.fit.widths[shown - 1] / 2.0 + s.fit.gap };
    if t < OUT + DELETE {
        cursor(after, baseline, base, pal.word[0], s.since() < 0.0 || t > OUT, t);
    }

    // Back-to-back: a second line.
    if s.hot && s.since() > 0.0 {
        let line = ">> BACK-TO-BACK  x2";
        let n = shown_line(s, line.len());
        draw_prefix(line, left, s.cy + base * 1.05, small * 1.2, n, style::alpha(pal.hot[1], fade));
    }

    scanlines(s);
}

/// A follow-up line's characters, typed in over 0.2 s after the chord.
fn shown_line(s: &Scene, len: usize) -> usize {
    shown(s.t, BANG, BANG + 0.2, len)
}

/// The chord: thin bright lines over the whole screen and a bright band rolling down it.
fn scanlines(s: &Scene) {
    let since = s.since();
    if !(0.0..0.35).contains(&since) {
        return;
    }
    let p = since / 0.35;
    let screen = s.screen;
    let step = (screen.h / 240.0).max(3.0).round();
    let line = style::alpha(s.pal.flash, 0.16 * (1.0 - p));
    let mut y = screen.y;
    while y < screen.bottom() {
        draw_rectangle(screen.x, y, screen.w, 1.0, line);
        y += step;
    }
    let band = screen.h * 0.08;
    let by = screen.y - band + (screen.h + band) * p;
    style::gradient_rect(Rect::new(screen.x, by, screen.w, band), style::alpha(s.pal.flash, 0.0), style::alpha(s.pal.flash, 0.22 * (1.0 - p)));
}

/// The prompt's colours: Ubuntu's green name and blue path.
const PROMPT_NAME: Color = Color::new(0.54, 0.89, 0.2, 1.0);
const PROMPT_PATH: Color = Color::new(0.45, 0.62, 0.81, 1.0);

pub(super) fn draw_shell(s: &Scene) {
    let (t, pal, base) = (s.t, s.pal, s.fit.size);
    s.dim(0.35);
    let small = (base * 0.34).max(10.0);
    let lines = if s.hot { 2.0 } else { 1.0 };
    let w = (s.fit.total + base * 1.2).min(s.screen.w * 0.96);
    let h = small * 1.65 + small * 1.9 + base * 1.35 + if s.hot { small * 1.6 } else { 0.0 } + small * 0.6 * lines;
    let full = Rect::new(s.cx - w / 2.0, s.cy - h * 0.45, w, h);

    // Pops up, and later shrinks away.
    let open = (t / 0.08).clamp(0.0, 1.0);
    let close = ease_in_cubic(s.leaving(0.22));
    let k = (0.9 + 0.1 * open) * (1.0 - close);
    if k <= 0.02 {
        return;
    }
    let r = Rect::new(full.center().x - full.w * k / 2.0, full.center().y - full.h * k / 2.0, full.w * k, full.h * k);
    let inner = match pal.window {
        Some(window) => window(r, "Terminal", pal.word[0], small, s.u),
        None => {
            (pal.panel)(r, pal.word[0], s.u);
            r
        }
    };
    if open < 1.0 || close > 0.0 {
        return;
    }
    draw_rectangle(inner.x, inner.y, inner.w, inner.h, pal.edge);

    // The prompt and the command.
    let x = inner.x + small * 0.6;
    let line1 = inner.y + small * 1.5;
    let mut cx = draw_prefix("jones@tetris", x, line1, small, 99, PROMPT_NAME);
    cx = draw_prefix(":", cx, line1, small, 99, WHITE);
    cx = draw_prefix("~", cx, line1, small, 99, PROMPT_PATH);
    cx = draw_prefix("$ ", cx, line1, small, 99, WHITE);
    let command = "tetris --jonesin";
    let n = shown(t, 0.0, COMMAND_TYPED, command.len());
    let end = draw_prefix(command, cx, line1, small, n, WHITE);
    let shown = word_shown(t);
    if shown == 0 {
        cursor(end + small * 0.1, line1, small, WHITE, true, t);
    }

    // The word, printed big.
    let left = inner.x + (inner.w - s.fit.total) / 2.0;
    let baseline = line1 + small * 0.6 + base;
    let mut after = left;
    for (i, ch) in chars().iter().enumerate().take(shown) {
        let lx = left + s.fit.widths[..i].iter().sum::<f32>() + s.fit.gap * i as f32;
        draw_text(ch, lx, baseline, base, s.fill(i));
        after = lx + s.fit.widths[i] + s.fit.gap;
    }
    if shown > 0 && !s.hot {
        cursor(after, baseline, base * 0.6, WHITE, s.since() < 0.0, t);
    }
    if s.hot && s.since() > 0.0 {
        let line = "BACK-TO-BACK  x2";
        let n = shown_line(s, line.len());
        let end = draw_prefix(line, x, baseline + small * 1.8, small * 1.1, n, pal.hot[0]);
        cursor(end + small * 0.1, baseline + small * 1.8, small * 1.1, WHITE, n < line.len(), t);
    }

    // The chord: the window flashes, a ring goes out.
    let since = s.since();
    if (0.0..0.1).contains(&since) {
        draw_rectangle(inner.x, inner.y, inner.w, inner.h, style::alpha(pal.flash, 0.35 * (1.0 - since / 0.1)));
    }
    ring(s.cx, s.cy, since, 0.5, s.screen.w * 0.5, pal.rings[0], base * 0.08);
}
