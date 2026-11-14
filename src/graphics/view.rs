//! Screen layout: where the fixed 900x750 game canvas goes on whatever screen we have.
//!
//! All the game and menu drawing is written for a 900x750 canvas (`WINDOW_WIDTH` x `WINDOW_HEIGHT`).
//! Instead of rewriting it, the canvas is drawn through a camera that scales it to fit the screen
//! (letterboxed, never stretched), inside the safe area on iPhone (clear of the Dynamic Island and
//! home bar). It's recomputed every frame, so rotating the phone or unfolding an iPhone Duo just
//! re-lays it out. Touch play is all gestures, so the whole screen is free for the canvas.

use macroquad::prelude::*;
use crate::game::config::{WINDOW_WIDTH, WINDOW_HEIGHT};

/// Canvas size in canvas units.
pub const CANVAS_W: f32 = WINDOW_WIDTH as f32;
pub const CANVAS_H: f32 = WINDOW_HEIGHT as f32;

/// One frame's layout, in screen pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    /// Where the canvas is drawn.
    pub game: Rect,
    /// Screen pixels per canvas unit.
    pub scale: f32,
    /// Screen pixels per point (UIKit point; 1 on desktop), for sizing touch targets and gestures.
    pub unit: f32,
    /// The screen minus the notch / island / home bar margins.
    pub safe: Rect,
}

impl Layout {
    /// Lay out the current screen.
    pub fn compute() -> Self {
        let (w, h) = (screen_width(), screen_height());
        let unit = points_scale();
        let (top, left, bottom, right) = safe_insets();
        let safe = Rect::new(left * unit, top * unit, w - (left + right) * unit, h - (top + bottom) * unit);
        let game = fit(safe);
        Self { game, scale: game.w / CANVAS_W, unit, safe }
    }

    /// Draw in canvas units from here on (the game and menus).
    pub fn set_canvas_camera(&self) {
        let g = self.game;
        set_camera(&Camera2D {
            target: vec2(CANVAS_W / 2.0, CANVAS_H / 2.0),
            zoom: vec2(2.0 / CANVAS_W, 2.0 / CANVAS_H),
            // GL viewport: origin at the bottom left. It also clips drawing to the canvas.
            viewport: Some((
                g.x.round() as i32,
                (screen_height() - g.y - g.h).round() as i32,
                g.w.round() as i32,
                g.h.round() as i32,
            )),
            ..Default::default()
        });
    }

    /// Screen pixels to canvas units, or None when outside the canvas.
    pub fn screen_to_canvas(&self, p: Vec2) -> Option<Vec2> {
        if !self.game.contains(p) {
            return None;
        }
        Some(vec2((p.x - self.game.x) / self.scale, (p.y - self.game.y) / self.scale))
    }
}

/// The largest canvas-shaped rect centred in `area`.
fn fit(area: Rect) -> Rect {
    let scale = (area.w / CANVAS_W).min(area.h / CANVAS_H).max(0.01);
    let (w, h) = (CANVAS_W * scale, CANVAS_H * scale);
    Rect::new(area.x + (area.w - w) / 2.0, area.y + (area.h - h) / 2.0, w, h)
}

/// Screen pixels per point. miniquad renders iOS at a fixed 2x when high_dpi is on.
fn points_scale() -> f32 {
    if cfg!(target_os = "ios") { 2.0 } else { 1.0 }
}

/// The notch / Dynamic Island / home indicator margins in points: (top, left, bottom, right).
#[cfg(target_os = "ios")]
fn safe_insets() -> (f32, f32, f32, f32) {
    crate::platform::ios_safe_area_insets().unwrap_or_else(|| {
        // Best guess when UIKit won't say: keep clear of the island and the home bar.
        if screen_width() > screen_height() { (0.0, 50.0, 21.0, 50.0) } else { (59.0, 0.0, 34.0, 0.0) }
    })
}

#[cfg(not(target_os = "ios"))]
fn safe_insets() -> (f32, f32, f32, f32) {
    (0.0, 0.0, 0.0, 0.0)
}
