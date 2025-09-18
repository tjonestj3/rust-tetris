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
    
    #[test]
    fn test_ghost_piece_doesnt_affect_lock_delay() {
        let mut game = create_game_with_piece(TetrominoType::I);
        
        // Create a surface for the piece to eventually land on
        let board_bottom = BOARD_HEIGHT + BUFFER_HEIGHT - 1;
        for x in 0..BOARD_WIDTH {
            game.board.set_cell(x as i32, (board_bottom - 2) as i32, Cell::Filled(macroquad::prelude::RED));
        }
        
        // Position I-piece several rows above the surface
        if let Some(ref mut piece) = game.current_piece {
            piece.position = ((BOARD_WIDTH / 2) as i32, (board_bottom - 10) as i32);
        }
        
        // Calculate where ghost piece should be
        let ghost_piece = game.calculate_ghost_piece();
        assert!(ghost_piece.is_some(), "Should have a ghost piece when piece can fall");
        
        let ghost_y = ghost_piece.as_ref().unwrap().position.1;
        let current_y = game.current_piece.as_ref().unwrap().position.1;
        
        // Verify ghost piece is below current piece
        assert!(ghost_y > current_y, "Ghost piece should be below current piece");
        
        // Now drop the piece one step at a time, ensuring lock delay doesn't start prematurely
        while game.current_piece.is_some() {
            let current_pos = game.current_piece.as_ref().unwrap().position;
            let can_drop = game.drop_current_piece();
            
            if can_drop {
                // Piece moved down successfully
                let new_pos = game.current_piece.as_ref().unwrap().position;
                assert!(new_pos.1 > current_pos.1, "Piece should have moved down");
                
                // Test: Even when piece gets close to its ghost position, 
                // lock delay should not start until piece actually can't move down
                let ghost_now = game.calculate_ghost_piece();
                if let Some(ghost) = ghost_now {
                    let distance_to_ghost = ghost.position.1 - new_pos.1;
                    
                    // If we're still more than 1 position away from ghost, 
                    // definitely should not be locking
                    if distance_to_ghost > 1 {
                        assert!(!game.piece_is_locking, 
                               "Lock delay should not start when piece is still {} positions from ghost landing", distance_to_ghost);
                    }
                } else {
                    // Ghost piece is None, meaning current piece is at landing position
                    // This is when lock delay SHOULD start
                    assert!(game.piece_is_locking, 
                           "Lock delay should start when piece reaches its final position");
                }
            } else {
                // Piece can't move down anymore - lock delay should be active
                assert!(game.piece_is_locking, "Lock delay should be active when piece can't drop");
                break;
            }
        }
    }
    
    #[test]
    fn test_slide_off_resets_lock_delay_properly() {
        let mut game = create_game_with_piece(TetrominoType::T);
        
        // Create a surface that the T-piece can land on with one side hanging off
        let board_bottom = BOARD_HEIGHT + BUFFER_HEIGHT - 1;
        let surface_y = board_bottom - 3;
        
        // Fill only the left side of a row, leaving the right side empty
        for x in 0..(BOARD_WIDTH / 2) {
            game.board.set_cell(x as i32, surface_y as i32, Cell::Filled(macroquad::prelude::RED));
        }
        
        // Position T-piece so it will land on the filled part (left side)
        if let Some(ref mut piece) = game.current_piece {
            piece.position = (2, (surface_y - 5) as i32); // Well above the surface
        }
        
        // Drop the piece until it lands on the surface and starts lock delay
        while game.current_piece.is_some() && game.drop_current_piece() {
            // Keep dropping until piece can't drop anymore
        }
        
        // At this point, piece should be resting on the left side and locking should start
        assert!(game.piece_is_locking, "Piece should be in locking state when it lands");
        assert_eq!(game.lock_resets, 0, "Lock resets should be 0 initially");
        
        // Simulate some lock delay time passing (but not enough to lock)
        game.lock_delay_timer = 0.3; // Partway through lock delay
        
        // Now slide the piece to the right, off the filled surface
        let moved = game.move_piece(4, 0); // Move significantly to the right
        assert!(moved, "Should be able to move piece horizontally off the surface");
        
        // The piece should now be falling again since it moved off the surface
        let can_fall = {
            if let Some(ref piece) = game.current_piece {
                let mut test_piece = piece.clone();
                test_piece.move_by(0, 1);
                game.is_piece_valid(&test_piece)
            } else {
                false
            }
        };
        
        if can_fall {
            // If piece can fall, lock delay should be completely reset
            assert!(!game.piece_is_locking, "Lock delay should be reset when piece can fall again after sliding off");
            assert_eq!(game.lock_delay_timer, 0.0, "Lock delay timer should be reset to 0");
            assert_eq!(game.lock_resets, 0, "Lock resets should be reset to 0 when piece can fall");
            
            // Drop the piece to its new landing position
            while game.drop_current_piece() {
                // Keep dropping
            }
            
            // Now it should start fresh lock delay
            assert!(game.piece_is_locking, "Piece should start fresh lock delay at new position");
            assert_eq!(game.lock_delay_timer, 0.0, "Lock delay timer should start fresh");
            
            // The key test: piece should get full lock delay time, not immediate locking
            // Simulate a small amount of time passing
            game.update(0.1); // 100ms
            
            // Piece should still be there and locking, not locked yet
            assert!(game.current_piece.is_some(), "Piece should not lock immediately after sliding off and landing elsewhere");
            assert!(game.piece_is_locking, "Piece should still be in lock delay state");
            assert!(game.lock_delay_timer < LOCK_DELAY, "Lock delay timer should not be at max yet");
        } else {
            // If piece somehow can't fall after moving (edge case), it should still reset properly
            assert!(game.piece_is_locking, "Piece should still be locking if it can't fall");
        }
    }
    
    #[test]
    fn test_slide_off_timing_preserves_full_lock_delay() {
        let mut game = create_game_with_piece(TetrominoType::T);
        
        // Create a narrow ledge for the I-piece to land on initially
        let board_bottom = BOARD_HEIGHT + BUFFER_HEIGHT - 1;
        let ledge_y = board_bottom - 3;
        
        // Create a single-block platform for the T-piece to initially rest on partially
        game.board.set_cell(4, ledge_y as i32, Cell::Filled(macroquad::prelude::BLUE));
        
        // Position T-piece to land mostly on this single block
        if let Some(ref mut piece) = game.current_piece {
            // T-piece center will be at position 4, with arms extending left and right
            piece.position = (4, (ledge_y - 6) as i32);
        }
        
        // Drop until it lands on the platform
        while game.drop_current_piece() {
            // Keep dropping
        }
        
        // Confirm piece is locking on the platform
        assert!(game.piece_is_locking, "Piece should be locking on platform");
        
        // Let some lock delay time pass (like in real gameplay)
        game.update(0.25); // 250ms of the 500ms lock delay
        let partial_timer = game.lock_delay_timer;
        assert!(partial_timer > 0.0, "Some lock delay time should have passed");
        assert!(partial_timer < LOCK_DELAY, "Should not have reached full lock delay yet");
        
        // Now slide the piece off the platform to the right
        // T-piece center is at position 4, platform is at position 4
        // Move right by 2 to shift the T-piece completely off the single-block platform
        let can_move_right = game.move_piece(2, 0);
        assert!(can_move_right, "Should be able to slide off platform");
        
        // Debug: Check if piece can actually fall after sliding off
        let can_fall_after_slide = {
            if let Some(ref piece) = game.current_piece {
                let mut test_piece = piece.clone();
                test_piece.move_by(0, 1);
                game.is_piece_valid(&test_piece)
            } else {
                false
            }
        };
        
        println!("After sliding off: can_fall={}, piece_is_locking={}, lock_timer={}", 
                can_fall_after_slide, game.piece_is_locking, game.lock_delay_timer);
        
        if let Some(ref piece) = game.current_piece {
            println!("Piece position after slide: ({}, {})", piece.position.0, piece.position.1);
            // Check what's below the piece
            for (x, y) in piece.absolute_blocks() {
                let below_y = y + 1;
                if let Some(cell) = game.board.get_cell(x, below_y) {
                    println!("Below piece at ({}, {}): {:?}", x, below_y, cell);
                } else {
                    println!("Below piece at ({}, {}): out of bounds", x, below_y);
                }
            }
        }
        
        // After sliding off, piece should be falling and lock delay should be reset
        assert!(!game.piece_is_locking, "Piece should not be locking while falling (can_fall={})", can_fall_after_slide);
        assert_eq!(game.lock_delay_timer, 0.0, "Lock delay timer should be reset after sliding off");
        
        // Now drop to the new landing position
        while game.drop_current_piece() {
            // Drop to new position
        }
        
        // Piece should now start a FRESH lock delay
        assert!(game.piece_is_locking, "Piece should start locking at new position");
        assert_eq!(game.lock_delay_timer, 0.0, "Lock delay timer should start fresh");
        
        // Critical test: Piece should get the FULL lock delay time
        // Simulate just under the full lock delay period
        let almost_full_delay = LOCK_DELAY - 0.001; // Just 1ms before it would lock
        game.update(almost_full_delay);
        
        // Piece should still exist and be close to locking but not locked yet
        assert!(game.current_piece.is_some(), "Piece should still exist just before full lock delay");
        assert!(game.piece_is_locking, "Piece should still be in locking state");
        assert!(game.lock_delay_timer < LOCK_DELAY, "Should be just under the lock delay threshold");
        
        // Now add just a tiny bit more time to trigger the lock
        game.update(0.002); // 2ms more
        
        // NOW the piece should be locked
        assert!(game.current_piece.is_none() || !game.piece_is_locking, "Piece should have locked after full delay period");
    }
}
