//! The look of the touch screens: a bright gradient backdrop, rounded glass panels with a soft
//! glow, glossy blocks. Everything here draws in screen pixels; sizes are passed in pixels.

use macroquad::models::{draw_mesh, Mesh, Vertex};
use macroquad::prelude::*;

pub const CYAN: Color = Color::new(0.30, 0.90, 1.00, 1.0);
pub const PINK: Color = Color::new(1.00, 0.38, 0.78, 1.0);
pub const GOLD: Color = Color::new(1.00, 0.82, 0.28, 1.0);
pub const GREEN: Color = Color::new(0.45, 1.00, 0.55, 1.0);
pub const VIOLET: Color = Color::new(0.68, 0.52, 1.00, 1.0);
pub const INK: Color = Color::new(1.00, 1.00, 1.00, 0.97);
pub const SOFT_INK: Color = Color::new(0.82, 0.84, 1.00, 0.85);

/// `c` with its alpha multiplied by `a`.
pub fn alpha(c: Color, a: f32) -> Color {
    Color::new(c.r, c.g, c.b, c.a * a)
}

/// `a` blended toward `b` by `t`.
pub fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

/// The backdrop over the whole screen (default camera): a violet-to-magenta gradient, two slow
/// drifting glows (cyan and pink), a faint grid and a few twinkles.
pub fn backdrop(time: f32) {
    let (w, h) = (screen_width(), screen_height());
    clear_background(Color::new(0.08, 0.05, 0.22, 1.0));
    gradient_rect(
        Rect::new(0.0, 0.0, w, h),
        Color::new(0.09, 0.07, 0.30, 1.0),
        Color::new(0.30, 0.07, 0.40, 1.0),
    );

    let span = w.max(h);
    let t = time * 0.15;
    glow(w * (0.15 + 0.08 * t.sin()), h * (0.12 + 0.05 * (t * 0.7).cos()), span * 0.55, CYAN, 0.22);
    glow(w * (0.90 - 0.08 * (t * 0.8).cos()), h * (0.88 - 0.05 * t.sin()), span * 0.60, PINK, 0.24);
    glow(w * 0.5, h * 0.5, span * 0.45, VIOLET, 0.10);

    // A faint grid, on whole pixels so it stays crisp.
    let step = (span / 14.0).round().max(8.0);
    let line = Color::new(0.75, 0.80, 1.00, 0.06);
    let mut x = ((w % step) / 2.0).round();
    while x < w {
        draw_rectangle(x, 0.0, 1.0, h, line);
        x += step;
    }
    let mut y = ((h % step) / 2.0).round();
    while y < h {
        draw_rectangle(0.0, y, w, 1.0, line);
        y += step;
    }

    // Twinkles on grid crossings, fading in and out at their own pace.
    for i in 0..18u32 {
        let fx = ((i * 7919) % 97) as f32 / 97.0;
        let fy = ((i * 104729) % 89) as f32 / 89.0;
        let a = ((time * (0.6 + (i % 5) as f32 * 0.15) + i as f32 * 1.7).sin() * 0.5 + 0.5).powi(3);
        let s = (step * 0.08).max(2.0);
        let (px, py) = ((fx * w / step).floor() * step + (w % step) / 2.0, (fy * h / step).floor() * step + (h % step) / 2.0);
        draw_rectangle(px - s / 2.0, py - s / 2.0, s, s, Color::new(1.0, 1.0, 1.0, 0.55 * a));
    }
}

/// A soft round light: stacked translucent discs, brightest in the middle.
pub fn glow(x: f32, y: f32, radius: f32, color: Color, strength: f32) {
    const RINGS: usize = 14;
    for i in 0..RINGS {
        let f = 1.0 - i as f32 / RINGS as f32;
        draw_poly(x, y, 64, radius * f, 0.0, alpha(color, strength / RINGS as f32 * 1.6));
    }
}

/// A rectangle shaded from `top` to `bottom`.
pub fn gradient_rect(r: Rect, top: Color, bottom: Color) {
    let v = |x: f32, y: f32, c: Color| Vertex::new(x, y, 0.0, 0.0, 0.0, c);
    draw_mesh(&Mesh {
        vertices: vec![v(r.x, r.y, top), v(r.right(), r.y, top), v(r.right(), r.bottom(), bottom), v(r.x, r.bottom(), bottom)],
        indices: vec![0, 1, 2, 0, 2, 3],
        texture: None,
    });
}

/// Points per corner for a corner of `radius` pixels: smooth, but no more than needed.
fn corner_segments(radius: f32) -> usize {
    ((radius / 2.0).ceil() as usize).clamp(2, 10)
}

/// The outline of a rounded rectangle, clockwise from the top-right corner, `segs` steps a corner.
fn round_path(r: Rect, radius: f32, segs: usize) -> Vec<Vec2> {
    let rad = radius.min(r.w / 2.0).min(r.h / 2.0).max(0.0);
    let corners = [
        (vec2(r.right() - rad, r.y + rad), -90.0f32),
        (vec2(r.right() - rad, r.bottom() - rad), 0.0),
        (vec2(r.x + rad, r.bottom() - rad), 90.0),
        (vec2(r.x + rad, r.y + rad), 180.0),
    ];
    let mut pts = Vec::with_capacity(4 * (segs + 1));
    for (c, start) in corners {
        for s in 0..=segs {
            let a = (start + 90.0 * s as f32 / segs as f32).to_radians();
            pts.push(c + vec2(a.cos(), a.sin()) * rad);
        }
    }
    pts
}

/// A rounded rectangle filled with a vertical gradient (no overlaps, so translucent fills are even).
pub fn round_rect(r: Rect, radius: f32, top: Color, bottom: Color) {
    let pts = round_path(r, radius, corner_segments(radius));
    let shade = |y: f32| mix(top, bottom, ((y - r.y) / r.h.max(1.0)).clamp(0.0, 1.0));
    let c = r.center();
    let mut vertices = vec![Vertex::new(c.x, c.y, 0.0, 0.0, 0.0, shade(c.y))];
    vertices.extend(pts.iter().map(|p| Vertex::new(p.x, p.y, 0.0, 0.0, 0.0, shade(p.y))));
    let n = pts.len() as u16;
    let mut indices = Vec::with_capacity(pts.len() * 3);
    for i in 0..n {
        indices.extend_from_slice(&[0, 1 + i, 1 + (i + 1) % n]);
    }
    draw_mesh(&Mesh { vertices, indices, texture: None });
}

/// A rounded rectangle's outline, `width` thick, inside `r`.
pub fn round_outline(r: Rect, radius: f32, width: f32, color: Color) {
    // The same number of points on both edges, so they pair up into a strip.
    let segs = corner_segments(radius);
    let outer = round_path(r, radius, segs);
    let inner = round_path(
        Rect::new(r.x + width, r.y + width, (r.w - 2.0 * width).max(0.0), (r.h - 2.0 * width).max(0.0)),
        (radius - width).max(0.0),
        segs,
    );
    let mut vertices = Vec::with_capacity(outer.len() * 2);
    for (o, i) in outer.iter().zip(inner.iter()) {
        vertices.push(Vertex::new(o.x, o.y, 0.0, 0.0, 0.0, color));
        vertices.push(Vertex::new(i.x, i.y, 0.0, 0.0, 0.0, color));
    }
    let n = outer.len() as u16;
    let mut indices = Vec::with_capacity(outer.len() * 6);
    for k in 0..n {
        let (a, b) = (2 * k, 2 * ((k + 1) % n));
        indices.extend_from_slice(&[a, a + 1, b, b, a + 1, b + 1]);
    }
    draw_mesh(&Mesh { vertices, indices, texture: None });
}

/// A soft glow around `r`: outlines spreading outward, fading.
pub fn halo(r: Rect, radius: f32, spread: f32, color: Color, strength: f32) {
    const STEPS: usize = 6;
    let w = spread / STEPS as f32;
    for i in 0..STEPS {
        let d = w * (i + 1) as f32;
        let f = 1.0 - i as f32 / STEPS as f32;
        round_outline(Rect::new(r.x - d, r.y - d, r.w + 2.0 * d, r.h + 2.0 * d), radius + d, w, alpha(color, strength * f * f));
    }
}

/// A glass panel: deep translucent fill, an accent-coloured edge and a soft glow in that colour.
pub fn panel(r: Rect, accent: Color, u: f32) {
    let radius = 10.0 * u;
    halo(r, radius, 6.0 * u, accent, 0.10);
    round_rect(r, radius, Color::new(0.13, 0.10, 0.34, 0.78), Color::new(0.06, 0.04, 0.18, 0.84));
    // A sheen along the top edge.
    round_rect(
        Rect::new(r.x + 2.0 * u, r.y + 2.0 * u, r.w - 4.0 * u, (r.h * 0.35).min(28.0 * u)),
        radius - 2.0 * u,
        Color::new(1.0, 1.0, 1.0, 0.07),
        Color::new(1.0, 1.0, 1.0, 0.0),
    );
    round_outline(r, radius, (1.5 * u).max(1.0), alpha(accent, 0.75));
}

/// A glossy block, `size` pixels square, at (x, y).
pub fn block(x: f32, y: f32, size: f32, color: Color) {
    let gap = (size * 0.05).max(1.0).round();
    let r = Rect::new(x + gap, y + gap, size - 2.0 * gap, size - 2.0 * gap);
    let radius = size * 0.16;
    let a = color.a;
    let light = mix(color, WHITE, 0.30);
    let dark = mix(color, BLACK, 0.30);
    round_rect(r, radius, light.with_alpha(a), dark.with_alpha(a));
    // Shine across the top, a darker rim.
    let shine = Rect::new(r.x + r.w * 0.12, r.y + r.h * 0.10, r.w * 0.76, r.h * 0.26);
    round_rect(shine, radius * 0.6, Color::new(1.0, 1.0, 1.0, 0.45 * a), Color::new(1.0, 1.0, 1.0, 0.05 * a));
    round_outline(r, radius, (size * 0.05).max(1.0), mix(color, BLACK, 0.45).with_alpha(0.7 * a));
}

/// Where a piece will land: its outline in its colour over a faint fill.
pub fn ghost_block(x: f32, y: f32, size: f32, color: Color) {
    let gap = (size * 0.05).max(1.0).round();
    let r = Rect::new(x + gap, y + gap, size - 2.0 * gap, size - 2.0 * gap);
    let radius = size * 0.16;
    round_rect(r, radius, alpha(color, 0.22), alpha(color, 0.10));
    round_outline(r, radius, (size * 0.07).max(1.0), mix(color, WHITE, 0.35).with_alpha(0.75));
}

// The legacy look: green terminal text on black, to go with the legacy board.

/// The legacy board's green: borders, blocks and values.
pub const TERM_GREEN: Color = Color::new(0.0, 1.0, 0.0, 1.0);
/// Labels and quieter text in the legacy look.
pub const TERM_DIM: Color = Color::new(0.0, 0.62, 0.0, 1.0);

/// The legacy board's block glyph (drawn as an outlined box), `size` pixels square at (x, y),
/// placed in its square the way the board places it.
pub fn term_block(x: f32, y: f32, size: f32, color: Color) {
    crate::graphics::text::draw_text("█", x + size * 0.25, y + size * 0.7, size * 0.8, color);
}

/// A legacy-look box: black (`fill` opaque), with a thin edge (buttons, banners).
pub fn term_box(r: Rect, edge: Color, fill: f32, u: f32) {
    draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.0, 0.0, 0.0, fill));
    // Whole pixels (macroquad's draw_rectangle_lines draws half the thickness asked for, and a
    // sub-pixel edge can vanish).
    let t = (1.5 * u).round().max(1.0);
    let (x, y, w, h) = (r.x.round(), r.y.round(), r.w.round(), r.h.round());
    draw_rectangle(x, y, w, t, edge);
    draw_rectangle(x, y + h - t, w, t, edge);
    draw_rectangle(x, y + t, t, h - 2.0 * t, edge);
    draw_rectangle(x + w - t, y + t, t, h - 2.0 * t, edge);
}

/// A legacy-look frame in the board's style: `<== LABEL ====>` across the top of `r` and
/// `<=======>` across the bottom, in `size` text, over black. Returns the part of `r` between
/// them.
pub fn term_frame(r: Rect, label: &str, size: f32, rule: Color, label_color: Color) -> Rect {
    use crate::graphics::text::{draw_text, measure_text};
    draw_rectangle(r.x, r.y, r.w, r.h, Color::new(0.0, 0.0, 0.0, 0.85));
    // The font is monospaced: the rules are whole characters, centred.
    let char_w = measure_text("=", None, 1, size).width.max(1.0);
    let n = ((r.w / char_w).floor() as usize).max(4);
    let x = r.x + (r.w - n as f32 * char_w) / 2.0;
    let top = r.y + size * 0.8;
    if label.is_empty() || n < label.chars().count() + 7 {
        draw_text(&format!("<{}>", "=".repeat(n - 2)), x, top, size, rule);
    } else {
        // "<== " LABEL " ====>"
        let head = 4;
        let tail = n - head - label.chars().count();
        draw_text("<== ", x, top, size, rule);
        draw_text(label, x + head as f32 * char_w, top, size, label_color);
        let after = x + (n - tail) as f32 * char_w;
        draw_text(&format!(" {}>", "=".repeat(tail - 2)), after, top, size, rule);
    }
    draw_text(&format!("<{}>", "=".repeat(n - 2)), x, r.bottom() - size * 0.25, size, rule);
    Rect::new(r.x, r.y + size, r.w, (r.h - 2.0 * size).max(0.0))
}

/// `[#####.......]` across `w` pixels in `size` text, `done` (0..1) of it filled.
pub fn term_bar(x: f32, baseline: f32, w: f32, size: f32, done: f32, color: Color) {
    use crate::graphics::text::{draw_text, measure_text};
    let char_w = measure_text("#", None, 1, size).width.max(1.0);
    let n = ((w / char_w).floor() as usize).saturating_sub(2).max(1);
    let filled = ((done.clamp(0.0, 1.0) * n as f32).round() as usize).min(n);
    draw_text(&format!("[{}{}]", "#".repeat(filled), ".".repeat(n - filled)), x, baseline, size, color);
}
