//! Game configuration constants and settings

/// Game board dimensions
pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;
pub const VISIBLE_HEIGHT: usize = 20;
pub const BUFFER_HEIGHT: usize = 4; // Extra rows above visible area for piece spawning

/// Rendering constants
pub const CELL_SIZE: f32 = 32.0;  // Slightly larger cells
pub const GRID_LINE_WIDTH: f32 = 1.5;
pub const BOARD_BORDER_WIDTH: f32 = 3.0;

/// Window dimensions (calculated to fit everything properly)
pub const WINDOW_WIDTH: i32 = 900;  // Extra width for UI elements
pub const WINDOW_HEIGHT: i32 = 750; // Height to fit: 100 (top) + 600 (board) + 50 (bottom)
pub const TARGET_FPS: i32 = 60;

/// Game timing (in seconds)
pub const INITIAL_DROP_TIME: f64 = 1.0; // 1 second per drop at level 1
pub const FAST_DROP_MULTIPLIER: f64 = 0.05; // Speed up factor for soft drop
pub const LOCK_DELAY: f64 = 0.3; // Time before piece locks in place
pub const MAX_LOCK_RESETS: u32 = 15; // Maximum number of times lock delay can be reset

/// Input timing (in seconds)
pub const INPUT_REPEAT_DELAY: f64 = 0.167; // Initial delay before key repeat
pub const INPUT_REPEAT_RATE: f64 = 0.033; // Time between repeated inputs
pub const SOFT_DROP_INTERVAL: f64 = 0.05; // Time between soft drop steps when held
pub const HORIZONTAL_MOVE_INTERVAL: f64 = 0.1; // Time between horizontal moves when held
pub const LINE_CLEAR_ANIMATION_TIME: f64 = 0.5; // Duration of line clearing animation
pub const TETRIS_CELEBRATION_TIME: f64 = 2.0; // Duration of TETRIS celebration message

/// Scoring constants
pub const SCORE_SINGLE_LINE: u32 = 100;
pub const SCORE_DOUBLE_LINE: u32 = 300;
pub const SCORE_TRIPLE_LINE: u32 = 500;
pub const SCORE_TETRIS: u32 = 800;
pub const SCORE_SOFT_DROP: u32 = 1;
pub const SCORE_HARD_DROP: u32 = 2;

/// Level progression
pub const LINES_PER_LEVEL: u32 = 10;
pub const LEVEL_SPEED_MULTIPLIER: f64 = 0.85; // Speed increase per level

/// UI Constants
pub const UI_MARGIN: f32 = 20.0;
pub const TEXT_SIZE: f32 = 24.0;
pub const TITLE_TEXT_SIZE: f32 = 32.0;

/// Game area calculations (computed from above constants)
pub const BOARD_WIDTH_PX: f32 = BOARD_WIDTH as f32 * CELL_SIZE;
pub const BOARD_HEIGHT_PX: f32 = VISIBLE_HEIGHT as f32 * CELL_SIZE;

/// Centered board positioning
pub const BOARD_OFFSET_X: f32 = (WINDOW_WIDTH as f32 - BOARD_WIDTH_PX) / 2.0;
pub const BOARD_OFFSET_Y: f32 = (WINDOW_HEIGHT as f32 - BOARD_HEIGHT_PX) / 2.0 + 20.0; // Slightly above center

/// Next piece preview area
pub const PREVIEW_OFFSET_X: f32 = BOARD_OFFSET_X + BOARD_WIDTH_PX + UI_MARGIN;
pub const PREVIEW_OFFSET_Y: f32 = BOARD_OFFSET_Y;
pub const PREVIEW_SIZE: f32 = 4.0 * CELL_SIZE;

/// Hold piece area
pub const HOLD_OFFSET_X: f32 = UI_MARGIN;
pub const HOLD_OFFSET_Y: f32 = BOARD_OFFSET_Y;
pub const HOLD_SIZE: f32 = 4.0 * CELL_SIZE;

/// Game window title
pub const WINDOW_TITLE: &str = "Rust Tetris";

/// Debug settings
pub const DEBUG_MODE: bool = cfg!(debug_assertions);
pub const SHOW_FPS: bool = DEBUG_MODE;