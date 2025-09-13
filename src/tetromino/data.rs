//! Tetromino shape data and definitions

use super::types::TetrominoType;

/// Get the block positions for a tetromino type and rotation
/// Returns relative positions from the piece center
pub fn get_tetromino_blocks(piece_type: TetrominoType, rotation: u8) -> Vec<(i32, i32)> {
    let rotation = rotation % 4; // Ensure rotation is 0-3
    
    match piece_type {
        TetrominoType::I => get_i_piece_blocks(rotation),
        TetrominoType::O => get_o_piece_blocks(rotation),
        TetrominoType::T => get_t_piece_blocks(rotation),
        TetrominoType::S => get_s_piece_blocks(rotation),
        TetrominoType::Z => get_z_piece_blocks(rotation),
        TetrominoType::J => get_j_piece_blocks(rotation),
        TetrominoType::L => get_l_piece_blocks(rotation),
    }
}

/// I-piece (line) - 4 blocks in a line
fn get_i_piece_blocks(rotation: u8) -> Vec<(i32, i32)> {
    match rotation {
        0 | 2 => vec![(-1, 0), (0, 0), (1, 0), (2, 0)], // Horizontal
        1 | 3 => vec![(0, -1), (0, 0), (0, 1), (0, 2)], // Vertical
        _ => vec![],
    }
}

/// O-piece (square) - 2x2 square, no rotation
fn get_o_piece_blocks(_rotation: u8) -> Vec<(i32, i32)> {
    vec![(0, 0), (1, 0), (0, 1), (1, 1)]
}

/// T-piece - T-shaped piece
fn get_t_piece_blocks(rotation: u8) -> Vec<(i32, i32)> {
    match rotation {
        0 => vec![(-1, 0), (0, 0), (1, 0), (0, -1)], // T pointing up
        1 => vec![(0, -1), (0, 0), (0, 1), (1, 0)],  // T pointing right
        2 => vec![(-1, 0), (0, 0), (1, 0), (0, 1)],  // T pointing down
        3 => vec![(0, -1), (0, 0), (0, 1), (-1, 0)], // T pointing left
        _ => vec![],
    }
}

/// S-piece - S-shaped piece
fn get_s_piece_blocks(rotation: u8) -> Vec<(i32, i32)> {
    match rotation {
        0 | 2 => vec![(0, 0), (1, 0), (-1, 1), (0, 1)], // Horizontal S
        1 | 3 => vec![(0, -1), (0, 0), (1, 0), (1, 1)], // Vertical S
        _ => vec![],
    }
}

/// Z-piece - Z-shaped piece
fn get_z_piece_blocks(rotation: u8) -> Vec<(i32, i32)> {
    match rotation {
        0 | 2 => vec![(-1, 0), (0, 0), (0, 1), (1, 1)], // Horizontal Z
        1 | 3 => vec![(0, 0), (0, 1), (-1, 1), (-1, 2)], // Vertical Z
        _ => vec![],
    }
}

/// J-piece - J-shaped piece
fn get_j_piece_blocks(rotation: u8) -> Vec<(i32, i32)> {
    match rotation {
        0 => vec![(-1, -1), (-1, 0), (0, 0), (1, 0)], // J pointing right
        1 => vec![(0, -1), (0, 0), (0, 1), (1, 1)],  // J pointing down
        2 => vec![(-1, 0), (0, 0), (1, 0), (1, 1)],  // J pointing left
        3 => vec![(-1, -1), (0, -1), (0, 0), (0, 1)], // J pointing up
        _ => vec![],
    }
}

/// L-piece - L-shaped piece
fn get_l_piece_blocks(rotation: u8) -> Vec<(i32, i32)> {
    match rotation {
        0 => vec![(1, -1), (-1, 0), (0, 0), (1, 0)], // L pointing right
        1 => vec![(0, -1), (0, 0), (0, 1), (1, -1)], // L pointing down
        2 => vec![(-1, 0), (0, 0), (1, 0), (-1, 1)], // L pointing left
        3 => vec![(0, -1), (0, 0), (0, 1), (-1, 1)], // L pointing up
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_pieces_have_four_blocks() {
        for piece_type in TetrominoType::all() {
            for rotation in 0..4 {
                let blocks = get_tetromino_blocks(piece_type, rotation);
                assert_eq!(blocks.len(), 4, 
                    "Piece {:?} rotation {} should have 4 blocks, got {}", 
                    piece_type, rotation, blocks.len());
            }
        }
    }

    #[test]
    fn test_o_piece_same_all_rotations() {
        let blocks_0 = get_tetromino_blocks(TetrominoType::O, 0);
        for rotation in 1..4 {
            let blocks = get_tetromino_blocks(TetrominoType::O, rotation);
            assert_eq!(blocks_0, blocks, 
                "O-piece should be same for all rotations");
        }
    }

    #[test]
    fn test_i_piece_rotations() {
        let horizontal = get_tetromino_blocks(TetrominoType::I, 0);
        let vertical = get_tetromino_blocks(TetrominoType::I, 1);
        
        // Should be different
        assert_ne!(horizontal, vertical);
        
        // Rotation 2 should match rotation 0
        assert_eq!(horizontal, get_tetromino_blocks(TetrominoType::I, 2));
        
        // Rotation 3 should match rotation 1
        assert_eq!(vertical, get_tetromino_blocks(TetrominoType::I, 3));
    }

    #[test]
    fn test_rotation_bounds() {
        // Test that rotation values > 3 are handled correctly
        let blocks_0 = get_tetromino_blocks(TetrominoType::T, 0);
        let blocks_4 = get_tetromino_blocks(TetrominoType::T, 4);
        let blocks_8 = get_tetromino_blocks(TetrominoType::T, 8);
        
        assert_eq!(blocks_0, blocks_4);
        assert_eq!(blocks_0, blocks_8);
    }
}
