//! Tetris game board data structure

use crate::game::config::*;
use macroquad::prelude::Color;
use serde::{Serialize, Deserialize};

// Custom serialization module for macroquad Color
mod color_serde {
    use super::*;
    use serde::{Serializer, Deserializer, Deserialize};
    
    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (color.r, color.g, color.b, color.a).serialize(serializer)
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (r, g, b, a) = <(f32, f32, f32, f32)>::deserialize(deserializer)?;
        Ok(Color::new(r, g, b, a))
    }
}

/// Represents a single cell on the game board
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Cell {
    /// Empty cell
    Empty,
    /// Filled cell with a specific color
    Filled(#[serde(with = "color_serde")] Color),
}

impl Cell {
    /// Check if the cell is empty
    pub fn is_empty(self) -> bool {
        matches!(self, Cell::Empty)
    }
    
    /// Check if the cell is filled
    pub fn is_filled(self) -> bool {
        matches!(self, Cell::Filled(_))
    }
    
    /// Get the color of the cell if it's filled
    pub fn color(self) -> Option<Color> {
        match self {
            Cell::Empty => None,
            Cell::Filled(color) => Some(color),
        }
    }
}

/// The main Tetris game board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    /// The game grid - includes buffer rows above visible area
    grid: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT + BUFFER_HEIGHT],
    /// Lines cleared this game
    lines_cleared: u32,
    /// Current level
    level: u32,
}

impl Board {
    /// Create a new empty board
    pub fn new() -> Self {
        Self {
            grid: [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT + BUFFER_HEIGHT],
            lines_cleared: 0,
            level: 1,
        }
    }
    
    /// Get the cell at the specified position
    /// Returns None if coordinates are out of bounds
    pub fn get_cell(&self, x: i32, y: i32) -> Option<Cell> {
        if x < 0 || y < 0 {
            return None;
        }
        
        let x = x as usize;
        let y = y as usize;
        
        if x >= BOARD_WIDTH || y >= (BOARD_HEIGHT + BUFFER_HEIGHT) {
            return None;
        }
        
        Some(self.grid[y][x])
    }
    
    /// Set the cell at the specified position
    /// Returns false if coordinates are out of bounds
    pub fn set_cell(&mut self, x: i32, y: i32, cell: Cell) -> bool {
        if x < 0 || y < 0 {
            return false;
        }
        
        let x = x as usize;
        let y = y as usize;
        
        if x >= BOARD_WIDTH || y >= (BOARD_HEIGHT + BUFFER_HEIGHT) {
            return false;
        }
        
        self.grid[y][x] = cell;
        true
    }
    
    /// Check if a position is valid and empty
    pub fn is_position_valid(&self, x: i32, y: i32) -> bool {
        // Check bounds
        if x < 0 || x >= BOARD_WIDTH as i32 {
            return false;
        }
        
        // Allow pieces to spawn above the visible area
        if y < 0 {
            return true;
        }
        
        if y >= (BOARD_HEIGHT + BUFFER_HEIGHT) as i32 {
            return false;
        }
        
        // Check if cell is empty
        match self.get_cell(x, y) {
            Some(Cell::Empty) => true,
            _ => false,
        }
    }
    
    /// Check if a line is completely filled
    pub fn is_line_full(&self, y: usize) -> bool {
        if y >= (BOARD_HEIGHT + BUFFER_HEIGHT) {
            return false;
        }
        
        self.grid[y].iter().all(|cell| cell.is_filled())
    }
    
    /// Check if a line is completely empty
    pub fn is_line_empty(&self, y: usize) -> bool {
        if y >= (BOARD_HEIGHT + BUFFER_HEIGHT) {
            return false;
        }
        
        self.grid[y].iter().all(|cell| cell.is_empty())
    }
    
    /// Find all complete lines that need to be cleared
    pub fn find_complete_lines(&self) -> Vec<usize> {
        let mut complete_lines = Vec::new();
        
        // Only check visible area and buffer
        for y in 0..(BOARD_HEIGHT + BUFFER_HEIGHT) {
            if self.is_line_full(y) {
                complete_lines.push(y);
            }
        }
        
        complete_lines
    }
    
    /// Clear the specified lines and drop rows above
    pub fn clear_lines(&mut self, lines_to_clear: &[usize]) -> u32 {
        if lines_to_clear.is_empty() {
            return 0;
        }
        
        let lines_cleared_count = lines_to_clear.len() as u32;
        
        // Sort lines in ascending order
        let mut sorted_lines = lines_to_clear.to_vec();
        sorted_lines.sort();
        
        // Create a new grid by copying non-cleared lines
        let mut new_grid = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT + BUFFER_HEIGHT];
        let mut new_y = (BOARD_HEIGHT + BUFFER_HEIGHT) - 1; // Start from bottom
        
        // Copy lines from bottom to top, skipping cleared lines
        for y in (0..(BOARD_HEIGHT + BUFFER_HEIGHT)).rev() {
            if !sorted_lines.contains(&y) {
                // This line is not being cleared, copy it
                new_grid[new_y] = self.grid[y];
                if new_y > 0 {
                    new_y -= 1;
                }
            }
            // If this line is being cleared, skip it (don't copy)
        }
        
        // Replace the old grid with the new one
        self.grid = new_grid;
        
        // Update statistics
        self.lines_cleared += lines_cleared_count;
        self.level = (self.lines_cleared / LINES_PER_LEVEL) + 1;
        
        lines_cleared_count
    }
    
    /// Get the current level
    pub fn level(&self) -> u32 {
        self.level
    }
    
    /// Get the total number of lines cleared
    pub fn lines_cleared(&self) -> u32 {
        self.lines_cleared
    }
    
    /// Check if the game is over (pieces have reached the top)
    pub fn is_game_over(&self) -> bool {
        // Check if any cells in the spawn area (buffer zone) are filled
        for y in 0..BUFFER_HEIGHT {
            for x in 0..BOARD_WIDTH {
                if self.grid[y][x].is_filled() {
                    return true;
                }
            }
        }
        false
    }
    
    /// Clear the entire board
    pub fn clear(&mut self) {
        self.grid = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT + BUFFER_HEIGHT];
        self.lines_cleared = 0;
        self.level = 1;
    }
    
    /// Get the height of the highest filled cell in a column
    pub fn column_height(&self, x: usize) -> usize {
        if x >= BOARD_WIDTH {
            return 0;
        }
        
        for y in 0..(BOARD_HEIGHT + BUFFER_HEIGHT) {
            if self.grid[y][x].is_filled() {
                return (BOARD_HEIGHT + BUFFER_HEIGHT) - y;
            }
        }
        
        0 // Column is empty
    }
    
    /// Get the total number of filled cells
    pub fn filled_cells_count(&self) -> usize {
        let mut count = 0;
        for row in &self.grid {
            for cell in row {
                if cell.is_filled() {
                    count += 1;
                }
            }
        }
        count
    }
    
    /// Create a debug representation of the board
    pub fn debug_string(&self) -> String {
        let mut result = String::new();
        
        // Only show visible area for debugging
        for y in BUFFER_HEIGHT..(BOARD_HEIGHT + BUFFER_HEIGHT) {
            result.push('|');
            for x in 0..BOARD_WIDTH {
                match self.grid[y][x] {
                    Cell::Empty => result.push(' '),
                    Cell::Filled(_) => result.push('#'),
                }
            }
            result.push_str("|\n");
        }
        
        // Add bottom border
        result.push('+');
        for _ in 0..BOARD_WIDTH {
            result.push('-');
        }
        result.push('+');
        
        result
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::colors::*;

    #[test]
    fn test_new_board() {
        let board = Board::new();
        assert_eq!(board.level(), 1);
        assert_eq!(board.lines_cleared(), 0);
        assert_eq!(board.filled_cells_count(), 0);
        assert!(!board.is_game_over());
    }

    #[test]
    fn test_cell_operations() {
        let mut board = Board::new();
        let test_color = TETROMINO_I;
        
        // Test setting and getting cells
        assert!(board.set_cell(5, 10, Cell::Filled(test_color)));
        
        let cell = board.get_cell(5, 10).unwrap();
        assert_eq!(cell, Cell::Filled(test_color));
        assert!(cell.is_filled());
        assert!(!cell.is_empty());
        assert_eq!(cell.color(), Some(test_color));
        
        // Test bounds checking
        assert!(!board.set_cell(-1, 10, Cell::Filled(test_color)));
        assert!(!board.set_cell(10, 10, Cell::Filled(test_color)));
        assert_eq!(board.get_cell(-1, 10), None);
        assert_eq!(board.get_cell(10, 10), None);
    }

    #[test]
    fn test_position_validation() {
        let mut board = Board::new();
        let test_color = TETROMINO_T;
        
        // Empty position should be valid
        assert!(board.is_position_valid(5, 10));
        
        // Fill a cell
        board.set_cell(5, 10, Cell::Filled(test_color));
        
        // Filled position should not be valid
        assert!(!board.is_position_valid(5, 10));
        
        // Out of bounds positions should not be valid
        assert!(!board.is_position_valid(-1, 10));
        assert!(!board.is_position_valid(10, 10));
        
        // Above visible area should be valid (spawn area)
        assert!(board.is_position_valid(5, -1));
    }

    #[test]
    fn test_line_operations() {
        let mut board = Board::new();
        let test_color = TETROMINO_S;
        
        // Empty line tests
        assert!(board.is_line_empty(23));
        assert!(!board.is_line_full(23));
        
        // Fill some cells in line 23
        for x in 0..5 {
            board.set_cell(x, 23, Cell::Filled(test_color));
        }
        
        // Partially filled line
        assert!(!board.is_line_empty(23));
        assert!(!board.is_line_full(23));
        
        // Fill the entire line
        for x in 5..10 {
            board.set_cell(x, 23, Cell::Filled(test_color));
        }
        
        // Full line
        assert!(!board.is_line_empty(23));
        assert!(board.is_line_full(23));
    }

    #[test]
    fn test_line_clearing() {
        let mut board = Board::new();
        let test_color = TETROMINO_Z;
        
        // Fill two complete lines
        for x in 0..10 {
            board.set_cell(x, 22, Cell::Filled(test_color));
            board.set_cell(x, 23, Cell::Filled(test_color));
        }
        
        // Add a block above the complete lines
        board.set_cell(0, 21, Cell::Filled(test_color));
        
        assert_eq!(board.filled_cells_count(), 21); // 20 + 1
        
        // Find and clear complete lines
        let complete_lines = board.find_complete_lines();
        assert_eq!(complete_lines.len(), 2);
        assert!(complete_lines.contains(&22));
        assert!(complete_lines.contains(&23));
        
        let lines_cleared = board.clear_lines(&complete_lines);
        assert_eq!(lines_cleared, 2);
        assert_eq!(board.lines_cleared(), 2);
        assert_eq!(board.level(), 1); // Still level 1 (need 10 lines for level 2)
        
        // The block that was at (0, 21) should now be at (0, 23)
        assert_eq!(board.get_cell(0, 23).unwrap(), Cell::Filled(test_color));
        
        // After clearing 2 complete lines (20 blocks), we should have 1 block remaining
        assert_eq!(board.filled_cells_count(), 1);
    }

    #[test]
    fn test_column_height() {
        let mut board = Board::new();
        let test_color = TETROMINO_J;
        
        // Empty column should have height 0
        assert_eq!(board.column_height(5), 0);
        
        // Add blocks to column 5
        board.set_cell(5, 23, Cell::Filled(test_color)); // Bottom
        board.set_cell(5, 22, Cell::Filled(test_color)); // Middle
        board.set_cell(5, 20, Cell::Filled(test_color)); // Top (with gap)
        
        // Height should be from top filled cell to bottom
        let expected_height = (BOARD_HEIGHT + BUFFER_HEIGHT) - 20; // 24 - 20 = 4
        assert_eq!(board.column_height(5), expected_height);
    }

    #[test]
    fn test_game_over() {
        let mut board = Board::new();
        let test_color = TETROMINO_L;
        
        // Game should not be over initially
        assert!(!board.is_game_over());
        
        // Fill a cell in the buffer area (spawn area)
        board.set_cell(5, 2, Cell::Filled(test_color)); // Buffer area
        
        // Game should now be over
        assert!(board.is_game_over());
    }

    #[test]
    fn test_board_clear() {
        let mut board = Board::new();
        let test_color = TETROMINO_O;
        
        // Add some blocks and statistics
        board.set_cell(0, 23, Cell::Filled(test_color));
        board.set_cell(1, 23, Cell::Filled(test_color));
        
        // Simulate some lines cleared
        for x in 0..10 {
            board.set_cell(x, 22, Cell::Filled(test_color));
        }
        let complete_lines = board.find_complete_lines();
        board.clear_lines(&complete_lines);
        
        // Verify state before clear
        assert!(board.filled_cells_count() > 0);
        assert!(board.lines_cleared() > 0);
        
        // Clear the board
        board.clear();
        
        // Verify everything is reset
        assert_eq!(board.filled_cells_count(), 0);
        assert_eq!(board.lines_cleared(), 0);
        assert_eq!(board.level(), 1);
        assert!(!board.is_game_over());
    }
}
