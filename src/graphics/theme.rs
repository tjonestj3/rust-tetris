//! Themes: the whole look of the touch screen. Each theme is one file in `themes/` and one entry
//! in [`ALL`]; nothing else changes to add one (docs/superpowers/specs/2026-11-20-themes-design.md).
//!
//! A theme is colours plus the functions that draw the parts that differ in shape: the backdrop,
//! panels, the board's well, the blocks. The effects (app::fx) draw from its [`FxPalette`].

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use super::themes;

/// Which theme. Saved in the settings by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ThemeId {
    #[default]
    Neon,
    Terminal,
}

/// The accent colours a screen asks for by name; each theme says what they are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Accent {
    Cyan,
    Pink,
    Gold,
    Green,
    Violet,
}

pub struct Theme {
    pub id: ThemeId,
    /// As shown in Settings.
    pub name: &'static str,
    /// Terminal-style text frames and the legacy text board (the Terminal theme).
    pub terminal: bool,
    /// Cyan, pink, gold, green, violet, in [`Accent`] order.
    pub accents: [Color; 5],
    /// Every label in this colour, whatever its accent (`None`: labels take their accent).
    pub label_ink: Option<Color>,
    /// Numbers and other values.
    pub value: Color,
    /// Quiet hints ("TAP TO HOLD").
    pub hint: Color,
    /// The ghost-block tip ("TAP TO AIM"): blinking on, and off.
    pub tip_on: Color,
    pub tip_off: Color,
    /// Behind the pause screen, and the lighter one behind the countdown.
    pub shade: Color,
    pub shade_light: Color,
    /// Every piece in this colour (`None`: each in its own).
    pub pieces: Option<Color>,
    /// Titles' letters, in turn (PAUSED, GET READY).
    pub rainbow: [Color; 6],
    /// The whole screen behind everything, given the time (for slow movement).
    pub backdrop: fn(time: f32),
    /// A panel's frame (Next, Hold, score...), in its accent; `u` is the layout's unit.
    pub panel: fn(r: Rect, accent: Color, u: f32),
    /// The board's well: `well` (the board plus a margin) behind the grid of `board`.
    pub well: fn(well: Rect, board: Rect, cell: f32, u: f32),
    /// The well's edge, drawn over the blocks.
    pub well_edge: fn(well: Rect, u: f32),
    pub block: fn(x: f32, y: f32, size: f32, color: Color),
    /// Where the falling piece will land.
    pub ghost: fn(x: f32, y: f32, size: f32, color: Color),
    pub fx: FxPalette,
}

impl Theme {
    pub fn accent(&self, a: Accent) -> Color {
        self.accents[a as usize]
    }

    /// A label's colour for `a`.
    pub fn label(&self, a: Accent) -> Color {
        self.label_ink.unwrap_or_else(|| self.accent(a))
    }

    /// A piece's colour in this theme.
    pub fn piece(&self, color: Color) -> Color {
        self.pieces.unwrap_or(color)
    }
}

/// Every theme, in the order Settings cycles through them.
pub const ALL: [ThemeId; 2] = [ThemeId::Neon, ThemeId::Terminal];

pub fn get(id: ThemeId) -> &'static Theme {
    match id {
        ThemeId::Neon => &themes::neon::THEME,
        ThemeId::Terminal => &themes::terminal::THEME,
    }
}

/// Every colour the effects (app::fx) draw with, and how they draw a block.
#[derive(Debug, Clone, Copy)]
pub struct FxPalette {
    /// JONES'IN!'s letters, left to right.
    pub word: [Color; 3],
    /// The letters for a back-to-back Tetris.
    pub hot: [Color; 3],
    /// The letters' raised edge, and their outline.
    pub edge: Color,
    pub outline: Color,
    /// The screen flash, the shockwave rings and the dim behind the word.
    pub flash: Color,
    pub rings: [Color; 2],
    pub dim: Color,
    /// BACK-TO-BACK's banner and its text; the X2 stamp.
    pub banner: Color,
    pub banner_ink: Color,
    pub stamp: Color,
    /// The heat rising behind a back-to-back word.
    pub heat: Color,
    /// Dust where a hard-dropped piece lands.
    pub dust: Color,
    /// T-SPIN's banner and the T's glow.
    pub tspin: Color,
    /// The combo chip, from a small combo to a big one.
    pub combo: [Color; 3],
    /// PERFECT CLEAR and its light.
    pub perfect: Color,
    /// The game-over card's edge.
    pub card_edge: Color,
    /// What a finished game's blocks fade to.
    pub grey: Color,
    /// A block (the ghost block's comet).
    pub block: fn(x: f32, y: f32, size: f32, color: Color),
    /// A card's frame (the game-over card).
    pub panel: fn(r: Rect, accent: Color, u: f32),
}

impl Default for FxPalette {
    fn default() -> Self {
        themes::neon::THEME.fx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_theme_is_listed_and_knows_its_id() {
        for id in ALL {
            assert_eq!(get(id).id, id);
        }
    }

    #[test]
    fn theme_ids_save_by_name() {
        assert_eq!(serde_json::to_string(&ThemeId::Terminal).unwrap(), "\"Terminal\"");
        let back: ThemeId = serde_json::from_str("\"Neon\"").unwrap();
        assert_eq!(back, ThemeId::Neon);
    }
}
