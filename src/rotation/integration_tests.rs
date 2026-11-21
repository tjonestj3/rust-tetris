//! Integration tests for the SRS rotation system
//!
//! These tests verify that the SRS system works correctly with the game board
//! and tetromino pieces, including wall kicks and T-spin detection.

#[cfg(test)]
mod tests {
    use super::super::srs::*;
    use crate::board::{Board, Cell};
    use crate::tetromino::{Tetromino, TetrominoType};
    use macroquad::prelude::GRAY;

    /// Helper function to create a board with walls on the sides
    fn create_confined_board() -> Board {
        let mut board = Board::new();
        
        // Add walls on left and right sides for testing wall kicks
        for y in 15..24 {  // Bottom portion of visible area
            board.set_cell(0, y, Cell::Filled(GRAY));
            board.set_cell(9, y, Cell::Filled(GRAY));
        }
        
        board
    }

    /// Helper function to create a board with floor
    fn create_board_with_floor() -> Board {
        let mut board = Board::new();
        
        // Add floor at the bottom
        for x in 0..10 {
            board.set_cell(x, 23, Cell::Filled(GRAY));
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
        piece.position = (7, 18);
        piece.rotation = 1; // Vertical (R state: bar one column right of centre)
        piece.update_blocks();
        assert!(
            piece.absolute_blocks().iter().all(|&(x, _)| x == 8),
            "setup: the bar should sit in column 8, against the wall at column 9"
        );

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
    fn test_vertical_i_rotates_off_right_edge() {
        let srs = SRSRotationSystem::new();
        let board = Board::new();

        // A vertical I (R state) in the rightmost column of an open board
        let mut piece = Tetromino::new(TetrominoType::I);
        piece.position = (8, 10);
        piece.rotation = 1;
        piece.update_blocks();
        assert!(piece.absolute_blocks().iter().all(|&(x, _)| x == 9));

        for (name, result) in [
            ("clockwise", srs.rotate_clockwise(&piece, &board)),
            ("counterclockwise", srs.rotate_counterclockwise(&piece, &board)),
        ] {
            match result {
                RotationResult::Success { new_piece }
                | RotationResult::SuccessWithKick { new_piece, .. } => {
                    assert!(new_piece.absolute_blocks().iter().all(|&(x, _)| (0..10).contains(&x)));
                }
                RotationResult::Failed => panic!("vertical I at the right edge should rotate {}", name),
            }
        }
    }

    /// Under SRS every piece can turn either way from any rotation state
    /// while pressed flat against either side of an open board.
    #[test]
    fn test_every_piece_rotates_off_both_walls_in_every_state() {
        let srs = SRSRotationSystem::new();
        let board = Board::new();
        let mut stuck = Vec::new();

        for piece_type in TetrominoType::all() {
            for rotation in 0..4u8 {
                for wall in ["left", "right"] {
                    let mut piece = Tetromino::new(piece_type);
                    piece.rotation = rotation;
                    piece.update_blocks();
                    piece.position = (0, 10);
                    let xs: Vec<i32> = piece.absolute_blocks().iter().map(|b| b.0).collect();
                    piece.position.0 = if wall == "left" {
                        -xs.iter().min().unwrap()
                    } else {
                        9 - xs.iter().max().unwrap()
                    };

                    for (direction, result) in [
                        ("clockwise", srs.rotate_clockwise(&piece, &board)),
                        ("counterclockwise", srs.rotate_counterclockwise(&piece, &board)),
                    ] {
                        if result == RotationResult::Failed {
                            stuck.push(format!("{:?} rotation {} at the {} wall can't turn {}",
                                piece_type, rotation, wall, direction));
                        }
                    }
                }
            }
        }
        assert!(stuck.is_empty(), "pieces stuck against a wall:\n{}", stuck.join("\n"));
    }

    /// Rotate `piece` on a board that is solid except for the piece's own
    /// cells and the cells of `piece_type` at `expected` in the target state,
    /// so the only place the rotation can land is `expected`.
    fn rotate_into_pocket(piece: &Tetromino, clockwise: bool, expected: (i32, i32)) -> RotationResult {
        let srs = SRSRotationSystem::new();
        let mut target = piece.clone();
        target.rotation = if clockwise { (piece.rotation + 1) % 4 } else { (piece.rotation + 3) % 4 };
        target.update_blocks();
        target.position = expected;

        let open: Vec<(i32, i32)> = piece.absolute_blocks().into_iter()
            .chain(target.absolute_blocks())
            .collect();
        let mut board = Board::new();
        for x in 0..10 {
            for y in 0..24 {
                if !open.contains(&(x, y)) {
                    board.set_cell(x, y, Cell::Filled(GRAY));
                }
            }
        }
        if clockwise { srs.rotate_clockwise(piece, &board) } else { srs.rotate_counterclockwise(piece, &board) }
    }

    fn assert_lands_at(result: RotationResult, expected: (i32, i32)) {
        match result {
            RotationResult::Success { new_piece } | RotationResult::SuccessWithKick { new_piece, .. } => {
                assert_eq!(new_piece.position, expected, "rotation landed in the wrong place");
            }
            RotationResult::Failed => panic!("rotation should kick to {:?} but failed", expected),
        }
    }

    // The SRS tables are written with +y pointing up; on this board +y
    // points down, so an SRS kick of (dx, dy) moves the piece by (dx, -dy).

    #[test]
    fn test_kick_up_moves_piece_up() {
        // T 0→R, third test is SRS (-1, +1): one left, one UP
        let mut piece = Tetromino::new(TetrominoType::T);
        piece.position = (5, 10);
        assert_lands_at(rotate_into_pocket(&piece, true, (4, 9)), (4, 9));
    }

    #[test]
    fn test_kick_down_moves_piece_down() {
        // T R→0, third test is SRS (+1, -1): one right, one DOWN
        let mut piece = Tetromino::new(TetrominoType::T);
        piece.rotation = 1;
        piece.update_blocks();
        piece.position = (5, 10);
        assert_lands_at(rotate_into_pocket(&piece, false, (6, 11)), (6, 11));
    }

    #[test]
    fn test_i_piece_kick_up_two_moves_piece_up_two() {
        // I 0→R, fifth test is SRS (+1, +2): one right, two UP
        let mut piece = Tetromino::new(TetrominoType::I);
        piece.position = (5, 10);
        assert_lands_at(rotate_into_pocket(&piece, true, (6, 8)), (6, 8));
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
                    board.set_cell(x, y, Cell::Filled(GRAY));
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
        board.set_cell(t_x - 1, t_y - 1, Cell::Filled(GRAY)); // Top-left
        board.set_cell(t_x + 1, t_y - 1, Cell::Filled(GRAY)); // Top-right
        board.set_cell(t_x - 1, t_y + 1, Cell::Filled(GRAY)); // Bottom-left
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
        
        board.set_cell(t_x - 1, t_y - 1, Cell::Filled(GRAY));
        board.set_cell(t_x + 1, t_y - 1, Cell::Filled(GRAY));
        board.set_cell(t_x - 1, t_y + 1, Cell::Filled(GRAY));
        
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