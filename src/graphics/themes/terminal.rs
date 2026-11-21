//! Terminal: green text on black, like the first versions of the game on a text terminal. The
//! board is the legacy text board; panels are ruled text frames (app::touch_view draws those
//! where `terminal` is set).

use macroquad::prelude::*;

use crate::graphics::style::{self, TERM_DIM, TERM_GREEN};
use crate::graphics::text::Face;
use crate::graphics::theme::{FxPalette, Theme, ThemeId};

const PALE: Color = Color::new(0.7, 1.0, 0.7, 1.0);

pub const THEME: Theme = Theme {
    id: ThemeId::Terminal,
    name: "TERMINAL",
    terminal: true,
    face: Face::Proggy,
    window: None,
    accents: [TERM_GREEN; 5],
    label_ink: Some(TERM_DIM),
    value: TERM_GREEN,
    hint: TERM_DIM,
    tip_on: TERM_GREEN,
    tip_off: TERM_DIM,
    shade: Color::new(0.0, 0.0, 0.0, 0.86),
    shade_light: Color::new(0.0, 0.0, 0.0, 0.7),
    rainbow: [TERM_GREEN; 6],
    surface: [BLACK, BLACK],
    pieces: Some([TERM_GREEN; 8]),
    backdrop,
    panel,
    well,
    well_edge,
    block: style::term_block,
    ghost,
    fx: FxPalette {
        word: [TERM_GREEN, PALE, TERM_GREEN],
        hot: [PALE, WHITE, PALE],
        edge: Color::new(0.0, 0.25, 0.0, 1.0),
        outline: BLACK,
        flash: TERM_GREEN,
        rings: [TERM_GREEN, TERM_DIM],
        dim: BLACK,
        banner: TERM_GREEN,
        banner_ink: BLACK,
        stamp: TERM_GREEN,
        heat: TERM_DIM,
        dust: TERM_DIM,
        tspin: TERM_GREEN,
        combo: [TERM_DIM, TERM_GREEN, PALE],
        perfect: TERM_GREEN,
        card_edge: TERM_GREEN,
        grey: Color::new(0.0, 0.3, 0.0, 1.0),
        card_ink: TERM_GREEN,
        card_label: PALE,
        chip: BLACK,
        block: style::term_block,
        panel,
    },
};

fn backdrop(_time: f32) {
    clear_background(BLACK);
}

fn panel(r: Rect, accent: Color, u: f32) {
    style::term_box(r, accent, 0.85, u);
}

fn well(well: Rect, _board: Rect, _cell: f32, _u: f32) {
    draw_rectangle(well.x, well.y, well.w, well.h, BLACK);
}

fn well_edge(well: Rect, u: f32) {
    style::term_box(well, TERM_GREEN, 0.0, u);
}

fn ghost(x: f32, y: f32, size: f32, _color: Color) {
    style::term_block(x, y, size, Color::new(0.0, 0.62, 0.0, 0.35));
}
