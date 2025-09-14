//! Game state management

use crate::board::{Board, Cell};
use crate::tetromino::{Tetromino, TetrominoType};
use crate::game::config::*;

/// Game states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
}

/// Main game struct
#[derive(Debug)]
pub struct Game {
    /// Current game state
    pub state: GameState,
    /// The game board
    pub board: Board,
    /// Currently falling piece
    pub current_piece: Option<Tetromino>,
    /// Next piece to spawn
    pub next_piece: TetrominoType,
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
}

impl Game {
    /// Create a new game instance
    pub fn new() -> Self {
        let mut game = Self {
            state: GameState::Playing,
            board: Board::new(),
            current_piece: None,
            next_piece: TetrominoType::random(),
            score: 0,
            drop_timer: 0.0,
            drop_interval: INITIAL_DROP_TIME,
            game_time: 0.0,
            clearing_lines: Vec::new(),
            clear_animation_timer: 0.0,
            soft_drop_timer: 0.0,
            left_move_timer: 0.0,
            right_move_timer: 0.0,
        };
        
        // Spawn the first piece
        game.spawn_next_piece();
        
        game
    }
    
    /// Update game logic
    pub fn update(&mut self, delta_time: f64) {
        if self.state != GameState::Playing {
            return;
        }
        
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
        
        // Check if it's time to drop the current piece
        if self.drop_timer >= self.drop_interval {
            self.drop_current_piece();
            self.drop_timer = 0.0;
        }
        
        // Update drop interval based on level
        let level_multiplier = LEVEL_SPEED_MULTIPLIER.powi((self.board.level() - 1) as i32);
        self.drop_interval = INITIAL_DROP_TIME * level_multiplier;
    }
    
    /// Try to drop the current piece by one row
    pub fn drop_current_piece(&mut self) -> bool {
        if let Some(mut piece) = self.current_piece.clone() {
            // Try to move down
            piece.move_by(0, 1);
            
            if self.is_piece_valid(&piece) {
                // Successfully moved down
                self.current_piece = Some(piece);
                return true;
            } else {
                // Can't move down, lock the piece
                self.lock_current_piece();
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
        self.next_piece = TetrominoType::random();
        
        // Check if the new piece can be placed
        if self.is_piece_valid(&new_piece) {
            self.current_piece = Some(new_piece);
        } else {
            // Game over - can't spawn new piece
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
                self.current_piece = Some(piece);
                return true;
            }
        }
        false
    }
    
    /// Try to rotate the current piece clockwise
    pub fn rotate_piece_clockwise(&mut self) -> bool {
        if let Some(mut piece) = self.current_piece.clone() {
            piece.rotate_clockwise();
            
            if self.is_piece_valid(&piece) {
                self.current_piece = Some(piece);
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
            self.clearing_lines.clear();
            self.clear_animation_timer = 0.0;
        }
        
        // Check game over after clearing lines
        if self.board.is_game_over() {
            self.state = GameState::GameOver;
            return;
        }
        
        // Spawn next piece
        self.spawn_next_piece();
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
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
