//! Sharp text at any scale.
//!
//! The game draws text in canvas units through a camera that scales the canvas to the screen.
//! macroquad rasterizes glyphs at the font size it's given and then stretches the quads, so text
//! drawn at 28 units and shown 2.5x bigger comes out blurry. These stand-ins for macroquad's
//! `draw_text` / `measure_text` take the same arguments but rasterize at the size the text will
//! actually have on screen (`font_size x pixels per unit`) and scale back down by the same factor,
//! so layout is unchanged and the result is crisp.
//!
//! Whoever sets a camera says how many screen pixels one unit is ([`set_pixels_per_unit`]);
//! the helpers in `graphics::view` do this. Import these instead of the prelude's:
//! `use crate::graphics::text::{draw_text, measure_text};` (an explicit import wins over a glob).

use std::cell::Cell;

use macroquad::prelude::{Color, Font, TextDimensions, TextParams};

thread_local! {
    static PIXELS_PER_UNIT: Cell<f32> = const { Cell::new(1.0) };
}

/// Screen pixels per drawing unit under the current camera (1 for the default camera).
pub fn set_pixels_per_unit(scale: f32) {
    PIXELS_PER_UNIT.with(|s| s.set(if scale.is_finite() && scale > 0.0 { scale } else { 1.0 }));
}

pub fn pixels_per_unit() -> f32 {
    PIXELS_PER_UNIT.with(|s| s.get())
}

/// The font size to rasterize at and the scale that brings it back to `size` units. Sizes are
/// rounded to steps (finer when small) so animated text doesn't fill the glyph cache with every
/// size it passes through; the scale makes up the difference exactly.
fn raster(size: f32) -> (u16, f32) {
    let want = (size * pixels_per_unit()).max(1.0);
    let step = if want < 24.0 { 1.0 } else if want < 64.0 { 2.0 } else { 4.0 };
    let px = ((want / step).round() * step).clamp(1.0, 512.0);
    (px as u16, size / px)
}

pub fn draw_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) -> TextDimensions {
    let (px, scale) = raster(font_size);
    macroquad::text::draw_text_ex(
        text,
        x,
        y,
        TextParams { font_size: px, font_scale: scale, color, ..Default::default() },
    )
}

pub fn measure_text(text: &str, font: Option<&Font>, font_size: u16, font_scale: f32) -> TextDimensions {
    let (px, scale) = raster(font_size as f32 * font_scale);
    macroquad::text::measure_text(text, font, px, scale)
}
