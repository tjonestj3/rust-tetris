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

//! The glyphs live in one texture per font (macroquad's atlas), which doubles in size whenever
//! it fills, without limit. Every size text is drawn at caches its own copy of each glyph, so
//! a long session (the legacy board is all text) could grow it past what the GPU allows, and
//! then every glyph draws as a black box. So the font here is our own copy of macroquad's, and
//! once its glyphs would fill a texture of [`ATLAS_BUDGET`] pixels it's swapped for a fresh one
//! at the start of the next frame ([`begin_frame`]); the old texture goes with the old font.

use std::cell::{Cell, RefCell};
use std::collections::HashSet;

use macroquad::prelude::{Color, Font, TextDimensions, TextParams};

/// The font (macroquad's default, ProggyClean).
const FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/ProggyClean.ttf");

/// Glyph pixels (with padding) a font may cache before it's replaced: about a 2048 x 2048
/// texture's worth, so the atlas stays well under the 4096 every iPhone GPU handles even
/// with the atlas's loose packing.
const ATLAS_BUDGET: f32 = 2048.0 * 2048.0 * 0.6;

/// The size macroquad pre-caches ASCII at when it loads a font.
const PRELOAD_SIZE: u16 = 15;

thread_local! {
    static PIXELS_PER_UNIT: Cell<f32> = const { Cell::new(1.0) };
    static FONT: RefCell<FontCache> = RefCell::new(FontCache::default());
}

/// Our font and a running tally of the glyphs it has cached.
#[derive(Default)]
struct FontCache {
    font: Option<Font>,
    usage: GlyphUsage,
}

/// Which (character, size) glyphs a font has cached, and roughly how much atlas they fill.
#[derive(Default)]
struct GlyphUsage {
    glyphs: HashSet<(char, u16)>,
    area: f32,
}

impl GlyphUsage {
    /// Note the glyphs `text` at `px` will cache.
    fn add(&mut self, text: &str, px: u16) {
        for ch in text.chars() {
            if self.glyphs.insert((ch, px)) {
                // A glyph's box (ProggyClean is narrow) plus the atlas's gaps.
                let size = px as f32;
                self.area += (size * 0.7 + 4.0) * (size + 4.0);
            }
        }
    }

    fn over_budget(&self) -> bool {
        self.area > ATLAS_BUDGET
    }

    /// A fresh font's: macroquad caches ASCII at 15 px when it loads one.
    fn fresh() -> Self {
        let mut usage = Self::default();
        let ascii: String = (0x20u8..0x7f).map(|b| b as char).collect();
        usage.add(&ascii, PRELOAD_SIZE);
        usage
    }
}

/// Start of a frame (before anything draws): if the font's glyph texture has grown as big as
/// it should, start over with a fresh font. Never mid-frame: this frame's draws still use the
/// old texture until they're flushed.
pub fn begin_frame() {
    FONT.with(|f| {
        let mut f = f.borrow_mut();
        if f.usage.over_budget() {
            log::info!("Text glyph cache full ({} glyphs); starting a fresh one", f.usage.glyphs.len());
            f.font = None;
            f.usage = GlyphUsage::default();
        }
    });
}

/// Run `draw` with our font, after noting the glyphs `text` at `px` will cache.
fn with_font<R>(text: &str, px: u16, draw: impl FnOnce(Option<&Font>) -> R) -> R {
    FONT.with(|f| {
        let mut f = f.borrow_mut();
        if f.font.is_none() {
            match macroquad::text::load_ttf_font_from_bytes(FONT_BYTES) {
                Ok(font) => {
                    f.font = Some(font);
                    f.usage = GlyphUsage::fresh();
                }
                Err(e) => log::warn!("Couldn't load the font, using macroquad's: {e:?}"),
            }
        }
        f.usage.add(text, px);
        draw(f.font.as_ref())
    })
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
    let step = if want < 24.0 {
        1.0
    } else if want < 64.0 {
        2.0
    } else if want < 128.0 {
        4.0
    } else {
        8.0
    };
    let px = ((want / step).round() * step).clamp(1.0, 512.0);
    (px as u16, size / px)
}

pub fn draw_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) -> TextDimensions {
    let (px, scale) = raster(font_size);
    with_font(text, px, |font| {
        macroquad::text::draw_text_ex(
            text,
            x,
            y,
            TextParams { font, font_size: px, font_scale: scale, color, ..Default::default() },
        )
    })
}

/// `font` is ignored (it's there to match macroquad's signature): text is measured in the font
/// it's drawn in.
pub fn measure_text(text: &str, _font: Option<&Font>, font_size: u16, font_scale: f32) -> TextDimensions {
    let (px, scale) = raster(font_size as f32 * font_scale);
    with_font(text, px, |font| macroquad::text::measure_text(text, font, px, scale))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_glyph_counts_once_per_size() {
        let mut u = GlyphUsage::default();
        u.add("AAB", 20);
        let once = u.area;
        u.add("BA", 20);
        assert_eq!(u.area, once);
        u.add("A", 21);
        assert!(u.area > once);
        assert_eq!(u.glyphs.len(), 3);
    }

    #[test]
    fn many_sizes_fill_the_budget_but_one_screen_does_not() {
        let ascii: String = (0x20u8..0x7f).map(|b| b as char).collect();
        // A screenful: the alphabet at a dozen sizes.
        let mut u = GlyphUsage::fresh();
        for px in [12, 14, 16, 18, 20, 24, 28, 32, 40, 48, 64, 96] {
            u.add(&ascii, px);
        }
        assert!(!u.over_budget());
        // Every size up to 200 px: more than an atlas should hold.
        for px in 1..=200 {
            u.add(&ascii, px);
        }
        assert!(u.over_budget());
    }
}
