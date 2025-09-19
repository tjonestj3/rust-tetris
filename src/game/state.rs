//! Game state management

use crate::board::{Board, Cell};
use crate::tetromino::{Tetromino, TetrominoType};
use crate::game::config::*;
use serde::{Serialize, Deserialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::fs;
use std::path::Path;

/// Game states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
}

/// Main game struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    /// Current game state
    pub state: GameState,
    /// The game board
    pub board: Board,
    /// Currently falling piece
    pub current_piece: Option<Tetromino>,
    /// Next piece to spawn
    pub next_piece: TetrominoType,
    /// Held piece (can be swapped with current piece)
    pub held_piece: Option<TetrominoType>,
    /// Whether hold has been used for the current piece (prevents infinite swapping)
    pub hold_used_this_piece: bool,
    /// Current score
    pub score: u32,
    /// Time accumulator for piece dropping
    pub drop_timer: f64,
    /// Time between drops (decreases with level)
    pub drop_interval: f64,
    /// Game time in seconds
    pub game_time: f64,
    /// Lines being cleared with animation
    pub clearing_lines: Vec<usize>,
    /// Line clearing animation timer
    pub clear_animation_timer: f64,
    /// Soft drop input timer
    pub soft_drop_timer: f64,
    /// Left movement input timer
    pub left_move_timer: f64,
    /// Right movement input timer
    pub right_move_timer: f64,

    /// Ghost blocks available for placement
    pub ghost_blocks_available: u32,
    /// Ghost block placement mode active
    pub ghost_block_placement_mode: bool,
    /// Ghost block cursor position (x, y)
    pub ghost_block_cursor: (i32, i32),
    /// Ghost block blink timer for animation
    pub ghost_block_blink_timer: f64,
    /// Smart positions sorted by strategic value (best first)
    pub ghost_smart_positions: Vec<(i32, i32, u32)>, // (x, y, blocks_needed_to_complete_line)
    /// Current index in smart positions list
    pub ghost_cursor_index: usize,

    /// Flag to track when a piece was just locked (for audio feedback)
    pub piece_just_locked: bool,
    /// Lock delay timer - tracks how long piece has been unable to move down
    pub lock_delay_timer: f64,
    /// Whether the current piece is in the "locking" state (can't move down)
    pub piece_is_locking: bool,
    /// Number of times lock delay has been reset for current piece
    pub lock_resets: u32,
    /// Total time the current piece has been active (prevents infinite floating)
    pub piece_lifetime_timer: f64,
    
    /// TETRIS celebration state
    pub tetris_celebration_active: bool,
    /// TETRIS celebration timer for animation
    pub tetris_celebration_timer: f64,
    
    /// Ghost block throwing animation state
    pub ghost_throw_active: bool,
    /// Ghost block throwing animation timer
    pub ghost_throw_timer: f64,
    /// Target position for ghost block throw
    pub ghost_throw_target: (i32, i32),
    /// Starting position for throw animation
    pub ghost_throw_start: (f32, f32),
    
    /// Legacy mode flag - when true, renders blocks as ASCII characters like Pajitnov's original
    pub legacy_mode: bool,
    
    /// Track if the last successful action was a rotation (for T-spin detection)
    pub last_action_was_rotation: bool,
}

impl Game {
    /// Create a new game instance
    pub fn new() -> Self {
        let mut game = Self {
            state: GameState::Playing,
            board: Board::new(),
            current_piece: None,
            next_piece: TetrominoType::random(),
            held_piece: None,
            hold_used_this_piece: false,
            score: 0,
            drop_timer: 0.0,
            drop_interval: 1.0, // Will be set properly by update_drop_interval()
            game_time: 0.0,
            clearing_lines: Vec::new(),
            clear_animation_timer: 0.0,
            soft_drop_timer: 0.0,
            left_move_timer: 0.0,
            right_move_timer: 0.0,

            ghost_blocks_available: 0,
            ghost_block_placement_mode: false,
            ghost_block_cursor: (BOARD_WIDTH as i32 / 2, (BUFFER_HEIGHT + VISIBLE_HEIGHT / 2) as i32),
            ghost_block_blink_timer: 0.0,
            ghost_smart_positions: Vec::new(),
            ghost_cursor_index: 0,

            piece_just_locked: false,
            lock_delay_timer: 0.0,
            piece_is_locking: false,
            lock_resets: 0,
            piece_lifetime_timer: 0.0,
            
            tetris_celebration_active: false,
            tetris_celebration_timer: 0.0,
            
            ghost_throw_active: false,
            ghost_throw_timer: 0.0,
            ghost_throw_target: (0, 0),
            ghost_throw_start: (0.0, 0.0),
            
            legacy_mode: false, // Start in modern mode by default
            last_action_was_rotation: false,
        };
        
        // Spawn the first piece
        game.spawn_next_piece();
        
        // Initialize drop interval based on starting level
        game.update_drop_interval();
        
        game
    }
    
    /// Update game logic
    pub fn update(&mut self, delta_time: f64) {
        if self.state != GameState::Playing {
            return;
        }
        
        // Reset piece locked flag at the start of each update cycle
        self.piece_just_locked = false;
        
        self.game_time += delta_time;
        
        // Handle line clearing animation
        if !self.clearing_lines.is_empty() {
            self.clear_animation_timer += delta_time;
            if self.clear_animation_timer >= LINE_CLEAR_ANIMATION_TIME {
                self.finish_line_clear();
            }
            return; // Don't update other game logic during animation
        }
        
        self.drop_timer += delta_time;
        self.soft_drop_timer += delta_time;
        self.left_move_timer += delta_time;
        self.right_move_timer += delta_time;
        self.ghost_block_blink_timer += delta_time;
        
        // Update piece lifetime timer
        if self.current_piece.is_some() {
            self.piece_lifetime_timer += delta_time;
        }
        
        // Update TETRIS celebration timer
        if self.tetris_celebration_active {
            self.tetris_celebration_timer += delta_time;
            if self.tetris_celebration_timer >= TETRIS_CELEBRATION_TIME {
                self.tetris_celebration_active = false;
                self.tetris_celebration_timer = 0.0;
            }
        }
        
        // Update ghost throw animation timer
        if self.ghost_throw_active {
            self.ghost_throw_timer += delta_time;
            if self.ghost_throw_timer >= GHOST_THROW_ANIMATION_TIME {
                self.finish_ghost_throw();
            }
        }
        
        // Check for force lock if piece has exceeded maximum lifetime
        // This is a critical safeguard against floating pieces
        if self.piece_lifetime_timer >= MAX_PIECE_LIFETIME {
            log::warn!("Piece exceeded maximum lifetime of {}s, force-locking to prevent floating bug", MAX_PIECE_LIFETIME);
            self.lock_current_piece();
            return; // Don't continue with other logic after locking
        }
        
        // Update lock delay timer if piece is in locking state
        if self.piece_is_locking {
            self.lock_delay_timer += delta_time;
            // Check if lock delay time has expired
            if self.lock_delay_timer >= LOCK_DELAY {
                self.lock_current_piece();
                return; // Don't continue with other logic after locking
            }
        }
        
        // Check if it's time to drop the current piece
        if self.drop_timer >= self.drop_interval {
            self.drop_current_piece();
            self.drop_timer = 0.0;
        }
    }
    
    /// Try to drop the current piece by one row
    pub fn drop_current_piece(&mut self) -> bool {
        if let Some(mut piece) = self.current_piece.clone() {
            // Try to move down
            piece.move_by(0, 1);
            
            if self.is_piece_valid(&piece) {
                // Successfully moved down - reset lock delay state
                self.current_piece = Some(piece);
                self.piece_is_locking = false;
                self.lock_delay_timer = 0.0;
                return true;
            } else {
                // Can't move down - start lock delay if not already started
                if !self.piece_is_locking {
                    self.piece_is_locking = true;
                    self.lock_delay_timer = 0.0;
                }
                return false;
            }
        }
        false
    }
    
    /// Check if the current piece is in a valid position
    pub fn is_piece_valid(&self, piece: &Tetromino) -> bool {
        for (x, y) in piece.absolute_blocks() {
            if !self.board.is_position_valid(x, y) {
                return false;
            }
        }
        true
    }
    
    /// Lock the current piece to the board and spawn a new one
    pub fn lock_current_piece(&mut self) {
        if let Some(piece) = self.current_piece.take() {
            // Debug logging for piece locking
            log::debug!("Locking piece {:?} at position ({}, {}) after {:.2}s lifetime, {} lock resets",
                       piece.piece_type, piece.position.0, piece.position.1, 
                       self.piece_lifetime_timer, self.lock_resets);
            
            // Set flag to indicate a piece was just locked (for audio feedback)
            self.piece_just_locked = true;
            
            // Reset lock delay state
            self.piece_is_locking = false;
            self.lock_delay_timer = 0.0;
            self.lock_resets = 0;
            self.piece_lifetime_timer = 0.0;
            
            // Place the piece on the board
            for (x, y) in piece.absolute_blocks() {
                if x >= 0 && y >= 0 {
                    self.board.set_cell(x, y, Cell::Filled(piece.color()));
                }
            }
            
            // Check for complete lines and start animation
            let complete_lines = self.board.find_complete_lines();
            if !complete_lines.is_empty() {
                self.start_line_clear_animation(complete_lines);
                return; // Don't spawn next piece until animation is done
            }
            
            // Check game over
            if self.board.is_game_over() {
                self.state = GameState::GameOver;
                return;
            }
            
            // Spawn next piece
            self.spawn_next_piece();
        }
    }
    
    /// Spawn the next piece
    pub fn spawn_next_piece(&mut self) {
        let new_piece = Tetromino::new(self.next_piece);
        log::debug!("Spawning new piece: {:?} at position ({}, {})", 
                   new_piece.piece_type, new_piece.position.0, new_piece.position.1);
        self.next_piece = TetrominoType::random();
        
        // Reset hold usage for the new piece
        self.hold_used_this_piece = false;
        
        // Reset lock delay state for new piece
        self.piece_is_locking = false;
        self.lock_delay_timer = 0.0;
        self.lock_resets = 0;
        self.piece_lifetime_timer = 0.0;
        
        // Update drop interval if level changed
        self.update_drop_interval();
        
        // Reset T-spin detection for new piece
        self.last_action_was_rotation = false;
        
        // Check if the new piece can be placed
        if self.is_piece_valid(&new_piece) {
            self.current_piece = Some(new_piece);
        } else {
            // Game over - can't spawn new piece
            log::warn!("Game over: Cannot spawn piece {:?} - board is full", new_piece.piece_type);
            self.state = GameState::GameOver;
        }
    }
    
    /// Add score for cleared lines
    pub fn add_score_for_lines(&mut self, lines_cleared: u32) {
        let base_score = match lines_cleared {
            1 => SCORE_SINGLE_LINE,
            2 => SCORE_DOUBLE_LINE,
            3 => SCORE_TRIPLE_LINE,
            4 => SCORE_TETRIS,
            _ => 0,
        };
        
        // Multiply by level for higher scores at higher levels
        self.score += base_score * self.board.level();
    }
    
    /// Try to move the current piece
    pub fn move_piece(&mut self, dx: i32, dy: i32) -> bool {
        if let Some(mut piece) = self.current_piece.clone() {
            piece.move_by(dx, dy);
            
            if self.is_piece_valid(&piece) {
                // Movement was successful - update piece position
                self.current_piece = Some(piece);
                
                // Movement resets rotation tracking for T-spin detection
                self.last_action_was_rotation = false;
                
                // NOW check if the piece can still fall from its CURRENT position
                // This prevents side collisions from triggering lock delay
                self.update_lock_state_for_current_piece();
                
                return true;
            }
        }
        false
    }
    
    /// Update lock delay state based on whether current piece can continue falling
    /// This should be called after any successful piece movement or rotation
    fn update_lock_state_for_current_piece(&mut self) {
        if let Some(ref piece) = self.current_piece {
            // Test if piece can move down from its CURRENT position
            let mut test_piece = piece.clone();
            test_piece.move_by(0, 1);
            
            if self.is_piece_valid(&test_piece) {
                // Piece can still fall - reset lock delay completely
                self.reset_lock_delay();
                log::debug!("Piece can still fall from current position - lock delay reset");
            } else {
                // Piece is truly grounded - start/continue lock delay
                if !self.piece_is_locking {
                    self.piece_is_locking = true;
                    self.lock_delay_timer = 0.0;
                    log::debug!("Piece is now grounded and cannot fall - starting lock delay");
                }
                // Note: We don't automatically reset lock delay for grounded pieces
                // Lock delay resets are handled explicitly in reset_lock_delay() method
            }
        }
    }
    
    /// Try to rotate the current piece clockwise
    pub fn rotate_piece_clockwise(&mut self) -> bool {
        if let Some(mut piece) = self.current_piece.clone() {
            piece.rotate_clockwise();
            
            if self.is_piece_valid(&piece) {
                self.current_piece = Some(piece);
                // Mark that the last successful action was a rotation
                self.last_action_was_rotation = true;
                // Check lock state after successful rotation
                self.update_lock_state_for_current_piece();
                return true;
            }
        }
        false
    }
    
    /// Try to rotate the current piece counterclockwise
    pub fn rotate_piece_counterclockwise(&mut self) -> bool {
        if let Some(mut piece) = self.current_piece.clone() {
            piece.rotate_counterclockwise();
            
            if self.is_piece_valid(&piece) {
                self.current_piece = Some(piece);
                // Mark that the last successful action was a rotation
                self.last_action_was_rotation = true;
                // Check lock state after successful rotation
                self.update_lock_state_for_current_piece();
                return true;
            }
        }
        false
    }
    
    /// Hard drop the current piece
    pub fn hard_drop(&mut self) {
        if self.current_piece.is_some() {
            let mut drop_distance = 0;
            
            // Drop as far as possible
            while self.drop_current_piece() {
                drop_distance += 1;
            }
            
            // Add score for hard drop
            self.score += (drop_distance as u32) * SCORE_HARD_DROP;
            
            // Immediately lock the piece after hard drop - no lock delay
            self.lock_current_piece();
        }
    }
    
    /// Pause/unpause the game
    pub fn toggle_pause(&mut self) {
        match self.state {
            GameState::Playing => self.state = GameState::Paused,
            GameState::Paused => self.state = GameState::Playing,
            _ => {}, // Can't pause in other states
        }
    }
    
    /// Reset the game
    pub fn reset(&mut self) {
        *self = Self::new();
    }
    
    /// Toggle legacy mode (inspired by Pajitnov's original terminal version)
    pub fn toggle_legacy_mode(&mut self) {
        self.legacy_mode = !self.legacy_mode;
        log::info!("Legacy mode {}", if self.legacy_mode { "ENABLED - Switching to terminal-style ASCII blocks" } else { "DISABLED - Switching to modern graphics" });
    }
    
    /// Check if legacy mode is currently active
    pub fn is_legacy_mode(&self) -> bool {
        self.legacy_mode
    }
    
    /// Get current level
    pub fn level(&self) -> u32 {
        self.board.level()
    }
    
    /// Get lines cleared
    pub fn lines_cleared(&self) -> u32 {
        self.board.lines_cleared()
    }
    
    /// Start line clearing animation
    pub fn start_line_clear_animation(&mut self, lines: Vec<usize>) {
        self.clearing_lines = lines;
        self.clear_animation_timer = 0.0;
    }
    
    /// Finish line clearing animation and actually clear the lines
    pub fn finish_line_clear(&mut self) {
        if !self.clearing_lines.is_empty() {
            let lines_cleared = self.board.clear_lines(&self.clearing_lines);
            self.add_score_for_lines(lines_cleared);
            
            // Check for TETRIS celebration (4 lines cleared at once)
            if lines_cleared == 4 {
                self.tetris_celebration_active = true;
                self.tetris_celebration_timer = 0.0;
                log::info!("TETRIS! 4 lines cleared - starting celebration!");
            }
            
            // Award ghost block every 4 lines cleared
            let total_lines_before = self.board.lines_cleared() - lines_cleared;
            let total_lines_after = self.board.lines_cleared();
            let ghost_blocks_before = total_lines_before / 4;
            let ghost_blocks_after = total_lines_after / 4;
            let ghost_blocks_earned = ghost_blocks_after - ghost_blocks_before;
            
            if ghost_blocks_earned > 0 {
                self.ghost_blocks_available += ghost_blocks_earned;
                log::info!("Ghost block earned! {} available", self.ghost_blocks_available);
            }
            
            self.clearing_lines.clear();
            self.clear_animation_timer = 0.0;
        }
        
        // Check game over after clearing lines
        if self.board.is_game_over() {
            self.state = GameState::GameOver;
            return;
        }
        
        // Only spawn next piece if we don't have a current piece
        if self.current_piece.is_none() {
            self.spawn_next_piece();
        } else {
            // If we have a current piece after line clearing, ensure it can continue falling normally
            // Reset lock delay state so the piece can continue its natural fall
            // DO NOT force reposition the piece - let natural game physics handle it
            self.piece_is_locking = false;
            self.lock_delay_timer = 0.0;
            
            // Log the state for debugging
            if let Some(ref piece) = self.current_piece {
                let is_valid = self.is_piece_valid(piece);
                log::debug!("After line clear: piece at ({}, {}) is {}", 
                           piece.position.0, piece.position.1, 
                           if is_valid { "valid and can continue falling" } else { "invalid - will be handled by normal game logic" });
            }
        }
    }
    
    /// Handle continuous soft drop
    pub fn update_soft_drop(&mut self, is_held: bool) {
        if is_held && self.soft_drop_timer >= SOFT_DROP_INTERVAL {
            if self.move_piece(0, 1) {
                self.score += SCORE_SOFT_DROP;
                self.soft_drop_timer = 0.0;
            }
        }
        
        if !is_held {
            self.soft_drop_timer = SOFT_DROP_INTERVAL; // Allow immediate drop when pressed
        }
    }
    
    /// Handle continuous left movement
    pub fn update_left_movement(&mut self, is_held: bool) {
        if is_held && self.left_move_timer >= HORIZONTAL_MOVE_INTERVAL {
            self.move_piece(-1, 0);
            self.left_move_timer = 0.0;
        }
        
        if !is_held {
            self.left_move_timer = HORIZONTAL_MOVE_INTERVAL; // Allow immediate move when pressed
        }
    }
    
    /// Handle continuous right movement
    pub fn update_right_movement(&mut self, is_held: bool) {
        if is_held && self.right_move_timer >= HORIZONTAL_MOVE_INTERVAL {
            self.move_piece(1, 0);
            self.right_move_timer = 0.0;
        }
        
        if !is_held {
            self.right_move_timer = HORIZONTAL_MOVE_INTERVAL; // Allow immediate move when pressed
        }
    }
    
    /// Check if lines are currently being cleared (for rendering)
    pub fn is_clearing_lines(&self) -> bool {
        !self.clearing_lines.is_empty()
    }
    
    /// Save the game state to a file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        log::info!("Game saved successfully");
        Ok(())
    }
    
    /// Load the game state from a file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let json = fs::read_to_string(path)?;
        let game: Game = serde_json::from_str(&json)?;
        log::info!("Game loaded successfully");
        Ok(game)
    }
    
    /// Check if a save file exists
    pub fn save_file_exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }
    
    /// Get the default save file path
    pub fn default_save_path() -> std::path::PathBuf {
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("tetris_save.json")
    }
    
    /// Get a hash of the current game state for efficient change detection
    pub fn get_state_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        // Hash key game state components that matter for saves
        self.score.hash(&mut hasher);
        self.board.lines_cleared().hash(&mut hasher);
        self.board.level().hash(&mut hasher);
        self.ghost_blocks_available.hash(&mut hasher);
        // Hash current piece position and type
        if let Some(ref piece) = self.current_piece {
            piece.piece_type.hash(&mut hasher);
            piece.position.hash(&mut hasher);
            piece.rotation.hash(&mut hasher);
        }
        self.next_piece.hash(&mut hasher);
        self.held_piece.hash(&mut hasher);
        // Hash filled cells in board (simplified)
        self.board.filled_cells_count().hash(&mut hasher);
        hasher.finish()
    }
    
    /// Get the lines being cleared (for animation rendering)
    pub fn get_clearing_lines(&self) -> &[usize] {
        &self.clearing_lines
    }
    
    /// Get the clear animation progress (0.0 to 1.0)
    pub fn get_clear_animation_progress(&self) -> f64 {
        if self.clearing_lines.is_empty() {
            0.0
        } else {
            (self.clear_animation_timer / LINE_CLEAR_ANIMATION_TIME).min(1.0)
        }
    }
    
    /// Hold the current piece (swap with held piece)
    /// Can only be used once per piece to prevent infinite swapping
    pub fn hold_piece(&mut self) -> bool {
        // Can't hold if already used for this piece
        if self.hold_used_this_piece {
            return false;
        }
        
        // Can't hold if no current piece
        if self.current_piece.is_none() {
            return false;
        }
        
        // Mark hold as used for this "piece cycle"
        self.hold_used_this_piece = true;
        
        if let Some(current) = self.current_piece.take() {
            match self.held_piece {
                Some(held_type) => {
                    // Swap current piece with held piece
                    self.held_piece = Some(current.piece_type);
                    let new_piece = Tetromino::new(held_type);
                    
                    // Check if the swapped piece can be placed
                    if self.is_piece_valid(&new_piece) {
                        self.current_piece = Some(new_piece);
                        // Reset lock delay for held piece
                        self.reset_lock_delay();
                    } else {
                        // Can't place swapped piece - game over
                        self.held_piece = Some(current.piece_type); // Keep the piece in hold
                        self.state = GameState::GameOver;
                        return false;
                    }
                }
                None => {
                    // First time holding - store current piece and spawn next
                    self.held_piece = Some(current.piece_type);
                    // Don't reset hold_used_this_piece when manually spawning in hold context
                    let new_piece = Tetromino::new(self.next_piece);
                    self.next_piece = TetrominoType::random();
                    
                    // Check if the new piece can be placed
                    if self.is_piece_valid(&new_piece) {
                        self.current_piece = Some(new_piece);
                        // Reset lock delay for new piece from hold
                        self.reset_lock_delay();
                    } else {
                        // Game over - can't spawn new piece
                        self.state = GameState::GameOver;
                        return false;
                    }
                }
            }
        }
        
        true
    }
    
    /// Check if hold is available for the current piece
    pub fn can_hold(&self) -> bool {
        !self.hold_used_this_piece && self.current_piece.is_some()
    }
    
    /// Reset the lock delay timer and state with improved anti-floating logic
    pub fn reset_lock_delay(&mut self) {
        // Always allow reset if piece can actually move down (not grounded)
        if let Some(ref piece) = self.current_piece {
            let mut test_piece = piece.clone();
            test_piece.move_by(0, 1);
            
            if self.is_piece_valid(&test_piece) {
                // Piece can move down - allow reset regardless of reset count
                self.piece_is_locking = false;
                self.lock_delay_timer = 0.0;
                self.lock_resets = 0; // Reset counter since piece can move down
                log::debug!("Lock delay reset: piece can still fall");
                return;
            }
        }
        
        // Piece is grounded - only reset if we haven't exceeded the maximum number of resets
        if self.lock_resets < MAX_LOCK_RESETS {
            self.piece_is_locking = false;
            self.lock_delay_timer = 0.0;
            self.lock_resets += 1;
            log::debug!("Lock delay reset #{}: grounded piece gets more time", self.lock_resets);
        } else {
            log::debug!("Lock delay reset denied: max resets ({}) exceeded, piece will lock soon", MAX_LOCK_RESETS);
            // Force the piece into locking state if it wasn't already
            if !self.piece_is_locking {
                self.piece_is_locking = true;
                self.lock_delay_timer = 0.0;
            }
        }
    }
    
    
    /// Calculate where the current piece will land (ghost piece position)
    pub fn calculate_ghost_piece(&self) -> Option<Tetromino> {
        if let Some(mut ghost_piece) = self.current_piece.clone() {
            // Drop the ghost piece as far as it can go
            loop {
                ghost_piece.move_by(0, 1);
                if !self.is_piece_valid(&ghost_piece) {
                    // Move back one step to the last valid position
                    ghost_piece.move_by(0, -1);
                    break;
                }
            }
            
            // Only return ghost piece if it's different from current position
            if let Some(ref current) = self.current_piece {
                if ghost_piece.position.1 != current.position.1 {
                    return Some(ghost_piece);
                }
            }
        }
        None
    }
    
    /// Toggle ghost block placement mode
    pub fn toggle_ghost_block_mode(&mut self) {
        if self.ghost_blocks_available > 0 {
            self.ghost_block_placement_mode = !self.ghost_block_placement_mode;
            if self.ghost_block_placement_mode {
                // Analyze board and find smart positions
                self.analyze_smart_positions();
                self.ghost_block_blink_timer = 0.0;
                log::info!("Ghost block placement mode activated - targeting strategic positions in rows with existing blocks");
                
                // Auto-fire if the best position only needs 1 block (instant TETRIS setup)
                if let Some(&(x, y, blocks_needed)) = self.ghost_smart_positions.first() {
                    if blocks_needed == 1 {
                        log::info!("Auto-firing ghost block for optimal 1-block position at ({}, {})", x, y);
                        self.start_ghost_throw(x, y);
                        return; // Exit placement mode immediately
                    }
                }
            } else {
                log::info!("Ghost block placement mode deactivated");
                self.ghost_smart_positions.clear();
                self.ghost_cursor_index = 0;
            }
        }
    }
    
    /// Move to next smart position
    pub fn next_smart_position(&mut self) {
        if self.ghost_block_placement_mode && !self.ghost_smart_positions.is_empty() {
            self.ghost_cursor_index = (self.ghost_cursor_index + 1) % self.ghost_smart_positions.len();
            let (x, y, _) = self.ghost_smart_positions[self.ghost_cursor_index];
            self.ghost_block_cursor = (x, y);
            log::debug!("Next smart position: ({}, {}) - index {}", x, y, self.ghost_cursor_index);
        }
    }
    
    /// Move to previous smart position
    pub fn previous_smart_position(&mut self) {
        if self.ghost_block_placement_mode && !self.ghost_smart_positions.is_empty() {
            self.ghost_cursor_index = if self.ghost_cursor_index == 0 {
                self.ghost_smart_positions.len() - 1
            } else {
                self.ghost_cursor_index - 1
            };
            let (x, y, _) = self.ghost_smart_positions[self.ghost_cursor_index];
            self.ghost_block_cursor = (x, y);
            log::debug!("Previous smart position: ({}, {}) - index {}", x, y, self.ghost_cursor_index);
        }
    }
    
    /// Move ghost block cursor manually (for arrow keys)
    pub fn move_ghost_block_cursor(&mut self, dx: i32, dy: i32) {
        if self.ghost_block_placement_mode {
            let new_x = (self.ghost_block_cursor.0 + dx).max(0).min(BOARD_WIDTH as i32 - 1);
            let new_y = (self.ghost_block_cursor.1 + dy).max(BUFFER_HEIGHT as i32).min((BOARD_HEIGHT + BUFFER_HEIGHT - 1) as i32);
            self.ghost_block_cursor = (new_x, new_y);
            
            // When manually moving, find the closest smart position and update index
            self.update_cursor_index_for_position(new_x, new_y);
        }
    }
    
    /// Update cursor index to match the current position (for manual movement)
    fn update_cursor_index_for_position(&mut self, x: i32, y: i32) {
        if let Some(index) = self.ghost_smart_positions.iter().position(|(px, py, _)| *px == x && *py == y) {
            self.ghost_cursor_index = index;
        }
        // If position is not in smart positions, keep current index
    }
    
    /// Get strategic info for current cursor position
    pub fn get_current_position_info(&self) -> Option<(usize, usize, u32)> {
        if self.ghost_block_placement_mode && !self.ghost_smart_positions.is_empty() {
            if let Some(&(_, _, blocks_needed)) = self.ghost_smart_positions.get(self.ghost_cursor_index) {
                return Some((self.ghost_cursor_index + 1, self.ghost_smart_positions.len(), blocks_needed));
            }
        }
        None
    }
    
    /// Place a ghost block at the current cursor position (with throwing animation)
    pub fn place_ghost_block(&mut self) -> bool {
        if self.ghost_block_placement_mode && self.ghost_blocks_available > 0 && !self.ghost_throw_active {
            let (x, y) = self.ghost_block_cursor;
            
            // Check if position is valid (empty)
            if let Some(cell) = self.board.get_cell(x, y) {
                if cell.is_empty() {
                    // Start throwing animation instead of instant placement
                    self.start_ghost_throw(x, y);
                    return true;
                }
            }
        }
        false
    }
    
    /// Check if ghost block cursor should be visible (always visible when in placement mode)
    pub fn is_ghost_cursor_visible(&self) -> bool {
        self.ghost_block_placement_mode
    }
    
    /// Analyze board and find smart positions for ghost block placement
    pub fn analyze_smart_positions(&mut self) {
        let mut positions = Vec::new();
        
        // Check each empty position on the board, but only on rows that have existing blocks
        for y in BUFFER_HEIGHT..(BOARD_HEIGHT + BUFFER_HEIGHT) {
            // First, check if this row has any existing blocks
            let row_has_blocks = self.row_has_existing_blocks(y);
            
            if row_has_blocks {
                for x in 0..BOARD_WIDTH {
                    let x_i32 = x as i32;
                    let y_i32 = y as i32;
                    
                    // Only consider empty positions
                    if let Some(cell) = self.board.get_cell(x_i32, y_i32) {
                        if cell.is_empty() {
                            // Calculate how many blocks are needed to complete this line
                            let blocks_needed = self.calculate_blocks_needed_for_line(y);
                            if blocks_needed > 0 {
                                positions.push((x_i32, y_i32, blocks_needed));
                            }
                        }
                    }
                }
            }
        }
        
        // Sort positions by strategic value:
        // 1. Fewer blocks needed to complete line (better)
        // 2. Lower row number (closer to bottom, better)
        // 3. Closer to center horizontally (better)
        positions.sort_by(|a, b| {
            // Primary: blocks needed (ascending - fewer is better)
            match a.2.cmp(&b.2) {
                std::cmp::Ordering::Equal => {
                    // Secondary: row position (descending - lower rows first)
                    match b.1.cmp(&a.1) {
                        std::cmp::Ordering::Equal => {
                            // Tertiary: distance from center (ascending - closer to center is better)
                            let center = BOARD_WIDTH as i32 / 2;
                            let dist_a = (a.0 - center).abs();
                            let dist_b = (b.0 - center).abs();
                            dist_a.cmp(&dist_b)
                        }
                        other => other,
                    }
                }
                other => other,
            }
        });
        
        self.ghost_smart_positions = positions;
        self.ghost_cursor_index = 0;
        
        // Set initial cursor position to the best position (if any)
        if let Some(&(x, y, _)) = self.ghost_smart_positions.first() {
            self.ghost_block_cursor = (x, y);
        }
        
        log::info!("Found {} smart positions for strategic ghost block placement (only targeting rows with existing blocks)", self.ghost_smart_positions.len());
    }
    
    /// Check if a row has any existing blocks (not completely empty)
    fn row_has_existing_blocks(&self, line_y: usize) -> bool {
        for x in 0..BOARD_WIDTH {
            if let Some(cell) = self.board.get_cell(x as i32, line_y as i32) {
                if cell.is_filled() {
                    return true;
                }
            }
        }
        false
    }
    
    /// Calculate how many blocks are needed to complete a specific line
    fn calculate_blocks_needed_for_line(&self, line_y: usize) -> u32 {
        let mut empty_count = 0;
        for x in 0..BOARD_WIDTH {
            if let Some(cell) = self.board.get_cell(x as i32, line_y as i32) {
                if cell.is_empty() {
                    empty_count += 1;
                }
            }
        }
        empty_count
    }
    
    /// Check if TETRIS celebration is currently active
    pub fn is_tetris_celebration_active(&self) -> bool {
        self.tetris_celebration_active
    }
    
    /// Get the TETRIS celebration animation progress (0.0 to 1.0)
    pub fn get_tetris_celebration_progress(&self) -> f64 {
        if self.tetris_celebration_active {
            (self.tetris_celebration_timer / TETRIS_CELEBRATION_TIME).min(1.0)
        } else {
            0.0
        }
    }
    
    /// Start ghost block throwing animation
    fn start_ghost_throw(&mut self, target_x: i32, target_y: i32) {
        // Calculate starting position (off-screen or from a corner)
        let start_x = BOARD_OFFSET_X - 100.0; // Start from left side off-screen
        let start_y = BOARD_OFFSET_Y + 50.0;  // Slightly below board top
        
        self.ghost_throw_active = true;
        self.ghost_throw_timer = 0.0;
        self.ghost_throw_target = (target_x, target_y);
        self.ghost_throw_start = (start_x, start_y);
        self.ghost_block_placement_mode = false; // Exit placement mode
        
        // Simply reset lock delay state when exiting ghost block mode
        // Let natural game physics handle piece positioning
        if self.current_piece.is_some() {
            self.reset_lock_delay();
            log::debug!("Ghost block mode exited - lock delay reset for current piece");
        }
        
        log::info!("Starting ghost block throw animation to ({}, {})", target_x, target_y);
    }
    
    /// Finish ghost block throwing animation and place the block
    fn finish_ghost_throw(&mut self) {
        let (target_x, target_y) = self.ghost_throw_target;
        
        // Actually place the block now
        self.board.set_cell(target_x, target_y, Cell::Filled(macroquad::prelude::Color::new(0.8, 0.8, 1.0, 1.0)));
        self.ghost_blocks_available -= 1;
        
        // Check if this placement creates any complete lines
        let complete_lines = self.board.find_complete_lines();
        if !complete_lines.is_empty() {
            self.start_line_clear_animation(complete_lines);
        }
        
        // Reset animation state
        self.ghost_throw_active = false;
        self.ghost_throw_timer = 0.0;
        
        log::info!("Ghost block thrown and placed at ({}, {}). Remaining: {}", 
                  target_x, target_y, self.ghost_blocks_available);
    }
    
    /// Check if ghost throw animation is currently active
    pub fn is_ghost_throw_active(&self) -> bool {
        self.ghost_throw_active
    }
    
    /// Light validation for current piece - only handles extreme cases
    fn validate_current_piece_position(&mut self) {
        // Only validate that we have a piece - don't force repositioning
        // Let the normal game update loop handle positioning via natural physics
        if let Some(ref piece) = self.current_piece {
            if !self.is_piece_valid(piece) {
                log::debug!("Current piece in invalid position after ghost operation - will be handled by normal game logic");
                // Reset lock delay to give the piece a chance to find a valid position naturally
                self.reset_lock_delay();
            } else {
                log::debug!("Current piece remains in valid position after ghost operation");
            }
        }
    }
    
    /// Get current throw animation progress and positions
    pub fn get_ghost_throw_info(&self) -> Option<(f64, (f32, f32), (f32, f32))> {
        if self.ghost_throw_active {
            let progress = (self.ghost_throw_timer / GHOST_THROW_ANIMATION_TIME).min(1.0);
            let target_screen = (
                BOARD_OFFSET_X + (self.ghost_throw_target.0 as f32 * CELL_SIZE) + CELL_SIZE / 2.0,
                BOARD_OFFSET_Y + ((self.ghost_throw_target.1 - BUFFER_HEIGHT as i32) as f32 * CELL_SIZE) + CELL_SIZE / 2.0
            );
            Some((progress, self.ghost_throw_start, target_screen))
        } else {
            None
        }
    }
    
    /// Get debug information about current piece state (for debugging locking issues)
    pub fn get_piece_debug_info(&self) -> String {
        if let Some(ref piece) = self.current_piece {
            format!("Piece: {:?} at ({}, {}) | Locking: {} | Lock Timer: {:.2}s | Resets: {} | Lifetime: {:.2}s | Can Fall: {}",
                   piece.piece_type,
                   piece.position.0, piece.position.1,
                   self.piece_is_locking,
                   self.lock_delay_timer,
                   self.lock_resets,
                   self.piece_lifetime_timer,
                   {
                       let mut test_piece = piece.clone();
                       test_piece.move_by(0, 1);
                       self.is_piece_valid(&test_piece)
                   }
            )
        } else {
            "No current piece".to_string()
        }
    }
    
    /// Update drop interval based on current level
    /// Uses a more reasonable progression that doesn't become microscopic
    fn update_drop_interval(&mut self) {
        let level = self.board.level();
        
        // Use a more reasonable drop speed progression
        // Each level increases speed but maintains playable intervals
        self.drop_interval = match level {
            1 => 1.0,      // 1 second (slow start)
            2 => 0.85,     // 850ms
            3 => 0.72,     // 720ms
            4 => 0.61,     // 610ms 
            5 => 0.52,     // 520ms
            6 => 0.44,     // 440ms
            7 => 0.37,     // 370ms
            8 => 0.31,     // 310ms
            9 => 0.26,     // 260ms
            10 => 0.22,    // 220ms
            11 => 0.19,    // 190ms
            12 => 0.16,    // 160ms
            13 => 0.13,    // 130ms
            14 => 0.11,    // 110ms
            15 => 0.09,    // 90ms
            _ => 0.08,     // 80ms minimum (very fast but still playable)
        };
        
        log::debug!("Updated drop interval for level {} to {:.3}s ({:.1}ms)", 
                   level, self.drop_interval, self.drop_interval * 1000.0);
    }
    
    /// Check if the current piece placement qualifies as a T-spin
    /// Basic T-spin detection: T-piece + last action was rotation + surrounded by blocks/walls
    pub fn is_t_spin(&self) -> bool {
        // Must have a current piece that is a T-piece
        if let Some(ref piece) = self.current_piece {
            if piece.piece_type != crate::tetromino::TetrominoType::T {
                return false;
            }
            
            // Last action must have been a rotation
            if !self.last_action_was_rotation {
                return false;
            }
            
            // Check if T-piece is in a "3-corner rule" position
            // For a proper T-spin, at least 3 corners around the T center should be occupied
            let center_x = piece.position.0;
            let center_y = piece.position.1;
            
            // Check the 4 corner positions around the T-piece center
            let corners = [
                (center_x - 1, center_y - 1), // Top-left
                (center_x + 1, center_y - 1), // Top-right  
                (center_x - 1, center_y + 1), // Bottom-left
                (center_x + 1, center_y + 1), // Bottom-right
            ];
            
            let occupied_corners = corners.iter()
                .filter(|(x, y)| {
                    // Consider position occupied if it's out of bounds or has a block
                    !self.board.is_position_valid(*x, *y) || 
                    self.board.get_cell(*x, *y).map_or(true, |cell| cell.is_filled())
                })
                .count();
            
            // T-spin if at least 3 corners are occupied
            occupied_corners >= 3
        } else {
            false
        }
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hold_piece_basic_functionality() {
        let mut game = Game::new();
        let original_piece_type = game.current_piece.as_ref().unwrap().piece_type;
        
        // Initially should be able to hold
        assert!(game.can_hold());
        assert!(game.held_piece.is_none());
        
        // Hold the current piece
        assert!(game.hold_piece());
        
        // Should now have a held piece and new current piece
        assert!(game.held_piece.is_some());
        assert_eq!(game.held_piece.unwrap(), original_piece_type);
        assert!(game.current_piece.is_some());
        
        // Should not be able to hold again for the same piece
        assert!(!game.can_hold());
        assert!(!game.hold_piece());
    }
    
    #[test]
    fn test_hold_piece_swap_functionality() {
        let mut game = Game::new();
        let first_piece_type = game.current_piece.as_ref().unwrap().piece_type;
        
        // Hold the first piece
        assert!(game.hold_piece());
        let _second_piece_type = game.current_piece.as_ref().unwrap().piece_type;
        
        // Spawn next piece to reset hold availability
        game.spawn_next_piece();
        
        // Now hold again - should swap
        let third_piece_type = game.current_piece.as_ref().unwrap().piece_type;
        assert!(game.can_hold());
        assert!(game.hold_piece());
        
        // The current piece should now be the first piece we held
        assert_eq!(game.current_piece.as_ref().unwrap().piece_type, first_piece_type);
        // The held piece should be the piece we just swapped out
        assert_eq!(game.held_piece.unwrap(), third_piece_type);
    }
    
    #[test]
    fn test_hold_availability_reset_on_spawn() {
        let mut game = Game::new();
        
        // Hold a piece
        assert!(game.hold_piece());
        assert!(!game.can_hold());
        
        // Spawn next piece should reset hold availability
        game.spawn_next_piece();
        assert!(game.can_hold());
    }
    
    #[test]
    fn test_cannot_hold_without_current_piece() {
        let mut game = Game::new();
        
        // Remove current piece
        game.current_piece = None;
        
        // Should not be able to hold
        assert!(!game.can_hold());
        assert!(!game.hold_piece());
    }
}
