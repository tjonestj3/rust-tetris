//! Screen layout, recomputed every frame (the phone may rotate, the Duo may fold or unfold).
//!
//! The game and menus are drawn in units of a fixed 900x750 canvas (`WINDOW_WIDTH` x
//! `WINDOW_HEIGHT`). On desktop the whole canvas is scaled to fit the window, as it always was.
//!
//! On touch screens nothing is letterboxed:
//! - **Playing**: [`PlayLayout`] gives the board the full height of the safe area (or the full
//!   width on tall phones) and puts Next / Hold / the score in a side column, two side columns, or
//!   a strip along the top, whichever gives the bigger board. The existing board drawing is reused
//!   through a camera that maps the canvas's board onto the new board rect ([`map_camera`]).
//! - **Menus**: only the part of the canvas a menu actually uses (`canvas_src`) is fitted to the
//!   safe area, so it comes out bigger, and the background covers the whole screen.
//!
//! On iPhone everything stays inside the safe area (clear of the Dynamic Island and home bar).

use macroquad::prelude::*;
use crate::game::config::{
    BOARD_HEIGHT_PX, BOARD_OFFSET_X, BOARD_OFFSET_Y, BOARD_WIDTH, BOARD_WIDTH_PX, VISIBLE_HEIGHT,
    WINDOW_HEIGHT, WINDOW_WIDTH,
};
use crate::graphics::text::set_pixels_per_unit;

/// Canvas size in canvas units.
pub const CANVAS_W: f32 = WINDOW_WIDTH as f32;
pub const CANVAS_H: f32 = WINDOW_HEIGHT as f32;

/// The whole canvas.
pub const FULL_CANVAS: Rect = Rect { x: 0.0, y: 0.0, w: CANVAS_W, h: CANVAS_H };

/// The board's rect in canvas units (where the existing drawing code puts it).
pub const CANVAS_BOARD: Rect =
    Rect { x: BOARD_OFFSET_X, y: BOARD_OFFSET_Y, w: BOARD_WIDTH_PX, h: BOARD_HEIGHT_PX };

/// One frame's layout, in screen pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Layout {
    /// The whole screen.
    pub screen: Rect,
    /// The screen minus the notch / island / home bar margins.
    pub safe: Rect,
    /// Screen pixels per point (UIKit point; 1 on desktop), for touch targets and gestures.
    pub unit: f32,
    /// The part of the canvas shown (menus and overlays), and where it's drawn.
    pub canvas_src: Rect,
    pub game: Rect,
    /// Screen pixels per canvas unit for `canvas_src` -> `game`.
    pub scale: f32,
    /// Where things go while playing by touch.
    pub play: PlayLayout,
}

impl Layout {
    /// Lay out the current screen, fitting `canvas_src` (a part of the canvas) to the safe area.
    pub fn compute(canvas_src: Rect) -> Self {
        let (w, h) = (screen_width(), screen_height());
        let unit = points_scale();
        let (top, left, bottom, right) = safe_insets();
        let screen = Rect::new(0.0, 0.0, w, h);
        let safe = Rect::new(left * unit, top * unit, w - (left + right) * unit, h - (top + bottom) * unit);
        Self::for_screen(screen, safe, unit, canvas_src)
    }

    /// The layout for a given screen (separate from [`Layout::compute`] for tests).
    pub fn for_screen(screen: Rect, safe: Rect, unit: f32, canvas_src: Rect) -> Self {
        let game = fit(canvas_src, safe);
        Self {
            screen,
            safe,
            unit,
            canvas_src,
            game,
            scale: game.w / canvas_src.w,
            play: PlayLayout::compute(safe, unit),
        }
    }

    /// Draw in canvas units from here on, `canvas_src` shown in `game` (clipped to it).
    pub fn set_canvas_camera(&self) {
        map_camera_clipped(self.canvas_src, self.game);
    }

    /// Screen pixels to canvas units. Anywhere on the screen maps somewhere (outside the shown
    /// part too), so "tap anywhere else to go back" works on the margins.
    pub fn screen_to_canvas(&self, p: Vec2) -> Option<Vec2> {
        Some(vec2(
            self.canvas_src.x + (p.x - self.game.x) / self.scale,
            self.canvas_src.y + (p.y - self.game.y) / self.scale,
        ))
    }
}

/// How the touch HUD is arranged around the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HudStyle {
    /// Board on the left, one column on the right: Next, Hold, stats, pause at the bottom.
    OneColumn,
    /// Board in the middle: Hold and stats on the left, Next and pause on the right.
    TwoColumns,
    /// Board filling the width, a strip above it: Hold, stats, Next, pause.
    TopStrip,
}

/// Where everything goes while playing by touch, in screen pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlayLayout {
    pub style: HudStyle,
    /// The board (10 x 20 cells, whole pixels per cell).
    pub board: Rect,
    /// One cell, in pixels.
    pub cell: f32,
    pub next: Rect,
    pub hold: Rect,
    pub stats: Rect,
    pub pause: Rect,
    /// The gap between panels.
    pub gap: f32,
}

/// Side column width, in cells (fits a 4-block piece at 0.8 of a cell plus padding).
const COLUMN_CELLS: f32 = 5.2;
/// Panel heights, in cells.
const PIECE_PANEL_CELLS: f32 = 3.6;
const STRIP_CELLS: f32 = 3.4;

impl PlayLayout {
    pub fn compute(safe: Rect, unit: f32) -> Self {
        let m = 10.0 * unit; // margins and gaps
        let min_col = 96.0 * unit; // narrowest side column that still reads well
        let (cols, rows) = (BOARD_WIDTH as f32, VISIBLE_HEIGHT as f32);
        let by_height = (safe.h - 2.0 * m) / rows;

        // The biggest cell each arrangement allows.
        let one = by_height.min((safe.w - 3.0 * m) / (cols + COLUMN_CELLS));
        let one = if one * COLUMN_CELLS < min_col {
            by_height.min((safe.w - 3.0 * m - min_col) / cols)
        } else {
            one
        };
        let two = by_height.min((safe.w - 4.0 * m) / (cols + 2.0 * COLUMN_CELLS));
        let top = ((safe.h - 3.0 * m) / (rows + STRIP_CELLS)).min((safe.w - 2.0 * m) / cols);

        // Two columns look balanced, so take them when they cost (almost) nothing.
        let (style, cell) = if two >= one * 0.98 && two >= top {
            (HudStyle::TwoColumns, two)
        } else if one >= top {
            (HudStyle::OneColumn, one)
        } else {
            (HudStyle::TopStrip, top)
        };
        let cell = cell.floor().max(4.0);
        let (bw, bh) = (cell * cols, cell * rows);
        let pause_size = (44.0 * unit).min(cell * 2.2).max(28.0 * unit);

        match style {
            HudStyle::OneColumn => {
                let col_w = (safe.w - bw - 3.0 * m).min(cell * 7.0);
                let group_w = bw + m + col_w;
                let x = (safe.x + (safe.w - group_w) / 2.0).round();
                let y = (safe.y + (safe.h - bh) / 2.0).round();
                let board = Rect::new(x, y, bw, bh);
                let cx = board.right() + m;
                let ph = cell * PIECE_PANEL_CELLS;
                let next = Rect::new(cx, y, col_w, ph);
                let hold = Rect::new(cx, next.bottom() + m, col_w, ph);
                let pause = Rect::new(cx + col_w - pause_size, board.bottom() - pause_size, pause_size, pause_size);
                // Down to the pause button: score, level / lines, and time at the bottom.
                let stats = Rect::new(cx, hold.bottom() + m, col_w, pause.y - m - (hold.bottom() + m));
                Self { style, board, cell, next, hold, stats, pause, gap: m }
            }
            HudStyle::TwoColumns => {
                let col_w = ((safe.w - bw - 4.0 * m) / 2.0).min(cell * 8.0);
                let x = (safe.x + (safe.w - bw) / 2.0).round();
                let y = (safe.y + (safe.h - bh) / 2.0).round();
                let board = Rect::new(x, y, bw, bh);
                let ph = cell * PIECE_PANEL_CELLS;
                let lx = board.x - m - col_w;
                let rx = board.right() + m;
                let hold = Rect::new(lx, y, col_w, ph);
                // As tall as its content (score, level / lines, time).
                let stats = Rect::new(lx, hold.bottom() + m, col_w, (cell * 6.8).min(board.bottom() - (hold.bottom() + m)));
                let next = Rect::new(rx, y, col_w, ph);
                let pause = Rect::new(rx + col_w - pause_size, board.bottom() - pause_size, pause_size, pause_size);
                Self { style, board, cell, next, hold, stats, pause, gap: m }
            }
            HudStyle::TopStrip => {
                let sh = (cell * STRIP_CELLS).round();
                let total_h = sh + m + bh;
                let x = (safe.x + (safe.w - bw) / 2.0).round();
                let y0 = (safe.y + (safe.h - total_h) / 2.0).round();
                let board = Rect::new(x, y0 + sh + m, bw, bh);
                // Hold | score | Next | pause, the score getting what's left (at least ~40%).
                let g = m * 0.6;
                let pause_w = pause_size.min(sh);
                let avail = bw - pause_w - 3.0 * g;
                let piece_w = (avail * 0.3).min(sh * 1.15).floor();
                let hold = Rect::new(x, y0, piece_w, sh);
                let stats = Rect::new(hold.right() + g, y0, avail - 2.0 * piece_w, sh);
                let next = Rect::new(stats.right() + g, y0, piece_w, sh);
                let pause = Rect::new(board.right() - pause_w, y0 + (sh - pause_w) / 2.0, pause_w, pause_w);
                Self { style, board, cell, next, hold, stats, pause, gap: m }
            }
        }
    }
}

/// Draw `src` (canvas units) into `dst` (screen pixels), keeping proportions (`dst` should have
/// `src`'s aspect ratio), without clipping: things drawn outside `src` show up outside `dst`.
pub fn map_camera(src: Rect, dst: Rect) {
    let (w, h) = (screen_width(), screen_height());
    let k = dst.w / src.w;
    // With the full-screen viewport the screen centre shows `target`; shift it so `src`'s centre
    // lands on `dst`'s.
    let screen_c = vec2(w / 2.0, h / 2.0);
    let target = src.center() - (dst.center() - screen_c) / k;
    set_camera(&Camera2D {
        target,
        zoom: vec2(2.0 * k / w, 2.0 * k / h),
        viewport: Some((0, 0, w as i32, h as i32)),
        ..Default::default()
    });
    set_pixels_per_unit(k);
}

/// Like [`map_camera`], but drawing is clipped to `dst`.
pub fn map_camera_clipped(src: Rect, dst: Rect) {
    set_camera(&Camera2D {
        target: src.center(),
        zoom: vec2(2.0 / src.w, 2.0 / src.h),
        // GL viewport: origin at the bottom left.
        viewport: Some((
            dst.x.round() as i32,
            (screen_height() - dst.y - dst.h).round() as i32,
            dst.w.round() as i32,
            dst.h.round() as i32,
        )),
        ..Default::default()
    });
    set_pixels_per_unit(dst.w / src.w);
}

/// Draw in screen pixels from here on.
pub fn screen_camera() {
    set_default_camera();
    set_pixels_per_unit(1.0);
}

/// The backdrop over the whole screen (scaled to cover it, centred), dimmed by `dim`. Draws in
/// screen pixels and puts the camera back the way it was.
pub fn draw_cover_background(texture: &Texture2D, dim: f32) {
    let ppu = crate::graphics::text::pixels_per_unit();
    push_camera_state();
    set_default_camera();
    let (w, h) = (screen_width(), screen_height());
    clear_background(Color::new(0.02, 0.02, 0.08, 1.0));
    let (tw, th) = (texture.width(), texture.height());
    let s = (w / tw).max(h / th);
    draw_texture_ex(
        texture,
        (w - tw * s) / 2.0,
        (h - th * s) / 2.0,
        WHITE,
        DrawTextureParams { dest_size: Some(vec2(tw * s, th * s)), ..Default::default() },
    );
    draw_rectangle(0.0, 0.0, w, h, Color::new(0.0, 0.0, 0.0, dim));
    pop_camera_state();
    set_pixels_per_unit(ppu);
}

/// The largest rect with `src`'s proportions, centred in `area`.
pub fn fit(src: Rect, area: Rect) -> Rect {
    let scale = (area.w / src.w).min(area.h / src.h).max(0.01);
    let (w, h) = (src.w * scale, src.h * scale);
    Rect::new(area.x + (area.w - w) / 2.0, area.y + (area.h - h) / 2.0, w, h)
}

/// Screen pixels per point (UIKit's on iPhone; 1 on desktop).
fn points_scale() -> f32 {
    #[cfg(target_os = "ios")]
    return crate::platform::ios_content_scale();
    #[cfg(not(target_os = "ios"))]
    return 1.0;
}

/// The notch / Dynamic Island / home indicator margins in points: (top, left, bottom, right).
#[cfg(target_os = "ios")]
fn safe_insets() -> (f32, f32, f32, f32) {
    crate::platform::ios_safe_area_insets().unwrap_or_else(|| {
        // Best guess when UIKit won't say: keep clear of the island and the home bar.
        if screen_width() > screen_height() { (0.0, 50.0, 21.0, 50.0) } else { (59.0, 0.0, 34.0, 0.0) }
    })
}

/// Desktop: none, unless `TETRIS_INSETS=top,left,bottom,right` fakes a phone's (for testing).
#[cfg(not(target_os = "ios"))]
fn safe_insets() -> (f32, f32, f32, f32) {
    std::env::var("TETRIS_INSETS")
        .ok()
        .and_then(|v| {
            let n: Vec<f32> = v.split(',').filter_map(|p| p.trim().parse().ok()).collect();
            (n.len() == 4).then(|| (n[0], n[1], n[2], n[3]))
        })
        .unwrap_or((0.0, 0.0, 0.0, 0.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn play(w: f32, h: f32) -> PlayLayout {
        PlayLayout::compute(Rect::new(0.0, 0.0, w, h), 1.0)
    }

    fn inside(r: Rect, area: Rect) -> bool {
        r.x >= area.x - 0.5 && r.y >= area.y - 0.5 && r.right() <= area.right() + 0.5 && r.bottom() <= area.bottom() + 0.5
    }

    fn assert_sane(p: &PlayLayout, w: f32, h: f32) {
        let area = Rect::new(0.0, 0.0, w, h);
        for (name, r) in [("board", p.board), ("next", p.next), ("hold", p.hold), ("stats", p.stats), ("pause", p.pause)] {
            assert!(inside(r, area), "{name} {r:?} outside {w}x{h} ({:?})", p.style);
            assert!(r.w > 0.0 && r.h > 0.0, "{name} empty: {r:?}");
        }
        for (name, r) in [("next", p.next), ("hold", p.hold), ("stats", p.stats), ("pause", p.pause)] {
            assert!(!r.overlaps(&p.board), "{name} {r:?} overlaps the board {:?}", p.board);
        }
        assert_eq!(p.board.w, p.cell * 10.0);
        assert_eq!(p.board.h, p.cell * 20.0);
    }

    #[test]
    fn folded_duo_gets_a_full_height_board_and_one_column() {
        let p = play(458.0, 574.0);
        assert_eq!(p.style, HudStyle::OneColumn);
        assert!(p.board.h >= 574.0 - 2.0 * 10.0 - 20.0, "board only {}", p.board.h);
        assert_sane(&p, 458.0, 574.0);
    }

    #[test]
    fn open_duo_gets_two_columns() {
        let p = play(568.0, 449.0);
        assert_eq!(p.style, HudStyle::TwoColumns);
        assert!(p.board.h >= 449.0 - 2.0 * 10.0 - 20.0);
        assert_sane(&p, 568.0, 449.0);
    }

    #[test]
    fn tall_phone_gets_a_top_strip() {
        let p = play(390.0, 751.0);
        assert_eq!(p.style, HudStyle::TopStrip);
        // Height-limited: the strip and the board use the full height.
        assert!(p.board.bottom() - p.hold.y >= 751.0 - 3.0 * 10.0 - p.cell, "{p:?}");
        assert!(p.stats.w >= p.cell * 3.0, "score panel too narrow: {:?}", p.stats);
        assert_sane(&p, 390.0, 751.0);
    }

    #[test]
    fn landscape_phone_and_ipad_are_sane() {
        for (w, h) in [(744.0, 369.0), (1024.0, 1300.0), (1366.0, 1000.0), (320.0, 480.0)] {
            assert_sane(&play(w, h), w, h);
        }
    }

    #[test]
    fn menu_taps_map_back_to_the_canvas() {
        let src = Rect::new(150.0, 60.0, 600.0, 600.0);
        let screen = Rect::new(0.0, 0.0, 400.0, 800.0);
        let l = Layout::for_screen(screen, screen, 1.0, src);
        let c = l.screen_to_canvas(l.game.center()).unwrap();
        assert!((c - src.center()).length() < 0.01, "{c:?}");
    }
}
