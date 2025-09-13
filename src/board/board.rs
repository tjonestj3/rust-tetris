//! Tetris game board data structure

use crate::game::config::*;
use macroquad::prelude::Color;

/// Represents a single cell on the game board
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    /// Empty cell
    Empty,
    /// Filled cell with a specific color
    Filled(Color),
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
#[derive(Debug, Clone)]
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
        
        let lines_cleared = lines_to_clear.len() as u32;
        
        // Sort lines in descending order to clear from bottom to top
        let mut sorted_lines = lines_to_clear.to_vec();
        sorted_lines.sort_by(|a, b| b.cmp(a));
        
        // Clear each line and drop rows above
        for &line_y in &sorted_lines {
            // Move all rows above down by one
            for y in (1..=line_y).rev() {
                self.grid[y] = self.grid[y - 1];
            }
            
            // Clear the top row
            self.grid[0] = [Cell::Empty; BOARD_WIDTH];
        }
        
        // Update statistics
        self.lines_cleared += lines_cleared;
        self.level = (self.lines_cleared / LINES_PER_LEVEL) + 1;
        
        lines_cleared
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
