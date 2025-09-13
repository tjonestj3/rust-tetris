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
        self.drop_timer += delta_time;
        
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
            
            // Clear any complete lines
            let complete_lines = self.board.find_complete_lines();
            if !complete_lines.is_empty() {
                let lines_cleared = self.board.clear_lines(&complete_lines);
                self.add_score_for_lines(lines_cleared);
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
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
