//! Neon: the glass look the game has had: glossy blocks, glowing glass panels, an indigo to
//! magenta backdrop.

use macroquad::prelude::*;

use crate::graphics::style::{self, alpha, CYAN, GOLD, GREEN, INK, PINK, VIOLET};
use crate::graphics::text::Face;
use crate::graphics::theme::{FxPalette, Theme, ThemeId};

const ORANGE: Color = Color::new(1.0, 0.54, 0.24, 1.0);

pub const THEME: Theme = Theme {
    id: ThemeId::Neon,
    name: "NEON",
    terminal: false,
    face: Face::Proggy,
    window: None,
    accents: [CYAN, PINK, GOLD, GREEN, VIOLET],
    label_ink: None,
    value: INK,
    hint: Color::new(PINK.r, PINK.g, PINK.b, 0.7),
    tip_on: Color::new(0.82, 0.74, 1.0, 1.0),
    tip_off: Color::new(0.82, 0.74, 1.0, 1.0),
    shade: Color::new(0.05, 0.02, 0.16, 0.80),
    shade_light: Color::new(0.05, 0.02, 0.16, 0.55),
    rainbow: [CYAN, GREEN, GOLD, PINK, VIOLET, Color::new(1.0, 0.55, 0.25, 1.0)],
    surface: [Color::new(0.13, 0.10, 0.34, 1.0), Color::new(0.06, 0.04, 0.18, 1.0)],
    pieces: None,
    backdrop: style::backdrop,
    panel: style::panel,
    well,
    well_edge,
    block: style::block,
    ghost: style::ghost_block,
    fx: FxPalette {
        word: [CYAN, PINK, GOLD],
        hot: [GOLD, ORANGE, PINK],
        edge: Color::new(0.16, 0.08, 0.38, 1.0),
        outline: Color::new(0.04, 0.02, 0.12, 1.0),
        flash: WHITE,
        rings: [CYAN, PINK],
        dim: Color::new(0.04, 0.02, 0.12, 1.0),
        banner: PINK,
        banner_ink: Color::new(0.08, 0.02, 0.18, 1.0),
        stamp: GOLD,
        heat: ORANGE,
        dust: Color::new(0.90, 0.86, 1.0, 1.0),
        tspin: VIOLET,
        combo: [CYAN, GOLD, PINK],
        perfect: GOLD,
        card_edge: PINK,
        grey: Color::new(0.42, 0.40, 0.52, 1.0),
        card_ink: WHITE,
        card_label: GOLD,
        chip: Color::new(0.04, 0.02, 0.12, 1.0),
        block: style::block,
        panel: style::panel,
    },
};

const RADIUS: f32 = 8.0;

/// A dark glass well with a glow, faint column stripes and row lines.
fn well(well: Rect, board: Rect, cell: f32, u: f32) {
    let radius = RADIUS * u;
    style::halo(well, radius, 10.0 * u, CYAN, 0.16);
    style::round_rect(well, radius, Color::new(0.06, 0.05, 0.19, 0.86), Color::new(0.10, 0.04, 0.22, 0.90));
    let cols = (board.w / cell).round() as usize;
    let rows = (board.h / cell).round() as usize;
    for x in 0..cols {
        if x % 2 == 1 {
            draw_rectangle(board.x + x as f32 * cell, board.y, cell, board.h, Color::new(1.0, 1.0, 1.0, 0.025));
        }
    }
    for y in 1..rows {
        draw_rectangle(board.x, (board.y + y as f32 * cell).round(), board.w, 1.0, Color::new(0.8, 0.85, 1.0, 0.06));
    }
}

fn well_edge(well: Rect, u: f32) {
    style::round_outline(well, RADIUS * u, (2.0 * u).max(1.0), alpha(CYAN, 0.85));
}
