//! Tetris game board data structure

/// Game board struct (placeholder for Phase 1)
#[derive(Debug)]
pub struct Board {
    // Will be implemented in next todo
}

impl Board {
    /// Create a new empty board
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}