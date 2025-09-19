//! SRS Wall Kick Tables
//! 
//! Contains the official Super Rotation System wall kick offset tables.
//! These define the positions to try when a basic rotation fails.

use crate::tetromino::TetrominoType;
use serde::{Serialize, Deserialize};

/// Rotation state (0° = 0, 90° CW = 1, 180° = 2, 270° CW = 3)
pub type RotationState = u8;

/// Wall kick offset data - (x_offset, y_offset)
pub type KickOffset = (i32, i32);

/// Wall kick data for a specific rotation transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WallKickData {
    /// From rotation state (0-3)
    pub from_state: RotationState,
    /// To rotation state (0-3)  
    pub to_state: RotationState,
    /// List of kick offsets to try in order
    pub kicks: Vec<KickOffset>,
}

/// Get wall kick offsets for a piece type and rotation transition
pub fn get_wall_kick_offsets(piece_type: TetrominoType, from_state: RotationState, to_state: RotationState) -> Vec<KickOffset> {
    match piece_type {
        TetrominoType::O => {
            // O-piece doesn't rotate in SRS
            vec![]
        },
        TetrominoType::I => {
            get_i_piece_kicks(from_state, to_state)
        },
        _ => {
            // JLSTZ pieces use the same kick table
            get_jlstz_kicks(from_state, to_state)
        }
    }
}

/// Wall kick offsets for JLSTZ pieces (standard SRS table)
fn get_jlstz_kicks(from_state: RotationState, to_state: RotationState) -> Vec<KickOffset> {
    match (from_state, to_state) {
        // 0 -> R (0° to 90° CW)
        (0, 1) => vec![(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
        // R -> 0 (90° CW to 0°)
        (1, 0) => vec![(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        // R -> 2 (90° CW to 180°)
        (1, 2) => vec![(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        // 2 -> R (180° to 90° CW)
        (2, 1) => vec![(0, 0), (-1, 0), (-1, 1), (0, -2), (-1, -2)],
        // 2 -> L (180° to 270° CW)
        (2, 3) => vec![(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
        // L -> 2 (270° CW to 180°)
        (3, 2) => vec![(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
        // L -> 0 (270° CW to 0°)
        (3, 0) => vec![(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
        // 0 -> L (0° to 270° CW)
        (0, 3) => vec![(0, 0), (1, 0), (1, 1), (0, -2), (1, -2)],
        _ => vec![(0, 0)], // Fallback to basic rotation only
    }
}

/// Wall kick offsets for I-piece (special SRS table)
fn get_i_piece_kicks(from_state: RotationState, to_state: RotationState) -> Vec<KickOffset> {
    match (from_state, to_state) {
        // 0 -> R (0° to 90° CW)
        (0, 1) => vec![(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
        // R -> 0 (90° CW to 0°)
        (1, 0) => vec![(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
        // R -> 2 (90° CW to 180°)
        (1, 2) => vec![(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
        // 2 -> R (180° to 90° CW)
        (2, 1) => vec![(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
        // 2 -> L (180° to 270° CW)
        (2, 3) => vec![(0, 0), (2, 0), (-1, 0), (2, 1), (-1, -2)],
        // L -> 2 (270° CW to 180°)
        (3, 2) => vec![(0, 0), (-2, 0), (1, 0), (-2, -1), (1, 2)],
        // L -> 0 (270° CW to 0°)
        (3, 0) => vec![(0, 0), (1, 0), (-2, 0), (1, -2), (-2, 1)],
        // 0 -> L (0° to 270° CW)
        (0, 3) => vec![(0, 0), (-1, 0), (2, 0), (-1, 2), (2, -1)],
        _ => vec![(0, 0)], // Fallback to basic rotation only
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jlstz_kick_tables() {
        // Test that we have kick data for common rotations
        let kicks = get_jlstz_kicks(0, 1); // 0° to 90° CW
        assert_eq!(kicks.len(), 5);
        assert_eq!(kicks[0], (0, 0)); // First kick is always no offset
        
        let kicks = get_jlstz_kicks(1, 0); // 90° CW to 0°
        assert_eq!(kicks.len(), 5);
        assert_eq!(kicks[0], (0, 0));
    }

    #[test]
    fn test_i_piece_kick_tables() {
        // Test I-piece specific kicks
        let kicks = get_i_piece_kicks(0, 1); // 0° to 90° CW
        assert_eq!(kicks.len(), 5);
        assert_eq!(kicks[0], (0, 0));
        assert_eq!(kicks[1], (-2, 0)); // I-piece specific offset
    }

    #[test]
    fn test_o_piece_no_kicks() {
        // O-piece doesn't rotate
        let kicks = get_wall_kick_offsets(TetrominoType::O, 0, 1);
        assert_eq!(kicks.len(), 0);
    }

    #[test]
    fn test_piece_type_routing() {
        // Test that different piece types use correct kick tables
        let t_kicks = get_wall_kick_offsets(TetrominoType::T, 0, 1);
        let i_kicks = get_wall_kick_offsets(TetrominoType::I, 0, 1);
        let o_kicks = get_wall_kick_offsets(TetrominoType::O, 0, 1);
        
        assert_eq!(t_kicks.len(), 5); // JLSTZ table
        assert_eq!(i_kicks.len(), 5); // I-piece table
        assert_eq!(o_kicks.len(), 0); // No rotation
    }
}