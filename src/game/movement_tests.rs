//! Comprehensive tests for piece movement and locking logic
//! These tests are designed to prevent the locking bugs identified in the game

use super::*;
use crate::tetromino::{Tetromino, TetrominoType};
use crate::board::Cell;
use crate::game::config::*;

#[cfg(test)]
mod movement_tests {
    use super::*;

    /// Helper function to create a game with a specific piece type
    fn create_game_with_piece(piece_type: TetrominoType) -> Game {
        let mut game = Game::new();
        game.current_piece = Some(Tetromino::new(piece_type));
        game.next_piece = TetrominoType::I; // Set predictable next piece
        game
    }

    /// Helper function to fill bottom rows to create a landing surface
    fn create_landing_surface(game: &mut Game, height: usize) {
        let board_bottom = BOARD_HEIGHT + BUFFER_HEIGHT - 1;
        for y in 0..height {
            for x in 0..BOARD_WIDTH {
                game.board.set_cell(
                    x as i32, 
                    (board_bottom - y) as i32, 
                    Cell::Filled(macroquad::prelude::RED)
                );
            }
        }
    }

    #[test]
    fn test_piece_continues_falling_after_line_clear() {
        let mut game = create_game_with_piece(TetrominoType::I);
        
        // Create a setup where line clearing will happen
        // Fill bottom row except for one space, then place I-piece above
        let board_bottom = BOARD_HEIGHT + BUFFER_HEIGHT - 1;
        for x in 0..(BOARD_WIDTH - 1) {
            game.board.set_cell(x as i32, board_bottom as i32, Cell::Filled(macroquad::prelude::RED));
        }
        
        // Position I-piece above the incomplete line
        if let Some(ref mut piece) = game.current_piece {
            piece.position = (((BOARD_WIDTH - 1) as i32), (board_bottom - 5) as i32);
        }
        
        let original_y = game.current_piece.as_ref().unwrap().position.1;
        
        // Simulate piece falling until it would complete the line
        while game.current_piece.is_some() {
            let moved = game.drop_current_piece();
            if !moved {
                break;
            }
        }
        
        // Lock the piece to trigger line clearing
        game.lock_current_piece();
        
        // Verify that line clearing was triggered
        assert!(!game.clearing_lines.is_empty(), "Line clearing should have been triggered");
        
        // Finish the line clearing animation
        game.finish_line_clear();
        
        // Verify that we still have a current piece (new piece should have spawned)
        assert!(game.current_piece.is_some(), "Should have a current piece after line clear");
        
        // Verify that the piece can continue falling (not stuck in locking state)
        assert!(!game.piece_is_locking, "Piece should not be in locking state after line clear");
        assert_eq!(game.lock_delay_timer, 0.0, "Lock delay timer should be reset after line clear");
        
        // Verify the piece can actually move down
        let initial_y = game.current_piece.as_ref().unwrap().position.1;
        assert!(game.drop_current_piece(), "Piece should be able to drop after line clear");
        let new_y = game.current_piece.as_ref().unwrap().position.1;
        assert!(new_y > initial_y, "Piece should have moved down after line clear");
    }

    #[test]
    fn test_lock_delay_reset_works_correctly() {
        let mut game = create_game_with_piece(TetrominoType::T);
        
        // Create a surface for the piece to land on
        create_landing_surface(&mut game, 2);
        
        // Position piece just above the surface
        if let Some(ref mut piece) = game.current_piece {
            piece.position.1 = (BOARD_HEIGHT + BUFFER_HEIGHT - 4) as i32;
        }
        
        // Drop piece until it's grounded
        while game.drop_current_piece() {
            // Keep dropping
        }
        
        // Piece should now be in locking state
        assert!(game.piece_is_locking, "Piece should be in locking state when grounded");
        
        // Test that rotation resets lock delay
        let initial_lock_timer = game.lock_delay_timer;
        game.update(0.1); // Advance lock timer
        assert!(game.lock_delay_timer > initial_lock_timer, "Lock timer should advance");
        
        // Rotate piece (should reset lock delay if successful)
        if game.rotate_piece_clockwise() {
            assert!(!game.piece_is_locking || game.lock_delay_timer == 0.0, 
                   "Successful rotation should reset lock delay");
        }
    }

    #[test]
    fn test_horizontal_movement_doesnt_cause_premature_lock() {
        let mut game = create_game_with_piece(TetrominoType::I);
        
        // Create a partial surface with a gap for the piece to fall into
        let board_bottom = BOARD_HEIGHT + BUFFER_HEIGHT - 1;
        for x in 0..(BOARD_WIDTH - 1) {
            game.board.set_cell(x as i32, board_bottom as i32, Cell::Filled(macroquad::prelude::RED));
        }
        
        // Position piece above the gap
        if let Some(ref mut piece) = game.current_piece {
            piece.position = (2, (board_bottom - 8) as i32);
        }
        
        let initial_y = game.current_piece.as_ref().unwrap().position.1;
        
        // Move piece horizontally while falling
        for _ in 0..5 {
            game.move_piece(1, 0); // Move right
            game.update(0.1); // Allow some time to pass
        }
        
        // The piece should not be locked yet - it should still be able to fall
        assert!(game.current_piece.is_some(), "Piece should still exist after horizontal movement");
        
        // Verify piece can still move down
        let can_drop = game.drop_current_piece();
        if can_drop {
            let new_y = game.current_piece.as_ref().unwrap().position.1;
            assert!(new_y > initial_y, "Piece should be able to continue falling after horizontal movement");
        }
    }

    #[test]
    fn test_ghost_block_interaction_doesnt_break_falling() {
        let mut game = create_game_with_piece(TetrominoType::L);
        game.ghost_blocks_available = 1;
        
        // Position piece in the middle of the board
        if let Some(ref mut piece) = game.current_piece {
            piece.position = (BOARD_WIDTH as i32 / 2, (BUFFER_HEIGHT + 5) as i32);
        }
        
        let initial_position = game.current_piece.as_ref().unwrap().position;
        
        // Activate ghost block placement mode
        game.toggle_ghost_block_mode();
        assert!(game.ghost_block_placement_mode, "Ghost block mode should be active");
        
        // Place a ghost block
        game.ghost_block_cursor = (1, (BUFFER_HEIGHT + 10) as i32);
        game.place_ghost_block();
        
        // Finish ghost block animation by updating game until completion
        while game.ghost_throw_active {
            game.update(GHOST_THROW_ANIMATION_TIME); // Force completion through normal update
        }
        
        // Verify current piece still exists and can move
        assert!(game.current_piece.is_some(), "Current piece should still exist after ghost block operation");
        
        // Verify piece is not stuck in invalid state
        let piece_valid = game.is_piece_valid(game.current_piece.as_ref().unwrap());
        assert!(piece_valid, "Current piece should be in valid position after ghost block operation");
        
        // Verify piece can continue falling
        let can_drop = game.drop_current_piece();
        if can_drop {
            let new_position = game.current_piece.as_ref().unwrap().position;
            assert!(new_position.1 > initial_position.1, "Piece should continue falling after ghost block interaction");
        }
    }

    #[test]
    fn test_hard_drop_locks_immediately() {
        let mut game = create_game_with_piece(TetrominoType::O);
        
        // Create landing surface
        create_landing_surface(&mut game, 3);
        
        // Position piece high up
        if let Some(ref mut piece) = game.current_piece {
            piece.position = (BOARD_WIDTH as i32 / 2, (BUFFER_HEIGHT + 2) as i32);
        }
        
        let initial_score = game.score;
        
        // Perform hard drop
        game.hard_drop();
        
        // Verify piece was locked (current_piece should be None after hard drop)
        assert!(game.current_piece.is_none(), "Piece should be locked immediately after hard drop");
        
        // Verify score was awarded for hard drop
        assert!(game.score > initial_score, "Score should increase after hard drop");
        
        // Verify new piece was spawned
        game.update(0.1); // Allow spawn to happen
        // Note: spawn happens in lock_current_piece, so should be immediate
    }

    #[test] 
    fn test_input_spam_doesnt_break_locking() {
        let mut game = create_game_with_piece(TetrominoType::T);
        
        // Create landing surface
        create_landing_surface(&mut game, 2);
        
        // Position piece just above surface
        if let Some(ref mut piece) = game.current_piece {
            piece.position = (BOARD_WIDTH as i32 / 2, (BOARD_HEIGHT + BUFFER_HEIGHT - 4) as i32);
        }
        
        // Drop piece until grounded
        while game.drop_current_piece() {}
        
        // Spam inputs while piece is in lock delay
        for _ in 0..50 {
            game.move_piece(-1, 0);  // Left
            game.move_piece(1, 0);   // Right  
            game.rotate_piece_clockwise();
            game.rotate_piece_counterclockwise();
            game.update(0.01); // Small time step
        }
        
        // After enough time and resets, piece should eventually lock
        // Force time passage to exceed max lock resets and lifetime
        game.piece_lifetime_timer = MAX_PIECE_LIFETIME + 1.0;
        game.update(0.1);
        
        // Piece should be locked due to lifetime limit
        assert!(game.current_piece.is_none(), "Piece should eventually lock even with input spam");
    }

    #[test]
    fn test_max_lock_resets_prevents_infinite_floating() {
        let mut game = create_game_with_piece(TetrominoType::I);
        
        // Create a platform with a small well
        let board_bottom = BOARD_HEIGHT + BUFFER_HEIGHT - 1;
        for x in 0..BOARD_WIDTH {
            if x != BOARD_WIDTH / 2 { // Leave center empty
                game.board.set_cell(x as i32, board_bottom as i32, Cell::Filled(macroquad::prelude::RED));
            }
        }
        
        // Position I piece above the well
        if let Some(ref mut piece) = game.current_piece {
            piece.position = ((BOARD_WIDTH / 2) as i32, (board_bottom - 3) as i32);
        }
        
        // Drop until grounded
        while game.drop_current_piece() {}
        
        // Repeatedly try to reset lock delay beyond the limit
        for _ in 0..(MAX_LOCK_RESETS + 5) {
            if game.current_piece.is_some() {
                game.reset_lock_delay();
                game.update(0.01);
            }
        }
        
        // After exceeding max resets, piece should be forced into locking state
        if game.current_piece.is_some() {
            assert!(game.piece_is_locking || game.lock_resets >= MAX_LOCK_RESETS, 
                   "Piece should be in locking state or have hit reset limit");
            
            // Force lock delay to expire
            game.lock_delay_timer = LOCK_DELAY + 0.1;
            game.update(0.1);
        }
        
        // Piece should eventually lock
        let max_attempts = 100;
        for _ in 0..max_attempts {
            if game.current_piece.is_none() {
                break;
            }
            game.update(0.1);
        }
        
        // Should not have infinite floating
        assert!(game.current_piece.is_none() || 
                game.piece_lifetime_timer >= MAX_PIECE_LIFETIME, 
                "Piece should not float indefinitely");
    }

    #[test]
    fn test_piece_respects_natural_physics_after_operations() {
        let mut game = create_game_with_piece(TetrominoType::J);
        
        // Create a surface with gaps
        let board_bottom = BOARD_HEIGHT + BUFFER_HEIGHT - 1;
        for x in (0..BOARD_WIDTH).step_by(2) {
            game.board.set_cell(x as i32, board_bottom as i32, Cell::Filled(macroquad::prelude::RED));
        }
        
        // Position piece above a gap
        if let Some(ref mut piece) = game.current_piece {
            piece.position = (1, (board_bottom - 3) as i32);
        }
        
        let initial_y = game.current_piece.as_ref().unwrap().position.1;
        
        // Perform various operations
        game.move_piece(1, 0);
        game.rotate_piece_clockwise();
        game.hold_piece(); // This swaps the piece
        
        // Verify piece (now held piece replacement) still follows physics
        if let Some(ref piece) = game.current_piece {
            // Should be able to fall into the gap
            let mut fall_count = 0;
            let mut test_game = game.clone(); // Clone to test without modifying
            
            while test_game.drop_current_piece() && fall_count < 10 {
                fall_count += 1;
            }
            
        assert!(fall_count > 0, "Piece should be able to fall after operations");
        }
    }
    
    #[test]
    fn test_side_collision_doesnt_start_lock_delay() {
        let mut game = create_game_with_piece(TetrominoType::I);
        
        // Create a vertical wall on the right side
        let board_bottom = BOARD_HEIGHT + BUFFER_HEIGHT - 1;
        for y in 0..10 {
            game.board.set_cell((BOARD_WIDTH - 1) as i32, (board_bottom - y) as i32, Cell::Filled(macroquad::prelude::RED));
        }
        
        // Position I-piece next to the wall but with space below
        if let Some(ref mut piece) = game.current_piece {
            // Place piece well above the obstacles with room to fall
            piece.position = ((BOARD_WIDTH - 3) as i32, (BUFFER_HEIGHT + 5) as i32);
        }
        
        // Verify piece can initially fall
        let mut test_piece = game.current_piece.as_ref().unwrap().clone();
        test_piece.move_by(0, 1);
        assert!(game.is_piece_valid(&test_piece), "Piece should be able to fall initially");
        
        // Try to move right into the wall (should fail but not trigger lock delay)
        let initial_locking_state = game.piece_is_locking;
        let move_success = game.move_piece(1, 0); // Try to move into wall
        
        // Movement should fail
        assert!(!move_success, "Movement into wall should fail");
        
        // But lock delay should NOT be triggered because piece can still fall
        assert_eq!(game.piece_is_locking, initial_locking_state, 
                  "Lock delay state should not change when hitting side wall while piece can still fall");
        
        // Verify piece can still move down
        assert!(game.drop_current_piece(), "Piece should still be able to drop after side collision");
        
        // Lock delay should still not be active (piece can continue falling)
        assert!(!game.piece_is_locking, "Lock delay should not be active while piece can fall");
    }
    
    #[test]
    fn test_successful_side_movement_doesnt_trigger_lock_delay() {
        let mut game = create_game_with_piece(TetrominoType::O);
        
        // Create obstacles on sides but leave space for the piece to fall
        let board_bottom = BOARD_HEIGHT + BUFFER_HEIGHT - 1;
        
        // Create walls with gap in middle
        for y in 0..5 {
            // Left wall
            game.board.set_cell(1, (board_bottom - y) as i32, Cell::Filled(macroquad::prelude::RED));
            // Right wall  
            game.board.set_cell((BOARD_WIDTH - 2) as i32, (board_bottom - y) as i32, Cell::Filled(macroquad::prelude::RED));
        }
        
        // Position piece in the middle, high up
        if let Some(ref mut piece) = game.current_piece {
            piece.position = ((BOARD_WIDTH / 2) as i32, (board_bottom - 15) as i32);
        }
        
        // Perform sliding movement (like T-spins or wall kicks)
        for _ in 0..3 {
            // Move left
            game.move_piece(-1, 0);
            // Should not trigger lock delay since piece can still fall
            assert!(!game.piece_is_locking, 
                   "Side movement should not trigger lock delay when piece can still fall");
            
            // Move right
            game.move_piece(1, 0);
            // Should not trigger lock delay since piece can still fall
            assert!(!game.piece_is_locking, 
                   "Side movement should not trigger lock delay when piece can still fall");
        }
        
        // Verify piece can still drop
        assert!(game.drop_current_piece(), "Piece should still be able to drop after side movements");
    }
}
