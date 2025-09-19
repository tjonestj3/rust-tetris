//! Integration tests for the SRS rotation system
//!
//! These tests verify that the SRS system works correctly with the game board
//! and tetromino pieces, including wall kicks and T-spin detection.

#[cfg(test)]
mod tests {
    use super::super::srs::*;
    use crate::board::{Board, Cell};
    use crate::tetromino::{Tetromino, TetrominoType};
    use macroquad::prelude::Color;

    /// Helper function to create a board with walls on the sides
    fn create_confined_board() -> Board {
        let mut board = Board::new();
        
        // Add walls on left and right sides for testing wall kicks
        for y in 15..24 {  // Bottom portion of visible area
            board.set_cell(0, y, Cell::Filled(Color::GRAY));
            board.set_cell(9, y, Cell::Filled(Color::GRAY));
        }
        
        board
    }

    /// Helper function to create a board with floor
    fn create_board_with_floor() -> Board {
        let mut board = Board::new();
        
        // Add floor at the bottom
        for x in 0..10 {
            board.set_cell(x, 23, Cell::Filled(Color::GRAY));
        }
        
        board
    }

    #[test]
    fn test_basic_rotation_in_open_space() {
        let srs = SRSRotationSystem::new();
        let board = Board::new();
        
        // Test T-piece rotation in open space
        let mut piece = Tetromino::new(TetrominoType::T);
        piece.position = (5, 10); // Center of board
        
        let result = srs.rotate_clockwise(&piece, &board);
        assert!(matches!(result, RotationResult::Success { .. }));
        
        if let RotationResult::Success { new_piece } = result {
            assert_eq!(new_piece.rotation, 1);
            assert_eq!(new_piece.position, (5, 10)); // Position unchanged
        }
    }

    #[test]
    fn test_wall_kick_against_left_wall() {
        let srs = SRSRotationSystem::new();
        let board = create_confined_board();
        
        // Position I-piece against left wall
        let mut piece = Tetromino::new(TetrominoType::I);
        piece.position = (1, 18); // Near left wall
        piece.rotation = 1; // Vertical
        
        // Try rotating to horizontal - should kick right
        let result = srs.rotate_clockwise(&piece, &board);
        
        match result {
            RotationResult::Success { new_piece } => {
                // Basic rotation worked without kick
                assert_eq!(new_piece.rotation, 2);
            },
            RotationResult::SuccessWithKick { new_piece, kick_used } => {
                // Wall kick was used
                assert_eq!(new_piece.rotation, 2);
                println!("Kick used: {:?}", kick_used);
            },
            RotationResult::Failed => {
                panic!("Rotation should succeed with wall kick");
            }
        }
    }

    #[test]
    fn test_wall_kick_against_right_wall() {
        let srs = SRSRotationSystem::new();
        let board = create_confined_board();
        
        // Position I-piece against right wall
        let mut piece = Tetromino::new(TetrominoType::I);
        piece.position = (8, 18); // Near right wall
        piece.rotation = 1; // Vertical
        
        // Try rotating to horizontal - should kick left
        let result = srs.rotate_clockwise(&piece, &board);
        
        match result {
            RotationResult::Success { new_piece } => {
                // Basic rotation worked without kick
                assert_eq!(new_piece.rotation, 2);
            },
            RotationResult::SuccessWithKick { new_piece, kick_used } => {
                // Wall kick was used
                assert_eq!(new_piece.rotation, 2);
                println!("Kick used: {:?}", kick_used);
            },
            RotationResult::Failed => {
                panic!("Rotation should succeed with wall kick");
            }
        }
    }

    #[test]
    fn test_counterclockwise_rotation() {
        let srs = SRSRotationSystem::new();
        let board = Board::new();
        
        let mut piece = Tetromino::new(TetrominoType::T);
        piece.position = (5, 10);
        piece.rotation = 1; // Start at 90 degrees
        
        let result = srs.rotate_counterclockwise(&piece, &board);
        assert!(matches!(result, RotationResult::Success { .. }));
        
        if let RotationResult::Success { new_piece } = result {
            assert_eq!(new_piece.rotation, 0); // Back to 0 degrees
        }
    }

    #[test]
    fn test_rotation_state_wrapping() {
        let srs = SRSRotationSystem::new();
        let board = Board::new();
        
        let mut piece = Tetromino::new(TetrominoType::J);
        piece.position = (5, 10);
        piece.rotation = 3; // Start at 270 degrees
        
        // Rotate clockwise should wrap to 0
        let result = srs.rotate_clockwise(&piece, &board);
        if let RotationResult::Success { new_piece } = result {
            assert_eq!(new_piece.rotation, 0);
        }
        
        // Rotate counterclockwise from 0 should wrap to 3
        piece.rotation = 0;
        let result = srs.rotate_counterclockwise(&piece, &board);
        if let RotationResult::Success { new_piece } = result {
            assert_eq!(new_piece.rotation, 3);
        }
    }

    #[test]
    fn test_o_piece_rotation_invariant() {
        let srs = SRSRotationSystem::new();
        let board = Board::new();
        
        let piece = Tetromino::new(TetrominoType::O);
        let original_rotation = piece.rotation;
        let original_blocks = piece.absolute_blocks();
        
        // O-piece should not change when rotated
        let result = srs.rotate_clockwise(&piece, &board);
        
        match result {
            RotationResult::Success { new_piece } => {
                // Rotation should succeed but piece should be unchanged
                assert_eq!(new_piece.absolute_blocks(), original_blocks);
            },
            _ => panic!("O-piece rotation should always succeed"),
        }
    }

    #[test]
    fn test_impossible_rotation_fails() {
        let srs = SRSRotationSystem::new();
        let mut board = Board::new();
        
        // Fill the entire area around a piece to make rotation impossible
        let center_x = 5;
        let center_y = 10;
        
        for x in (center_x - 2)..(center_x + 3) {
            for y in (center_y - 2)..(center_y + 3) {
                if x != center_x || y != center_y {
                    board.set_cell(x, y, Cell::Filled(Color::GRAY));
                }
            }
        }
        
        let mut piece = Tetromino::new(TetrominoType::T);
        piece.position = (center_x, center_y);
        
        let result = srs.rotate_clockwise(&piece, &board);
        assert_eq!(result, RotationResult::Failed);
    }

    #[test]
    fn test_t_spin_detection() {
        let srs = SRSRotationSystem::new();
        let mut board = Board::new();
        
        // Create a T-spin setup
        let t_x = 5;
        let t_y = 20;
        
        // Fill 3 corners around the T-piece to create T-spin condition
        board.set_cell(t_x - 1, t_y - 1, Cell::Filled(Color::GRAY)); // Top-left
        board.set_cell(t_x + 1, t_y - 1, Cell::Filled(Color::GRAY)); // Top-right
        board.set_cell(t_x - 1, t_y + 1, Cell::Filled(Color::GRAY)); // Bottom-left
        // Leave bottom-right corner open
        
        let mut piece = Tetromino::new(TetrominoType::T);
        piece.position = (t_x, t_y);
        
        // Test T-spin detection
        let is_t_spin = srs.is_t_spin_position(&piece, &board, None);
        assert!(is_t_spin);
        
        // Non-T-piece should not register as T-spin
        let mut i_piece = Tetromino::new(TetrominoType::I);
        i_piece.position = (t_x, t_y);
        let is_not_t_spin = srs.is_t_spin_position(&i_piece, &board, None);
        assert!(!is_not_t_spin);
    }

    #[test]
    fn test_t_spin_detection_disabled() {
        let srs = SRSRotationSystem::without_t_spin_detection();
        let mut board = Board::new();
        
        // Same setup as above
        let t_x = 5;
        let t_y = 20;
        
        board.set_cell(t_x - 1, t_y - 1, Cell::Filled(Color::GRAY));
        board.set_cell(t_x + 1, t_y - 1, Cell::Filled(Color::GRAY));
        board.set_cell(t_x - 1, t_y + 1, Cell::Filled(Color::GRAY));
        
        let mut piece = Tetromino::new(TetrominoType::T);
        piece.position = (t_x, t_y);
        
        // T-spin detection should be disabled
        let is_t_spin = srs.is_t_spin_position(&piece, &board, None);
        assert!(!is_t_spin);
    }

    #[test]
    fn test_multiple_piece_types_rotation() {
        let srs = SRSRotationSystem::new();
        let board = Board::new();
        
        let piece_types = [
            TetrominoType::I,
            TetrominoType::O,
            TetrominoType::T,
            TetrominoType::S,
            TetrominoType::Z,
            TetrominoType::J,
            TetrominoType::L,
        ];
        
        for piece_type in piece_types {
            let mut piece = Tetromino::new(piece_type);
            piece.position = (5, 10);
            
            // All pieces should be able to rotate in open space
            let result = srs.rotate_clockwise(&piece, &board);
            match result {
                RotationResult::Success { .. } | RotationResult::SuccessWithKick { .. } => {
                    // Success expected
                },
                RotationResult::Failed => {
                    panic!("{:?} should be able to rotate in open space", piece_type);
                }
            }
        }
    }

    #[test]
    fn test_all_rotation_states() {
        let srs = SRSRotationSystem::new();
        let board = Board::new();
        
        let mut piece = Tetromino::new(TetrominoType::T);
        piece.position = (5, 10);
        
        // Test rotating through all 4 states
        for expected_rotation in 1..=4 {
            let result = srs.rotate_clockwise(&piece, &board);
            match result {
                RotationResult::Success { new_piece } | RotationResult::SuccessWithKick { new_piece, .. } => {
                    piece = new_piece;
                    assert_eq!(piece.rotation, expected_rotation % 4);
                },
                RotationResult::Failed => {
                    panic!("Rotation {} failed", expected_rotation);
                }
            }
        }
        
        // Should be back to original rotation
        assert_eq!(piece.rotation, 0);
    }
}