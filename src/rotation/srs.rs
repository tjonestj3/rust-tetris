//! Super Rotation System (SRS) Implementation
//! 
//! This module implements the core SRS rotation logic, including wall kicks
//! and T-spin detection following official Tetris guidelines.

use crate::tetromino::{Tetromino, TetrominoType};
use crate::board::Board;
use super::kick_tables::{get_wall_kick_offsets, KickOffset};
use serde::{Serialize, Deserialize};

/// Rotation state representation (0°, 90° CW, 180°, 270° CW)
pub type RotationState = u8;

/// Result of a rotation attempt
#[derive(Debug, Clone, PartialEq)]
pub enum RotationResult {
    /// Rotation succeeded without kicks
    Success { new_piece: Tetromino },
    /// Rotation succeeded with wall kick
    SuccessWithKick { new_piece: Tetromino, kick_used: KickOffset },
    /// Rotation failed - no valid position found
    Failed,
}

/// Trait for rotation systems - allows for different rotation behaviors
pub trait RotationSystem {
    /// Attempt to rotate a piece clockwise
    fn rotate_clockwise(&self, piece: &Tetromino, board: &Board) -> RotationResult;
    
    /// Attempt to rotate a piece counterclockwise
    fn rotate_counterclockwise(&self, piece: &Tetromino, board: &Board) -> RotationResult;
    
    /// Check if the last rotation could result in a T-spin
    fn is_t_spin_position(&self, piece: &Tetromino, board: &Board, kick_used: Option<KickOffset>) -> bool;
}

/// Super Rotation System implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SRSRotationSystem {
    /// Whether to enable T-spin detection
    pub enable_t_spin_detection: bool,
}

impl Default for SRSRotationSystem {
    fn default() -> Self {
        Self {
            enable_t_spin_detection: true,
        }
    }
}

impl SRSRotationSystem {
    /// Create a new SRS rotation system
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create SRS system with T-spin detection disabled
    pub fn without_t_spin_detection() -> Self {
        Self {
            enable_t_spin_detection: false,
        }
    }
    
    /// Attempt rotation with wall kicks
    fn try_rotation_with_kicks(
        &self,
        piece: &Tetromino,
        board: &Board,
        target_rotation: RotationState,
    ) -> RotationResult {
        let from_state = piece.rotation;
        let kick_offsets = get_wall_kick_offsets(piece.piece_type, from_state, target_rotation);
        
        // If no kicks are available (like O-piece), just try basic rotation
        if kick_offsets.is_empty() {
            return self.try_basic_rotation(piece, board, target_rotation);
        }
        
        // Try each kick offset in order
        for (kick_index, (kick_x, kick_y)) in kick_offsets.iter().enumerate() {
            let mut test_piece = piece.clone();
            
            // Apply rotation
            test_piece.rotation = target_rotation;
            test_piece.update_blocks();
            
            // Apply kick offset
            test_piece.position.0 += kick_x;
            test_piece.position.1 += kick_y;
            
            // Test if the new position is valid
            if self.is_position_valid(&test_piece, board) {
                return if kick_index == 0 {
                    // First kick (0, 0) is basic rotation
                    RotationResult::Success { new_piece: test_piece }
                } else {
                    // Successful wall kick
                    RotationResult::SuccessWithKick { 
                        new_piece: test_piece, 
                        kick_used: (*kick_x, *kick_y) 
                    }
                };
            }
        }
        
        RotationResult::Failed
    }
    
    /// Try basic rotation without kicks
    fn try_basic_rotation(
        &self,
        piece: &Tetromino,
        board: &Board,
        target_rotation: RotationState,
    ) -> RotationResult {
        let mut test_piece = piece.clone();
        test_piece.rotation = target_rotation;
        test_piece.update_blocks();
        
        if self.is_position_valid(&test_piece, board) {
            RotationResult::Success { new_piece: test_piece }
        } else {
            RotationResult::Failed
        }
    }
    
    /// Check if a piece position is valid on the board
    fn is_position_valid(&self, piece: &Tetromino, board: &Board) -> bool {
        for (x, y) in piece.absolute_blocks() {
            if !board.is_position_valid(x, y) {
                return false;
            }
        }
        true
    }
    
    /// Get the next rotation state clockwise
    fn next_rotation_cw(current: RotationState) -> RotationState {
        (current + 1) % 4
    }
    
    /// Get the next rotation state counterclockwise  
    fn next_rotation_ccw(current: RotationState) -> RotationState {
        if current == 0 { 3 } else { current - 1 }
    }
}

impl RotationSystem for SRSRotationSystem {
    fn rotate_clockwise(&self, piece: &Tetromino, board: &Board) -> RotationResult {
        let target_rotation = Self::next_rotation_cw(piece.rotation);
        self.try_rotation_with_kicks(piece, board, target_rotation)
    }
    
    fn rotate_counterclockwise(&self, piece: &Tetromino, board: &Board) -> RotationResult {
        let target_rotation = Self::next_rotation_ccw(piece.rotation);
        self.try_rotation_with_kicks(piece, board, target_rotation)
    }
    
    fn is_t_spin_position(&self, piece: &Tetromino, board: &Board, _kick_used: Option<KickOffset>) -> bool {
        if !self.enable_t_spin_detection {
            return false;
        }
        
        // Only T-pieces can T-spin
        if piece.piece_type != TetrominoType::T {
            return false;
        }
        
        // Check the "3-corner rule" - T-piece must have 3 of its 4 corners occupied
        let center_x = piece.position.0;
        let center_y = piece.position.1;
        
        let corners = [
            (center_x - 1, center_y - 1), // Top-left
            (center_x + 1, center_y - 1), // Top-right
            (center_x - 1, center_y + 1), // Bottom-left
            (center_x + 1, center_y + 1), // Bottom-right
        ];
        
        let occupied_corners = corners.iter()
            .filter(|(x, y)| {
                // Position is occupied if it's out of bounds or filled
                !board.is_position_valid(*x, *y) || 
                board.get_cell(*x, *y).map_or(true, |cell| cell.is_filled())
            })
            .count();
        
        // T-spin requires at least 3 corners to be occupied
        // For "proper" T-spins, we could add additional checks here
        occupied_corners >= 3
    }
}

/// Helper function to create the default SRS rotation system
pub fn create_srs_system() -> Box<dyn RotationSystem> {
    Box::new(SRSRotationSystem::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::Board;
    use crate::tetromino::Tetromino;
    
    fn create_test_board() -> Board {
        Board::new()
    }
    
    #[test]
    fn test_basic_rotation_success() {
        let srs = SRSRotationSystem::new();
        let board = create_test_board();
        let piece = Tetromino::new(TetrominoType::T);
        
        let result = srs.rotate_clockwise(&piece, &board);
        assert!(matches!(result, RotationResult::Success { .. }));
    }
    
    #[test]
    fn test_o_piece_no_rotation() {
        let srs = SRSRotationSystem::new();
        let board = create_test_board();
        let piece = Tetromino::new(TetrominoType::O);
        
        // O-piece should succeed (no change) since it doesn't rotate
        let result = srs.rotate_clockwise(&piece, &board);
        match result {
            RotationResult::Success { new_piece } => {
                assert_eq!(new_piece.rotation, piece.rotation);
            },
            _ => panic!("O-piece rotation should succeed with no change"),
        }
    }
    
    #[test]
    fn test_rotation_state_transitions() {
        assert_eq!(SRSRotationSystem::next_rotation_cw(0), 1);
        assert_eq!(SRSRotationSystem::next_rotation_cw(1), 2);
        assert_eq!(SRSRotationSystem::next_rotation_cw(2), 3);
        assert_eq!(SRSRotationSystem::next_rotation_cw(3), 0);
        
        assert_eq!(SRSRotationSystem::next_rotation_ccw(0), 3);
        assert_eq!(SRSRotationSystem::next_rotation_ccw(1), 0);
        assert_eq!(SRSRotationSystem::next_rotation_ccw(2), 1);
        assert_eq!(SRSRotationSystem::next_rotation_ccw(3), 2);
    }
    
    #[test]
    fn test_t_spin_detection_enabled() {
        let srs = SRSRotationSystem::new();
        assert!(srs.enable_t_spin_detection);
        
        let srs_no_tspin = SRSRotationSystem::without_t_spin_detection();
        assert!(!srs_no_tspin.enable_t_spin_detection);
    }
}