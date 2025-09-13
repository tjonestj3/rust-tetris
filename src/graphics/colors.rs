//! Color definitions for the Tetris game

use macroquad::prelude::Color;

/// Background colors
pub const BACKGROUND_COLOR: Color = Color::new(0.1, 0.1, 0.1, 1.0);
pub const BOARD_BACKGROUND: Color = Color::new(0.05, 0.05, 0.05, 1.0);

/// Grid colors
pub const GRID_LINE_COLOR: Color = Color::new(0.3, 0.3, 0.3, 1.0);
pub const BOARD_BORDER_COLOR: Color = Color::new(0.6, 0.6, 0.6, 1.0);

/// Text colors
pub const TEXT_COLOR: Color = Color::new(1.0, 1.0, 1.0, 1.0);
pub const TEXT_SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.5);

/// UI colors
pub const UI_BACKGROUND: Color = Color::new(0.2, 0.2, 0.2, 0.8);
pub const UI_BORDER: Color = Color::new(0.4, 0.4, 0.4, 1.0);

/// Tetromino piece colors (standard Tetris colors)
pub const TETROMINO_I: Color = Color::new(0.0, 1.0, 1.0, 1.0);  // Cyan
pub const TETROMINO_O: Color = Color::new(1.0, 1.0, 0.0, 1.0);  // Yellow
pub const TETROMINO_T: Color = Color::new(0.5, 0.0, 1.0, 1.0);  // Purple
pub const TETROMINO_S: Color = Color::new(0.0, 1.0, 0.0, 1.0);  // Green
pub const TETROMINO_Z: Color = Color::new(1.0, 0.0, 0.0, 1.0);  // Red
pub const TETROMINO_J: Color = Color::new(0.0, 0.0, 1.0, 1.0);  // Blue
pub const TETROMINO_L: Color = Color::new(1.0, 0.65, 0.0, 1.0); // Orange

/// Ghost piece color (translucent version of active piece)
pub const GHOST_PIECE_ALPHA: f32 = 0.3;

/// Special effect colors
pub const LINE_CLEAR_FLASH: Color = Color::new(1.0, 1.0, 1.0, 0.8);
pub const GAME_OVER_OVERLAY: Color = Color::new(0.0, 0.0, 0.0, 0.7);

/// Get the color for a specific tetromino type
pub fn get_tetromino_color(piece_type: &crate::tetromino::TetrominoType) -> Color {
    match piece_type {
        crate::tetromino::TetrominoType::I => TETROMINO_I,
        crate::tetromino::TetrominoType::O => TETROMINO_O,
        crate::tetromino::TetrominoType::T => TETROMINO_T,
        crate::tetromino::TetrominoType::S => TETROMINO_S,
        crate::tetromino::TetrominoType::Z => TETROMINO_Z,
        crate::tetromino::TetrominoType::J => TETROMINO_J,
        crate::tetromino::TetrominoType::L => TETROMINO_L,
    }
}

/// Create a ghost version of a color (more transparent)
pub fn make_ghost_color(color: Color) -> Color {
    Color::new(color.r, color.g, color.b, GHOST_PIECE_ALPHA)
}