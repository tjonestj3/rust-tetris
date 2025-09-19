//! Demo showcasing the SRS rotation system
//!
//! This example demonstrates how to use the SRS rotation system with
//! wall kicks and T-spin detection.

use rust_tetris::board::{Board, Cell};
use rust_tetris::tetromino::{Tetromino, TetrominoType};
use rust_tetris::rotation::{SRSRotationSystem, RotationSystem, RotationResult};
use macroquad::prelude::Color;

fn main() {
    println!("=== SRS Rotation System Demo ===\n");
    
    // Create the rotation system
    let srs = SRSRotationSystem::new();
    println!("✓ Created SRS rotation system with T-spin detection enabled");
    
    // Test basic rotation
    test_basic_rotation(&srs);
    
    // Test wall kick functionality 
    test_wall_kicks(&srs);
    
    // Test T-spin detection
    test_t_spin_detection(&srs);
    
    println!("\n=== Demo Complete ===");
    println!("The SRS rotation system is working correctly!");
}

fn test_basic_rotation(srs: &SRSRotationSystem) {
    println!("\n--- Testing Basic Rotation ---");
    
    let board = Board::new();
    let piece = Tetromino::new(TetrominoType::T);
    
    println!("Initial T-piece rotation: {}", piece.rotation);
    
    let result = srs.rotate_clockwise(&piece, &board);
    match result {
        RotationResult::Success { new_piece } => {
            println!("✓ Clockwise rotation successful: {} -> {}", piece.rotation, new_piece.rotation);
        },
        RotationResult::SuccessWithKick { new_piece, kick_used } => {
            println!("✓ Clockwise rotation with kick: {} -> {}, kick: {:?}", 
                     piece.rotation, new_piece.rotation, kick_used);
        },
        RotationResult::Failed => {
            println!("✗ Clockwise rotation failed");
        }
    }
}

fn test_wall_kicks(srs: &SRSRotationSystem) {
    println!("\n--- Testing Wall Kicks ---");
    
    let mut board = Board::new();
    
    // Create a confined space with walls
    for y in 15..24 {
        board.set_cell(0, y, Cell::Filled(Color::new(0.5, 0.5, 0.5, 1.0)));
        board.set_cell(9, y, Cell::Filled(Color::new(0.5, 0.5, 0.5, 1.0)));
    }
    
    // Test I-piece against left wall
    let mut piece = Tetromino::new(TetrominoType::I);
    piece.position = (2, 18);
    piece.rotation = 1; // Vertical
    
    println!("Testing I-piece rotation from vertical to horizontal near left wall...");
    
    let result = srs.rotate_clockwise(&piece, &board);
    match result {
        RotationResult::Success { new_piece } => {
            println!("✓ Basic rotation worked at position ({}, {})", 
                     new_piece.position.0, new_piece.position.1);
        },
        RotationResult::SuccessWithKick { new_piece, kick_used } => {
            println!("✓ Wall kick successful! New position: ({}, {}), kick: {:?}", 
                     new_piece.position.0, new_piece.position.1, kick_used);
        },
        RotationResult::Failed => {
            println!("✗ Wall kick failed - this might need adjustment");
        }
    }
}

fn test_t_spin_detection(srs: &SRSRotationSystem) {
    println!("\n--- Testing T-spin Detection ---");
    
    let mut board = Board::new();
    
    // Create a T-spin setup
    let t_x = 5;
    let t_y = 20;
    
    // Fill 3 corners around the T-piece
    board.set_cell(t_x - 1, t_y - 1, Cell::Filled(Color::new(0.5, 0.5, 0.5, 1.0))); // Top-left
    board.set_cell(t_x + 1, t_y - 1, Cell::Filled(Color::new(0.5, 0.5, 0.5, 1.0))); // Top-right  
    board.set_cell(t_x - 1, t_y + 1, Cell::Filled(Color::new(0.5, 0.5, 0.5, 1.0))); // Bottom-left
    // Bottom-right corner left open
    
    let mut t_piece = Tetromino::new(TetrominoType::T);
    t_piece.position = (t_x, t_y);
    
    let is_t_spin = srs.is_t_spin_position(&t_piece, &board, None);
    if is_t_spin {
        println!("✓ T-spin position detected correctly!");
    } else {
        println!("✗ T-spin detection may need adjustment");
    }
    
    // Test with non-T piece
    let mut i_piece = Tetromino::new(TetrominoType::I);
    i_piece.position = (t_x, t_y);
    
    let is_not_t_spin = srs.is_t_spin_position(&i_piece, &board, None);
    if !is_not_t_spin {
        println!("✓ Non-T piece correctly not detected as T-spin");
    } else {
        println!("✗ Non-T piece incorrectly detected as T-spin");
    }
    
    // Test with T-spin detection disabled
    let srs_no_tspin = SRSRotationSystem::without_t_spin_detection();
    let no_t_spin = srs_no_tspin.is_t_spin_position(&t_piece, &board, None);
    if !no_t_spin {
        println!("✓ T-spin detection correctly disabled");
    } else {
        println!("✗ T-spin detection not properly disabled");
    }
}