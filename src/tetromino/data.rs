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
///
/// These are the four SRS states, which the SRS I kick table assumes. The
/// I-piece turns about the centre of its 4x4 box, not about a cell, so its
/// states are not mirror pairs: R sits one column right of L, and 2 one row
/// below 0. Collapsing them to two shapes (0 == 2, 1 == 3) leaves a vertical
/// I in state R against the right wall unable to rotate at all.
fn get_i_piece_blocks(rotation: u8) -> Vec<(i32, i32)> {
    match rotation {
        0 => vec![(-1, 0), (0, 0), (1, 0), (2, 0)],   // Horizontal (spawn)
        1 => vec![(1, -1), (1, 0), (1, 1), (1, 2)],   // Vertical, right of centre
        2 => vec![(-1, 1), (0, 1), (1, 1), (2, 1)],   // Horizontal, one row lower
        3 => vec![(0, -1), (0, 0), (0, 1), (0, 2)],   // Vertical, left of centre
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
///
/// The four SRS states, turning about the cell at (0, 1). As with the
/// I-piece, the states are not mirror pairs: R sits right of centre, L left
/// of it, and 2 one row below 0. With only two shapes the SRS kicks can't
/// move a vertical S off the left wall.
fn get_s_piece_blocks(rotation: u8) -> Vec<(i32, i32)> {
    match rotation {
        0 => vec![(0, 0), (1, 0), (-1, 1), (0, 1)],   // Horizontal (spawn)
        1 => vec![(0, 0), (0, 1), (1, 1), (1, 2)],    // Vertical, right of centre
        2 => vec![(0, 1), (1, 1), (-1, 2), (0, 2)],   // Horizontal, one row lower
        3 => vec![(-1, 0), (-1, 1), (0, 1), (0, 2)],  // Vertical, left of centre
        _ => vec![],
    }
}

/// Z-piece - Z-shaped piece
///
/// The four SRS states, turning about the cell at (0, 1); see the S-piece.
fn get_z_piece_blocks(rotation: u8) -> Vec<(i32, i32)> {
    match rotation {
        0 => vec![(-1, 0), (0, 0), (0, 1), (1, 1)],   // Horizontal (spawn)
        1 => vec![(1, 0), (1, 1), (0, 1), (0, 2)],    // Vertical, right of centre
        2 => vec![(-1, 1), (0, 1), (0, 2), (1, 2)],   // Horizontal, one row lower
        3 => vec![(0, 0), (0, 1), (-1, 1), (-1, 2)],  // Vertical, left of centre
        _ => vec![],
    }
}

/// J-piece - J-shaped piece
fn get_j_piece_blocks(rotation: u8) -> Vec<(i32, i32)> {
    match rotation {
        0 => vec![(-1, -1), (-1, 0), (0, 0), (1, 0)], // Hook top-left (spawn)
        1 => vec![(0, -1), (1, -1), (0, 0), (0, 1)],  // Hook top-right
        2 => vec![(-1, 0), (0, 0), (1, 0), (1, 1)],   // Hook bottom-right
        3 => vec![(0, -1), (0, 0), (-1, 1), (0, 1)],  // Hook bottom-left
        _ => vec![],
    }
}

/// L-piece - L-shaped piece
fn get_l_piece_blocks(rotation: u8) -> Vec<(i32, i32)> {
    match rotation {
        0 => vec![(1, -1), (-1, 0), (0, 0), (1, 0)],  // Hook top-right (spawn)
        1 => vec![(0, -1), (0, 0), (0, 1), (1, 1)],   // Hook bottom-right
        2 => vec![(-1, 0), (0, 0), (1, 0), (-1, 1)],  // Hook bottom-left
        3 => vec![(-1, -1), (0, -1), (0, 0), (0, 1)], // Hook top-left
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

        let shifted = |blocks: &[(i32, i32)], dx: i32, dy: i32| -> Vec<(i32, i32)> {
            blocks.iter().map(|(x, y)| (x + dx, y + dy)).collect()
        };

        // SRS: rotation 2 is rotation 0 one row lower
        assert_eq!(shifted(&horizontal, 0, 1), get_tetromino_blocks(TetrominoType::I, 2));

        // SRS: rotation 1 (R) is rotation 3 (L) one column to the right
        let vertical_l = get_tetromino_blocks(TetrominoType::I, 3);
        assert_eq!(shifted(&vertical_l, 1, 0), vertical);

        // Every state is a straight line of four
        for rotation in 0..4 {
            let blocks = get_tetromino_blocks(TetrominoType::I, rotation);
            let same_row = blocks.iter().all(|b| b.1 == blocks[0].1);
            let same_col = blocks.iter().all(|b| b.0 == blocks[0].0);
            assert!(same_row != same_col, "I rotation {} should be a straight line", rotation);
        }
    }

    /// Turn a shape 90° clockwise about the origin (y points down).
    fn turned_cw(blocks: &[(i32, i32)]) -> Vec<(i32, i32)> {
        blocks.iter().map(|&(x, y)| (-y, x)).collect()
    }

    fn sorted(mut blocks: Vec<(i32, i32)>) -> Vec<(i32, i32)> {
        blocks.sort();
        blocks
    }

    /// SRS: each rotation state is the previous one turned 90° clockwise
    /// about one fixed centre. A turn about a centre c equals a turn about
    /// the origin plus a fixed shift, so every 0→1→2→3→0 step must need
    /// the same shift. A mirrored shape (J drawn as L) can't be matched by
    /// any shift; a two-shape piece (S/Z with 0 == 2) needs different
    /// shifts on different steps, and so can't use the SRS kick table.
    #[test]
    fn test_rotation_states_turn_about_one_centre() {
        for piece_type in TetrominoType::all() {
            if piece_type == TetrominoType::O {
                continue; // The O-piece doesn't rotate
            }
            let mut shifts = Vec::new();
            for rotation in 0..4u8 {
                let turned = sorted(turned_cw(&get_tetromino_blocks(piece_type, rotation)));
                let next = sorted(get_tetromino_blocks(piece_type, (rotation + 1) % 4));
                let shift = (next[0].0 - turned[0].0, next[0].1 - turned[0].1);
                let moved: Vec<_> = turned.iter().map(|&(x, y)| (x + shift.0, y + shift.1)).collect();
                assert_eq!(moved, next,
                    "{:?} rotation {} turned clockwise is not the shape of rotation {} (mirrored?)",
                    piece_type, rotation, (rotation + 1) % 4);
                shifts.push(shift);
            }
            assert!(shifts.iter().all(|&s| s == shifts[0]),
                "{:?} rotation states don't turn about one centre (shift per step: {:?})",
                piece_type, shifts);
        }
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
