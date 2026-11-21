//! One file per theme (see super::theme).

pub mod arcade;
pub mod chess;
pub mod desktop98;
pub mod glass;
pub mod linux;
pub mod macosx;
pub mod neon;
pub mod terminal;
pub mod warm_paper;

use macroquad::prelude::*;

use super::style;

/// A raised (or, `raised` false, sunken) bevel around `r`: light along the top and left, dark
/// along the bottom and right, `t` pixels each.
pub(super) fn bevel(r: Rect, t: f32, light: Color, dark: Color, raised: bool) {
    let (tl, br) = if raised { (light, dark) } else { (dark, light) };
    draw_rectangle(r.x, r.y, r.w, t, tl);
    draw_rectangle(r.x, r.y, t, r.h, tl);
    draw_rectangle(r.x, r.bottom() - t, r.w, t, br);
    draw_rectangle(r.right() - t, r.y, t, r.h, br);
}

/// A number in 0..1, the same each time for the same `i` and `k` (grain, specks).
pub(super) fn hash01(i: u32, k: u32) -> f32 {
    let mut h = i.wrapping_mul(0x9E37_79B1) ^ k.wrapping_mul(0x85EB_CA77);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0xFFFF) as f32 / 65536.0
}

/// Soft edge: a filled circle drawn as rings fading out, for blurred blobs of light.
pub(super) fn soft_circle(x: f32, y: f32, r: f32, color: Color) {
    const RINGS: usize = 40;
    for i in 0..RINGS {
        let k = 1.0 - i as f32 / RINGS as f32;
        draw_circle(x, y, r * k, style::alpha(color, color.a * 0.03));
    }
}
