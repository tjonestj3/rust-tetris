//! Perfect Clear (All Clear) detection for Tetris
//!
//! This module handles detection of Perfect Clears (when all blocks are cleared 
//! from the board) and determines the appropriate bonus type.

use crate::board::Board;
use super::PerfectClearType;

/// Perfect Clear detector
pub struct PerfectClearDetector;

impl PerfectClearDetector {
    /// Check if the board is completely empty (Perfect Clear achieved)
    pub fn is_perfect_clear(board: &Board) -> bool {
        // Check if there are any filled cells in the visible area
        for y in crate::game::config::BUFFER_HEIGHT..(crate::game::config::BOARD_HEIGHT + crate::game::config::BUFFER_HEIGHT) {
            for x in 0..crate::game::config::BOARD_WIDTH {
                if let Some(cell) = board.get_cell(x as i32, y as i32) {
                    if cell.is_filled() {
                        return false;
                    }
                }
            }
        }
        true
    }
    
    /// Determine the type of Perfect Clear based on the number of lines cleared
    pub fn determine_perfect_clear_type(lines_cleared: u32) -> Option<PerfectClearType> {
        match lines_cleared {
            0 => None, // No lines cleared, can't be a perfect clear
            1 => Some(PerfectClearType::Single),
            2 => Some(PerfectClearType::Double),
            3 => Some(PerfectClearType::Triple),
            4 => Some(PerfectClearType::Tetris),
            _ => None, // Invalid number of lines
        }
    }
    
    /// Check for Perfect Clear and return the type if achieved
    pub fn check_perfect_clear(board: &Board, lines_cleared: u32) -> Option<PerfectClearType> {
        if Self::is_perfect_clear(board) && lines_cleared > 0 {
            Self::determine_perfect_clear_type(lines_cleared)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::{Board, Cell};
    use macroquad::prelude::Color;
    
    #[test]
    fn test_empty_board_is_perfect_clear() {
        let board = Board::new();
        assert!(PerfectClearDetector::is_perfect_clear(&board));
    }
    
    #[test]
    fn test_board_with_blocks_is_not_perfect_clear() {
        let mut board = Board::new();
        // Add a single block
        board.set_cell(5, 19, Cell::Filled(Color::RED));
        assert!(!PerfectClearDetector::is_perfect_clear(&board));
    }
    
    #[test]
    fn test_perfect_clear_type_determination() {
        assert_eq!(PerfectClearDetector::determine_perfect_clear_type(1), Some(PerfectClearType::Single));
        assert_eq!(PerfectClearDetector::determine_perfect_clear_type(2), Some(PerfectClearType::Double));
        assert_eq!(PerfectClearDetector::determine_perfect_clear_type(3), Some(PerfectClearType::Triple));
        assert_eq!(PerfectClearDetector::determine_perfect_clear_type(4), Some(PerfectClearType::Tetris));
        assert_eq!(PerfectClearDetector::determine_perfect_clear_type(0), None);
        assert_eq!(PerfectClearDetector::determine_perfect_clear_type(5), None);
    }
    
    #[test]
    fn test_check_perfect_clear() {
        let empty_board = Board::new();
        
        // Perfect clear with 4 lines should return Tetris
        assert_eq!(
            PerfectClearDetector::check_perfect_clear(&empty_board, 4),
            Some(PerfectClearType::Tetris)
        );
        
        // No lines cleared should return None even if board is empty
        assert_eq!(
            PerfectClearDetector::check_perfect_clear(&empty_board, 0),
            None
        );
        
        // Board with blocks should return None
        let mut filled_board = Board::new();
        filled_board.set_cell(0, 23, Cell::Filled(Color::BLUE));
        assert_eq!(
            PerfectClearDetector::check_perfect_clear(&filled_board, 1),
            None
        );
    }
}