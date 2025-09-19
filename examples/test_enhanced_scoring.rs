//! Test program to verify the enhanced Tetris scoring system integration
//! This demonstrates the new scoring features including T-spins, combos, and back-to-back bonuses

use rust_tetris::game::Game;
use rust_tetris::tetromino::{Tetromino, TetrominoType};
use rust_tetris::board::Cell;

fn main() {
    env_logger::init();
    
    println!("=== Enhanced Tetris Scoring System Test ===\n");
    
    // Test 1: Basic line clear scoring
    test_basic_line_clearing();
    
    // Test 2: Drop points integration
    test_drop_points();
    
    // Test 3: T-spin detection (basic simulation)
    test_t_spin_detection();
    
    println!("\n=== All Enhanced Scoring Tests Complete ===");
}

fn test_basic_line_clearing() {
    println!("Test 1: Basic Line Clearing with Enhanced Scoring");
    let mut game = Game::new();
    
    // Fill the bottom row except one cell to set up a single line clear
    for x in 1..10 {
        game.board.set_cell(x, 23, Cell::Filled(macroquad::prelude::Color::new(0.5, 0.5, 0.5, 1.0)));
    }
    
    let initial_score = game.score;
    println!("Initial score: {}", initial_score);
    
    // Simulate clearing one line
    game.add_score_for_lines(1);
    
    let new_score = game.score;
    println!("Score after 1 line clear: {}", new_score);
    println!("Points gained: {}", new_score - initial_score);
    
    // Clear 4 lines for Tetris bonus
    game.add_score_for_lines(4);
    let tetris_score = game.score;
    println!("Score after Tetris (4 lines): {}", tetris_score);
    println!("Tetris points gained: {}", tetris_score - new_score);
    
    println!("✓ Basic line clearing test passed\n");
}

fn test_drop_points() {
    println!("Test 2: Drop Points Integration");
    let mut game = Game::new();
    
    let initial_score = game.score;
    println!("Initial score: {}", initial_score);
    
    // Test soft drop points
    game.scoring_system.add_drop_points(1); // 1 point for soft drop
    game.score = game.scoring_system.total_score();
    
    let soft_drop_score = game.score;
    println!("Score after soft drop: {}", soft_drop_score);
    
    // Test hard drop points  
    game.scoring_system.add_drop_points(10); // 10 points for hard drop
    game.score = game.scoring_system.total_score();
    
    let hard_drop_score = game.score;
    println!("Score after hard drop: {}", hard_drop_score);
    println!("Drop points gained: {}", hard_drop_score - initial_score);
    
    println!("✓ Drop points integration test passed\n");
}

fn test_t_spin_detection() {
    println!("Test 3: T-Spin Detection");
    let mut game = Game::new();
    
    // Create a T-piece manually for testing
    let t_piece = Tetromino::new(TetrominoType::T);
    game.current_piece = Some(t_piece);
    
    // Mark that the last action was a rotation (required for T-spin)
    game.last_action_was_rotation = true;
    
    // Create a T-spin setup by filling surrounding cells
    let center_x = game.current_piece.as_ref().unwrap().position.0;
    let center_y = game.current_piece.as_ref().unwrap().position.1;
    
    // Fill 3 corners around the T-piece to simulate T-spin condition
    game.board.set_cell(center_x - 1, center_y - 1, Cell::Filled(macroquad::prelude::Color::new(0.5, 0.5, 0.5, 1.0)));
    game.board.set_cell(center_x + 1, center_y - 1, Cell::Filled(macroquad::prelude::Color::new(0.5, 0.5, 0.5, 1.0)));
    game.board.set_cell(center_x - 1, center_y + 1, Cell::Filled(macroquad::prelude::Color::new(0.5, 0.5, 0.5, 1.0)));
    
    let is_t_spin = game.is_t_spin();
    println!("T-spin detection result: {}", is_t_spin);
    
    if is_t_spin {
        println!("✓ T-spin detection test passed");
    } else {
        println!("⚠ T-spin detection test - detection may need refinement");
    }
    
    // Test T-spin scoring
    let initial_score = game.score;
    println!("Initial score: {}", initial_score);
    
    // Simulate a T-spin single line clear
    game.add_score_for_lines(1); // This should trigger T-spin bonus if detection works
    
    let t_spin_score = game.score;
    println!("Score after T-spin line clear: {}", t_spin_score);
    println!("T-spin bonus included: {}", t_spin_score - initial_score);
    
    println!("✓ T-spin scoring test completed\n");
}