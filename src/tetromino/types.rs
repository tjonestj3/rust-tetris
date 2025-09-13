//! Tetromino type definitions

/// Seven standard Tetris pieces
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TetrominoType {
    I, // Line piece
    O, // Square piece
    T, // T-piece
    S, // S-piece
    Z, // Z-piece
    J, // J-piece
    L, // L-piece
}

/// Tetromino piece struct (placeholder for Phase 1)
#[derive(Debug)]
pub struct Tetromino {
    // Will be implemented in later todo
}

impl Tetromino {
    /// Create a new tetromino
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Tetromino {
    fn default() -> Self {
        Self::new()
    }
}