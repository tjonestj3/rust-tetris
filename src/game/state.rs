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
            ghost_blocks_available: 0,
            ghost_block_placement_mode: false,
            ghost_block_cursor: (BOARD_WIDTH as i32 / 2, (BUFFER_HEIGHT + VISIBLE_HEIGHT / 2) as i32),
            ghost_block_blink_timer: 0.0,
            ghost_smart_positions: Vec::new(),
            ghost_cursor_index: 0,
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
        self.ghost_block_blink_timer += delta_time;
        
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
        // This prevents ghost block placements from interrupting the current falling piece
        if self.current_piece.is_none() {
            self.spawn_next_piece();
        } else {
            // If we have a current piece, make sure it's still in a valid position after line clearing
            // If not valid, try to move it up until it is valid
            if let Some(mut piece) = self.current_piece.clone() {
                while !self.is_piece_valid(&piece) && piece.position.1 > 0 {
                    piece.move_by(0, -1);
                }
                
                // If piece is still not valid even after moving up, lock it and spawn new one
                if !self.is_piece_valid(&piece) {
                    self.current_piece = None;
                    self.spawn_next_piece();
                } else {
                    self.current_piece = Some(piece);
                }
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
    
    /// Toggle ghost block placement mode
    pub fn toggle_ghost_block_mode(&mut self) {
        if self.ghost_blocks_available > 0 {
            self.ghost_block_placement_mode = !self.ghost_block_placement_mode;
            if self.ghost_block_placement_mode {
                // Analyze board and find smart positions
                self.analyze_smart_positions();
                self.ghost_block_blink_timer = 0.0;
                log::info!("Ghost block placement mode activated with smart positioning");
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
    
    /// Place a ghost block at the current cursor position
    pub fn place_ghost_block(&mut self) -> bool {
        if self.ghost_block_placement_mode && self.ghost_blocks_available > 0 {
            let (x, y) = self.ghost_block_cursor;
            
            // Check if position is valid (empty)
            if let Some(cell) = self.board.get_cell(x, y) {
                if cell.is_empty() {
                    // Place the ghost block
                    self.board.set_cell(x, y, Cell::Filled(macroquad::prelude::Color::new(0.8, 0.8, 1.0, 1.0))); // Light blue ghost block
                    self.ghost_blocks_available -= 1;
                    self.ghost_block_placement_mode = false;
                    
                    // Check if this placement creates any complete lines
                    let complete_lines = self.board.find_complete_lines();
                    if !complete_lines.is_empty() {
                        self.start_line_clear_animation(complete_lines);
                    }
                    
                    log::info!("Ghost block placed at ({}, {}). Remaining: {}", x, y, self.ghost_blocks_available);
                    return true;
                }
            }
        }
        false
    }
    
    /// Check if ghost block cursor should be visible (blinking effect)
    pub fn is_ghost_cursor_visible(&self) -> bool {
        if !self.ghost_block_placement_mode {
            return false;
        }
        // Blink every 0.5 seconds
        (self.ghost_block_blink_timer % 1.0) < 0.5
    }
    
    /// Analyze board and find smart positions for ghost block placement
    pub fn analyze_smart_positions(&mut self) {
        let mut positions = Vec::new();
        
        // Check each empty position on the board
        for y in BUFFER_HEIGHT..(BOARD_HEIGHT + BUFFER_HEIGHT) {
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
        
        log::info!("Found {} smart positions for ghost block placement", self.ghost_smart_positions.len());
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
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}
