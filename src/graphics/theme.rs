//! Themes: the whole look of the touch screen. Each theme is one file in `themes/` and one entry
//! in [`ALL`]; nothing else changes to add one (docs/superpowers/specs/2026-11-20-themes-design.md).
//!
//! A theme is colours plus the functions that draw the parts that differ in shape: the backdrop,
//! panels, the board's well, the blocks. The effects (app::fx) draw from its [`FxPalette`].

use macroquad::prelude::*;
use serde::{Deserialize, Serialize};

use super::text::Face;
use super::themes;
use crate::tetromino::TetrominoType;

/// Which theme. Saved in the settings by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ThemeId {
    #[default]
    Neon,
    Terminal,
    Arcade,
    Desktop98,
    MacOsX,
    Linux,
    Glass,
    WarmPaper,
    Chess,
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

/// Draws a window titled `title` in `r` and returns the area inside it, below the title bar.
/// `size` is the label text size; `u` the layout's unit.
pub type WindowFn = fn(r: Rect, title: &str, accent: Color, size: f32, u: f32) -> Rect;

pub struct Theme {
    pub id: ThemeId,
    /// As shown in Settings.
    pub name: &'static str,
    /// Terminal-style text frames and the legacy text board (the Terminal theme).
    pub terminal: bool,
    /// The font everything on the play screen is drawn in.
    pub face: Face,
    /// Panels drawn as windows with a title bar (`None`: plain panels with their label inside):
    /// draws the window titled `title` and returns the area inside it, below the title bar.
    /// `size` is the label text size.
    pub window: Option<WindowFn>,
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
    /// The pieces' colours in this theme, in [`TetrominoType::all`] order, then the ghost
    /// block's (`None`: the standard colours). Blocks on the board are recoloured by matching
    /// their stored colour to the standard ones, so saved games pick up the theme too.
    pub pieces: Option<[Color; 8]>,
    /// Titles' letters, in turn (PAUSED, GET READY).
    pub rainbow: [Color; 6],
    /// Small cards over the board (a callout, the aim banner, the gesture hint): top and bottom
    /// of their fill, before their alpha.
    pub surface: [Color; 2],
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

    /// A block's colour in this theme: a standard piece colour (or the ghost block's) becomes
    /// the theme's; anything else stays as it is.
    pub fn piece(&self, color: Color) -> Color {
        let Some(palette) = &self.pieces else { return color };
        match standard_index(color) {
            Some(i) => palette[i].with_alpha(color.a),
            None => color,
        }
    }
}

/// The colour a ghost block is placed in (Game::finish_ghost_throw).
pub const GHOST_BLOCK: Color = Color::new(0.8, 0.8, 1.0, 1.0);

/// Which standard colour `c` is: a piece type's (in [`TetrominoType::all`] order), or 7 for the
/// ghost block's. Alpha doesn't count.
fn standard_index(c: Color) -> Option<usize> {
    let same = |a: Color, b: Color| (a.r - b.r).abs() < 0.01 && (a.g - b.g).abs() < 0.01 && (a.b - b.b).abs() < 0.01;
    TetrominoType::all()
        .iter()
        .position(|t| same(t.color(), c))
        .or_else(|| same(GHOST_BLOCK, c).then_some(7))
}

/// Every theme, in the order Settings cycles through them.
pub const ALL: [ThemeId; 9] = [
    ThemeId::Neon,
    ThemeId::Terminal,
    ThemeId::Arcade,
    ThemeId::Desktop98,
    ThemeId::MacOsX,
    ThemeId::Linux,
    ThemeId::Glass,
    ThemeId::WarmPaper,
    ThemeId::Chess,
];

/// The theme after `id` in [`ALL`], round to the first.
pub fn next(id: ThemeId) -> ThemeId {
    let i = ALL.iter().position(|&t| t == id).unwrap_or(0);
    ALL[(i + 1) % ALL.len()]
}

pub fn get(id: ThemeId) -> &'static Theme {
    match id {
        ThemeId::Neon => &themes::neon::THEME,
        ThemeId::Terminal => &themes::terminal::THEME,
        ThemeId::Arcade => &themes::arcade::THEME,
        ThemeId::Desktop98 => &themes::desktop98::THEME,
        ThemeId::MacOsX => &themes::macosx::THEME,
        ThemeId::Linux => &themes::linux::THEME,
        ThemeId::Glass => &themes::glass::THEME,
        ThemeId::WarmPaper => &themes::warm_paper::THEME,
        ThemeId::Chess => &themes::chess::THEME,
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
    /// Text on the game-over card and the combo chip, and the card's label.
    pub card_ink: Color,
    pub card_label: Color,
    /// The combo chip's fill.
    pub chip: Color,
    /// A block (the ghost block's comet).
    pub block: fn(x: f32, y: f32, size: f32, color: Color),
    /// A card's frame (the game-over card).
    pub panel: fn(r: Rect, accent: Color, u: f32),
    /// The theme's window, for effects drawn as one (a dialog, a terminal window).
    pub window: Option<WindowFn>,
    /// Which version of each effect the theme uses.
    pub styles: FxStyles,
}

/// Which version of each effect a theme uses (docs/superpowers/specs/2026-11-20-themes-design.md,
/// "Animations are modular too"). The effects are in app::fx; a theme names its picks and takes
/// [`DEFAULT_STYLES`] for the rest.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FxStyles {
    pub jonesin: JonesinStyle,
    /// How a label like T-SPIN DOUBLE crosses the board.
    pub label: LabelStyle,
    /// What a T-spin is called: no lines, then single, double, triple.
    pub tspin_names: [&'static str; 4],
}

pub const DEFAULT_STYLES: FxStyles = FxStyles {
    jonesin: JonesinStyle::Slam,
    label: LabelStyle::Slant,
    tspin_names: ["T-SPIN", "T-SPIN SINGLE", "T-SPIN DOUBLE", "T-SPIN TRIPLE"],
};

/// The ways JONES'IN! (a Tetris) plays out. Each lands its big moment with the fanfare's chord.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JonesinStyle {
    /// Letters slam down with a raised edge, shockwaves and confetti.
    Slam,
    /// Typed at a prompt in big block text, a cursor, a scanline flash.
    Typed,
    /// A command typed in a terminal window, the word printed big.
    Shell,
    /// A lit sign: chasing bulbs, letters flickering on like neon tubes, a CRT switch-off.
    Marquee,
    /// A message box that bounces off and leaves a trail of copies behind it.
    Dialog,
    /// Glossy tiles that bounce in like icons in a dock, and get sucked away.
    Bounce,
    /// A rubber stamp pressed down in ink.
    Stamp,
    /// A wooden plaque with the word carved and gilded.
    Plaque,
    /// A frosted card, the letters fading in, soft colour blooming behind.
    Bloom,
}

/// The ways a label (T-SPIN DOUBLE) crosses the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelStyle {
    /// A slanted bar slides in and out.
    Slant,
    /// Typed at a prompt, then deleted.
    Typed,
    /// Flat pixel text with a hard shadow that blinks in.
    Pixel,
    /// A small window slides in.
    Window,
    /// A wooden plank with brass letters.
    Plank,
    /// An ink stamp.
    Stamp,
    /// A frosted pill.
    Pill,
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
    fn next_goes_round_every_theme() {
        let mut id = ALL[0];
        for _ in 0..ALL.len() {
            id = next(id);
        }
        assert_eq!(id, ALL[0]);
    }

    #[test]
    fn themes_recolour_the_standard_pieces_and_leave_the_rest() {
        let paper = get(ThemeId::WarmPaper);
        let i = TetrominoType::I.color();
        assert_ne!(paper.piece(i), i);
        assert_eq!(paper.piece(i.with_alpha(0.4)).a, 0.4, "alpha kept");
        assert_ne!(paper.piece(GHOST_BLOCK), GHOST_BLOCK);
        let odd = Color::new(0.5, 0.4, 0.9, 1.0);
        assert_eq!(paper.piece(odd), odd);
        assert_eq!(get(ThemeId::Neon).piece(i), i);
        // Every theme with its own colours gives each piece type a different one.
        for id in ALL {
            if let Some(p) = &get(id).pieces {
                if id == ThemeId::Terminal {
                    continue;
                }
                for a in 0..7 {
                    for b in a + 1..7 {
                        assert_ne!(p[a], p[b], "{id:?}: pieces {a} and {b} look the same");
                    }
                }
            }
        }
    }

    #[test]
    fn theme_ids_save_by_name() {
        assert_eq!(serde_json::to_string(&ThemeId::Terminal).unwrap(), "\"Terminal\"");
        let back: ThemeId = serde_json::from_str("\"Neon\"").unwrap();
        assert_eq!(back, ThemeId::Neon);
    }
}
