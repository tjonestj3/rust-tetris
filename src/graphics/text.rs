//! Sharp text at any scale, from a glyph texture that's never deleted.
//!
//! The game draws text in canvas units through a camera that scales the canvas to the screen.
//! macroquad rasterizes glyphs at the font size it's given and then stretches the quads, so text
//! drawn at 28 units and shown 2.5x bigger comes out blurry. These stand-ins for macroquad's
//! `draw_text` / `measure_text` take the same arguments but rasterize at the size the text will
//! actually have on screen (`font_size x pixels per unit`) and scale back down by the same factor,
//! so layout is unchanged and the result is crisp.
//!
//! The glyphs are our own, not macroquad's: macroquad keeps them in a texture that, when full, it
//! deletes and replaces with one twice the size, in the middle of a frame. On the iPhone's GL
//! that left text sampling a dead texture (every glyph a black box, for good) as soon as a new
//! set of sizes appeared, e.g. on switching to the legacy look. Here the glyph texture is made
//! once, at a fixed size, and only ever written to; when it fills up it's emptied (not
//! replaced) at the start of the next frame ([`begin_frame`]), and a glyph that didn't fit
//! skips that one frame.
//!
//! Whoever sets a camera says how many screen pixels one unit is ([`set_pixels_per_unit`]);
//! the helpers in `graphics::view` do this. Import these instead of the prelude's:
//! `use crate::graphics::text::{draw_text, measure_text};` (an explicit import wins over a glob).

use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use macroquad::prelude::{
    draw_texture_ex, vec2, Color, DrawTextureParams, FilterMode, Font, Image, Rect, TextDimensions, Texture2D,
};

/// The font: macroquad's default, ProggyClean.
const FONT_BYTES: &[u8] = include_bytes!("../../assets/fonts/ProggyClean.ttf");

/// The glyph texture's side, in pixels: every iPhone GPU takes 4096.
const ATLAS_SIZE: u16 = 2048;

/// Space around each glyph in the texture, so smooth sampling doesn't pick up its neighbours.
const GAP: u16 = 2;

thread_local! {
    static PIXELS_PER_UNIT: Cell<f32> = const { Cell::new(1.0) };
    static FONT: fontdue::Font = fontdue::Font::from_bytes(FONT_BYTES, fontdue::FontSettings::default())
        .expect("the bundled font loads");
    /// Glyph sizes and placement, by (character, pixel size).
    static GLYPHS: RefCell<HashMap<(char, u16), Glyph>> = RefCell::new(HashMap::new());
    static ATLAS: RefCell<Option<Atlas>> = const { RefCell::new(None) };
}

/// One character at one pixel size.
#[derive(Debug, Clone, Copy)]
struct Glyph {
    advance: f32,
    offset_x: f32,
    offset_y: f32,
    width: u16,
    height: u16,
    /// Where it is in the texture (pixels), once drawn.
    place: Option<Rect>,
}

/// The glyph texture and where the next glyph goes.
struct Atlas {
    texture: Texture2D,
    packer: Packer,
}

/// Fills the texture in rows, left to right, top to bottom.
#[derive(Debug, Default)]
struct Packer {
    x: u16,
    y: u16,
    row_height: u16,
    /// Something didn't fit: empty the texture at the start of the next frame.
    full: bool,
}

impl Packer {
    /// Room for a `w` x `h` glyph: its top-left corner, or None (and `full`) if there's none.
    fn place(&mut self, w: u16, h: u16) -> Option<(u16, u16)> {
        let (w, h) = (w + GAP, h + GAP);
        if self.x + w > ATLAS_SIZE {
            // Next row.
            self.y += self.row_height;
            self.x = 0;
            self.row_height = 0;
        }
        if self.x + w > ATLAS_SIZE || self.y + h > ATLAS_SIZE {
            self.full = true;
            return None;
        }
        let at = (self.x, self.y);
        self.x += w;
        self.row_height = self.row_height.max(h);
        Some(at)
    }
}

/// Start of a frame, before anything draws: if the glyph texture filled up last frame, empty
/// it. (Not mid-frame: the frame's earlier text still reads from it until it's drawn.)
pub fn begin_frame() {
    ATLAS.with(|a| {
        if let Some(atlas) = a.borrow_mut().as_mut() {
            if atlas.packer.full {
                log::info!("Glyph texture full; starting it over");
                atlas.packer = Packer::default();
                GLYPHS.with(|g| g.borrow_mut().values_mut().for_each(|glyph| glyph.place = None));
            }
        }
    });
}

/// Screen pixels per drawing unit under the current camera (1 for the default camera).
pub fn set_pixels_per_unit(scale: f32) {
    PIXELS_PER_UNIT.with(|s| s.set(if scale.is_finite() && scale > 0.0 { scale } else { 1.0 }));
}

pub fn pixels_per_unit() -> f32 {
    PIXELS_PER_UNIT.with(|s| s.get())
}

/// The font size to rasterize at and the scale that brings it back to `size` units. Sizes are
/// rounded to steps (finer when small) so animated text doesn't fill the glyph texture with
/// every size it passes through; the scale makes up the difference exactly.
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

/// The glyph for `ch` at `px` (its measurements; not necessarily in the texture yet).
fn glyph(ch: char, px: u16) -> Glyph {
    GLYPHS.with(|g| {
        *g.borrow_mut().entry((ch, px)).or_insert_with(|| {
            let m = FONT.with(|f| f.metrics(ch, px as f32));
            Glyph {
                advance: m.advance_width,
                offset_x: m.xmin as f32,
                offset_y: m.ymin as f32,
                width: m.width as u16,
                height: m.height as u16,
                place: None,
            }
        })
    })
}

/// Where `ch` at `px` is in the glyph texture, drawing it there first if it isn't yet. None if
/// it has no pixels (a space) or there's no room this frame.
fn placed(ch: char, px: u16, g: &Glyph) -> Option<(Texture2D, Rect)> {
    if g.width == 0 || g.height == 0 {
        return None;
    }
    ATLAS.with(|a| {
        let mut a = a.borrow_mut();
        let atlas = a.get_or_insert_with(|| {
            let bytes = vec![0u8; ATLAS_SIZE as usize * ATLAS_SIZE as usize * 4];
            let texture = Texture2D::from_rgba8(ATLAS_SIZE, ATLAS_SIZE, &bytes);
            texture.set_filter(FilterMode::Linear);
            Atlas { texture, packer: Packer::default() }
        });
        if let Some(rect) = g.place {
            return Some((atlas.texture.clone(), rect));
        }
        let (x, y) = atlas.packer.place(g.width, g.height)?;
        let (m, coverage) = FONT.with(|f| f.rasterize(ch, px as f32));
        let (w, h) = (m.width as u16, m.height as u16);
        let image = Image {
            bytes: coverage.iter().flat_map(|&c| [255, 255, 255, c]).collect(),
            width: w,
            height: h,
        };
        atlas.texture.update_part(&image, x as i32, y as i32, w as i32, h as i32);
        let rect = Rect::new(x as f32, y as f32, w as f32, h as f32);
        GLYPHS.with(|gs| {
            if let Some(stored) = gs.borrow_mut().get_mut(&(ch, px)) {
                stored.place = Some(rect);
            }
        });
        Some((atlas.texture.clone(), rect))
    })
}

/// Lay out `text` at `px` scaled by `scale`: its dimensions, calling `each` with every glyph
/// and its top-left corner relative to the baseline start.
fn layout(text: &str, px: u16, scale: f32, mut each: impl FnMut(char, &Glyph, f32, f32)) -> TextDimensions {
    let mut width = 0.0;
    let (mut min_y, mut max_y) = (f32::MAX, f32::MIN);
    for ch in text.chars() {
        let g = glyph(ch, px);
        let offset_y = g.offset_y * scale;
        let h = g.height as f32 * scale;
        min_y = min_y.min(offset_y);
        max_y = max_y.max(h + offset_y);
        each(ch, &g, width + g.offset_x * scale, -h - offset_y);
        width += g.advance * scale;
    }
    if text.is_empty() {
        return TextDimensions::default();
    }
    TextDimensions { width, height: max_y - min_y, offset_y: max_y }
}

pub fn draw_text(text: &str, x: f32, y: f32, font_size: f32, color: Color) -> TextDimensions {
    let (px, scale) = raster(font_size);
    layout(text, px, scale, |ch, g, dx, dy| {
        if let Some((texture, source)) = placed(ch, px, g) {
            draw_texture_ex(
                &texture,
                x + dx,
                y + dy,
                color,
                DrawTextureParams {
                    dest_size: Some(vec2(source.w * scale, source.h * scale)),
                    source: Some(source),
                    ..Default::default()
                },
            );
        }
    })
}

/// `font` is ignored (it's there to match macroquad's signature): text is measured in the font
/// it's drawn in.
pub fn measure_text(text: &str, _font: Option<&Font>, font_size: u16, font_scale: f32) -> TextDimensions {
    let (px, scale) = raster(font_size as f32 * font_scale);
    layout(text, px, scale, |_, _, _, _| {})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyphs_fill_rows_then_run_out() {
        let mut p = Packer::default();
        assert_eq!(p.place(100, 50), Some((0, 0)));
        assert_eq!(p.place(100, 30), Some((100 + GAP, 0)));
        // Too wide for what's left of the row: the next row, under the tallest so far.
        assert_eq!(p.place(ATLAS_SIZE - 10, 20), Some((0, 50 + GAP)));
        assert!(!p.full);
        // Too tall for what's left.
        assert_eq!(p.place(10, ATLAS_SIZE), None);
        assert!(p.full);
    }

    #[test]
    fn measuring_needs_no_texture_and_scales() {
        set_pixels_per_unit(1.0);
        let small = measure_text("HELLO", None, 20, 1.0);
        let big = measure_text("HELLO", None, 40, 1.0);
        assert!(small.width > 0.0 && small.height > 0.0);
        assert!((big.width / small.width - 2.0).abs() < 0.15);
        // Sharp on a 3x screen, same size in units.
        set_pixels_per_unit(3.0);
        let sharp = measure_text("HELLO", None, 20, 1.0);
        assert!((sharp.width / small.width - 1.0).abs() < 0.1);
        set_pixels_per_unit(1.0);
        assert_eq!(measure_text("", None, 20, 1.0).width, 0.0);
    }
}
