//! Tetromino type definitions

use crate::graphics::colors::*;
use macroquad::prelude::Color;
use rand::Rng;
use serde::{Serialize, Deserialize};

/// Seven standard Tetris pieces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TetrominoType {
    I, // Line piece (cyan)
    O, // Square piece (yellow)
    T, // T-piece (purple)
    S, // S-piece (green)
    Z, // Z-piece (red)
    J, // J-piece (blue)
    L, // L-piece (orange)
}

impl TetrominoType {
    /// Get all tetromino types as an array
    pub fn all() -> [TetrominoType; 7] {
        [TetrominoType::I, TetrominoType::O, TetrominoType::T, 
         TetrominoType::S, TetrominoType::Z, TetrominoType::J, 
         TetrominoType::L]
    }
    
    /// Generate a random tetromino type
    pub fn random() -> TetrominoType {
        let types = Self::all();
        let mut rng = rand::thread_rng();
        types[rng.gen_range(0..types.len())]
    }
    
    /// Get the color associated with this tetromino type
    pub fn color(self) -> Color {
        match self {
            TetrominoType::I => TETROMINO_I,
            TetrominoType::O => TETROMINO_O,
            TetrominoType::T => TETROMINO_T,
            TetrominoType::S => TETROMINO_S,
            TetrominoType::Z => TETROMINO_Z,
            TetrominoType::J => TETROMINO_J,
            TetrominoType::L => TETROMINO_L,
        }
    }
    
    /// Get the name of the tetromino
    pub fn name(self) -> &'static str {
        match self {
            TetrominoType::I => "I-piece",
            TetrominoType::O => "O-piece",
            TetrominoType::T => "T-piece",
            TetrominoType::S => "S-piece",
            TetrominoType::Z => "Z-piece",
            TetrominoType::J => "J-piece",
            TetrominoType::L => "L-piece",
        }
    }
}

/// Represents a tetromino piece in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tetromino {
    /// The type of tetromino
    pub piece_type: TetrominoType,
    /// Current position (x, y) of the piece center
    pub position: (i32, i32),
    /// Current rotation state (0-3)
    pub rotation: u8,
    /// The blocks that make up this piece (relative to position)
    pub blocks: Vec<(i32, i32)>,
}

impl Tetromino {
    /// Create a new tetromino at the spawn position
    pub fn new(piece_type: TetrominoType) -> Self {
        let mut tetromino = Self {
            piece_type,
            position: (4, 2), // Start lower in buffer area for visibility
            rotation: 0,
            blocks: Vec::new(),
        };
        tetromino.update_blocks();
        tetromino
    }
    
    /// Create a random tetromino
    pub fn random() -> Self {
        Self::new(TetrominoType::random())
    }
    
    /// Update the blocks array based on current type and rotation
    fn update_blocks(&mut self) {
        self.blocks = crate::tetromino::data::get_tetromino_blocks(self.piece_type, self.rotation);
    }
    
    /// Get the absolute positions of all blocks
    pub fn absolute_blocks(&self) -> Vec<(i32, i32)> {
        self.blocks.iter()
            .map(|(dx, dy)| (self.position.0 + dx, self.position.1 + dy))
            .collect()
    }
    
    /// Move the tetromino by the specified offset
    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.position.0 += dx;
        self.position.1 += dy;
    }
    
    /// Rotate the tetromino clockwise
    pub fn rotate_clockwise(&mut self) {
        self.rotation = (self.rotation + 1) % 4;
        self.update_blocks();
    }
    
    /// Rotate the tetromino counterclockwise
    pub fn rotate_counterclockwise(&mut self) {
        self.rotation = if self.rotation == 0 { 3 } else { self.rotation - 1 };
        self.update_blocks();
    }
    
    /// Get the color of this tetromino
    pub fn color(&self) -> Color {
        self.piece_type.color()
    }
    
    /// Reset position to spawn point
    pub fn reset_position(&mut self) {
        self.position = (4, 2);
    }
    
    /// Get the bounding box of the tetromino (min_x, min_y, max_x, max_y)
    pub fn bounding_box(&self) -> (i32, i32, i32, i32) {
        let abs_blocks = self.absolute_blocks();
        if abs_blocks.is_empty() {
            return (0, 0, 0, 0);
        }
        
        let min_x = abs_blocks.iter().map(|(x, _)| *x).min().unwrap();
        let min_y = abs_blocks.iter().map(|(_, y)| *y).min().unwrap();
        let max_x = abs_blocks.iter().map(|(x, _)| *x).max().unwrap();
        let max_y = abs_blocks.iter().map(|(_, y)| *y).max().unwrap();
        
        (min_x, min_y, max_x, max_y)
    }
}

impl Default for Tetromino {
    fn default() -> Self {
        Self::new(TetrominoType::T)
    }
}
