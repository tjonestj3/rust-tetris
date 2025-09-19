# 🎮 Learning Rust Through Tetris: A Comprehensive Study Guide

*Master Rust programming concepts by exploring a real-world game implementation*

---

## 📋 Table of Contents

1. [Introduction](#introduction)
2. [Start Here: Rust Basics With Your Tetris Game](#-start-here-rust-basics-with-your-tetris-game)
3. [Project Overview](#project-overview)
4. [Current Feature Map](#-current-feature-map)
5. [Core Rust Concepts Demonstrated](#core-rust-concepts-demonstrated)
6. [Project Architecture Deep Dive](#project-architecture-deep-dive)
7. [Ownership and Borrowing in Action](#ownership-and-borrowing-in-action)
8. [Error Handling Patterns](#error-handling-patterns)
9. [Trait System Usage](#trait-system-usage)
10. [Pattern Matching Excellence](#pattern-matching-excellence)
11. [Memory Safety and Performance](#memory-safety-and-performance)
12. [Advanced Rust Features](#advanced-rust-features)
13. [Game Logic Implementation](#game-logic-implementation)
14. [Step-by-step Labs and Exercises](#-step-by-step-labs-and-exercises)
15. [Next Steps](#next-steps)

---

## 🎯 Introduction

Welcome to an innovative approach to learning Rust! This guide uses your Tetris game implementation as a comprehensive teaching tool to demonstrate real-world Rust programming concepts. Instead of abstract examples, you'll see how Rust's features solve actual problems in game development.

### Why This Approach Works

- **Real-world context**: Every concept is demonstrated in practical use
- **Progressive complexity**: From basic syntax to advanced patterns
- **Performance-focused**: See how Rust's zero-cost abstractions work
- **Safety-first**: Understand how Rust prevents common bugs

---

## 🌱 Start Here: Rust Fundamentals Through Your Tetris Game

### Prerequisites and Setup

**Running Your Game:**
```bash
# Debug build (faster compile times, slower runtime)
cargo run

# Release build (slower compile, optimized performance - use for actual play)
cargo run --release

# Code formatting and linting (recommended workflow)
cargo fmt      # Format code to Rust standards
cargo clippy   # Catch common mistakes and suggest improvements
```

**Essential File Map:**
- **Entry Point:** `src/main.rs` - Application lifecycle and main game loop
- **Game Core:** `src/game/state.rs` - Game state, update logic, piece management
- **Configuration:** `src/game/config.rs` - Constants, timing, and game parameters
- **Game Board:** `src/board/board.rs` - 2D grid, line clearing, collision detection
- **Game Pieces:** `src/tetromino/types.rs` + `src/tetromino/data.rs` - Piece definitions and shape data
- **Rotation System:** `src/rotation/srs.rs` - Super Rotation System with wall kicks
- **User Interface:** `src/menu/mod.rs` - Menus, settings, high scores
- **Audio System:** `src/audio/system.rs` - Sound effects and music management

### Core Rust Concepts (With Your Code)

#### 1. Variables and Mutability

Rust variables are **immutable by default** - you must explicitly opt into mutability:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/main.rs start=73
// From your main.rs - notice the explicit mut keywords
let mut frame_count = 0u64;              // Mutable counter - can be changed
let mut last_fps_time = get_time();      // Mutable timestamp - updates each frame
let mut fps = 0.0;                       // Mutable FPS calculation result
let auto_save_interval = 30.0;           // Immutable constant - never changes
let mut last_game_state_hash = 0u64;     // Mutable for state change detection
```

**Key Learning Points:**
- `mut` must be explicitly declared - prevents accidental modifications
- Type annotations (like `0u64`) are optional when Rust can infer them
- Immutable by default encourages functional programming patterns
- The compiler will error if you try to modify an immutable variable

#### 2. Functions vs Methods

**Standalone Functions:**
```rust path=/home/xenocide/rust-projects/rust-tetris/src/main.rs start=22
/// Window configuration for macroquad - standalone function
fn window_conf() -> Conf {
    Conf {
        window_title: WINDOW_TITLE.to_owned(),    // Convert &str to String
        window_width: WINDOW_WIDTH,               // Copy integer value
        window_height: WINDOW_HEIGHT,
        window_resizable: false,                  // Boolean literal
        high_dpi: false,
        ..Default::default()                      // Use default values for other fields
    }
}
```

**Methods (associated with structs):**
```rust path=/home/xenocide/rust-projects/rust-tetris/src/game/state.rs start=165
// From Game struct - methods operate on struct instances
impl Game {
    /// Update game logic - takes mutable reference to self
    pub fn update(&mut self, delta_time: f64) {
        // &mut self allows us to modify game state fields
        if self.state != GameState::Playing {
            return;  // Early return for non-playing states
        }
        
        self.piece_just_locked = false;    // Modify struct field
        self.game_time += delta_time;       // Accumulate time
        // ... more mutations ...
    }
    
    /// Get current level - takes immutable reference to self
    pub fn level(&self) -> u32 {
        // &self is read-only - cannot modify game state
        self.board.level()  // Delegate to board's level method
    }
}
```

**Key Learning Points:**
- `&mut self` = "borrow this struct mutably" (can read and write fields)
- `&self` = "borrow this struct immutably" (can only read fields)
- `self` = "take ownership of this struct" (rarely used, consumes the value)
- Methods provide namespaced, type-safe operations on data

#### 3. Enums and Pattern Matching

**Enum Definition:**
```rust path=/home/xenocide/rust-projects/rust-tetris/src/game/state.rs start=14
/// Game states - exhaustive enum prevents invalid states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameState {
    Menu,        // In the start menu
    Playing,     // Actively playing the game
    Paused,      // Game is paused
    GameOver,    // Game has ended
}
```

**Pattern Matching:**
```rust path=/home/xenocide/rust-projects/rust-tetris/src/main.rs start=97
// From your main application loop - exhaustive matching
match app_state {
    AppState::Menu => {
        // Update menu system
        menu_system.update(delta_time as f64);
        
        // Handle menu input and get the user's choice
        let action = menu_system.handle_input();
        
        match action {
            MenuAction::NewGame => {
                log::info!("Starting new game");
                game = Some(Game::new());      // Create new game instance
                app_state = AppState::Playing; // Transition to playing state
            },
            MenuAction::LoadGame => {
                // Attempt to load saved game with error handling
                match Game::load_from_file(&save_path) {
                    Ok(loaded_game) => {
                        game = Some(loaded_game);
                        app_state = AppState::Playing;
                    },
                    Err(e) => {
                        log::warn!("Failed to load save file: {}", e);
                        // Graceful fallback to new game
                        game = Some(Game::new());
                        app_state = AppState::Playing;
                    }
                }
            },
            MenuAction::Quit => std::process::exit(0),
            MenuAction::None => {}, // Continue in menu
        }
        
        // Render the menu
        menu_system.render(&background_texture);
    },
    AppState::Playing => {
        // Game playing logic here...
    },
    AppState::GameOver => {
        // Game over handling here...
    }
}
```

**Key Learning Points:**
- Enums represent "one of several possible values" (sum types)
- Pattern matching is **exhaustive** - compiler ensures all cases are handled
- `match` arms can have complex patterns and guards
- Nested matching allows handling complex control flow safely

#### 4. Structs and Associated Functions

**Struct Definition:**
```rust path=/home/xenocide/rust-projects/rust-tetris/src/tetromino/types.rs start=63
/// Represents a tetromino piece in the game
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tetromino {
    /// The type of tetromino (I, O, T, S, Z, J, L)
    pub piece_type: TetrominoType,
    /// Current position (x, y) of the piece center
    pub position: (i32, i32),
    /// Current rotation state (0-3 representing 0°, 90°, 180°, 270°)
    pub rotation: u8,
    /// The blocks that make up this piece (relative to center position)
    pub blocks: Vec<(i32, i32)>,
}
```

**Implementation Block with Methods:**
```rust path=/home/xenocide/rust-projects/rust-tetris/src/tetromino/types.rs start=75
impl Tetromino {
    /// Create a new tetromino at the spawn position
    pub fn new(piece_type: TetrominoType) -> Self {
        let mut tetromino = Self {
            piece_type,                          // Shorthand for piece_type: piece_type
            position: (4, 2),                    // Start in middle of board, slightly down
            rotation: 0,                         // Start with no rotation
            blocks: Vec::new(),                  // Empty vector, will be populated
        };
        tetromino.update_blocks();               // Calculate block positions
        tetromino                                // Return the new tetromino
    }
    
    /// Move the tetromino by the specified offset
    pub fn move_by(&mut self, dx: i32, dy: i32) {
        self.position.0 += dx;                   // Modify x coordinate
        self.position.1 += dy;                   // Modify y coordinate
        // Note: blocks are relative to position, so no need to update them
    }
    
    /// Get the absolute positions of all blocks in world coordinates
    pub fn absolute_blocks(&self) -> Vec<(i32, i32)> {
        self.blocks.iter()                       // Create iterator over block positions
            .map(|(dx, dy)| (                    // Transform each relative position
                self.position.0 + dx,            // Add piece position to relative x
                self.position.1 + dy             // Add piece position to relative y
            ))
            .collect()                           // Collect results into Vec
    }
}
```

**Key Learning Points:**
- `#[derive(...)]` automatically implements common traits
- `Self` refers to the struct type being implemented
- Methods can take `self`, `&self`, or `&mut self` depending on ownership needs
- Iterator chains like `.iter().map().collect()` are zero-cost abstractions

#### 5. Ownership and Option<T>

**Option for Safe Null Handling:**
```rust path=/home/xenocide/rust-projects/rust-tetris/src/game/state.rs start=273
/// Lock the current piece to the board and spawn a new one
pub fn lock_current_piece(&mut self) {
    // .take() moves the value out of Option, replacing it with None
    if let Some(piece) = self.current_piece.take() {
        // 'piece' is now owned by this scope, not borrowed
        
        log::debug!("Locking piece {:?} at position ({}, {})",
                   piece.piece_type, piece.position.0, piece.position.1);
        
        // Set flag for audio feedback
        self.piece_just_locked = true;
        
        // Place the piece on the board - consuming ownership
        for (x, y) in piece.absolute_blocks() {
            if x >= 0 && y >= 0 {
                self.board.set_cell(x, y, Cell::Filled(piece.color()));
            }
        }
        
        // Check for line clears
        let complete_lines = self.board.find_complete_lines();
        if !complete_lines.is_empty() {
            self.start_line_clear_animation(complete_lines);
            return; // Don't spawn next piece during animation
        }
        
        // Spawn the next piece
        self.spawn_next_piece();
    }
    // If current_piece was None, this function does nothing
}
```

**Borrowing for Validation:**
```rust path=/home/xenocide/rust-projects/rust-tetris/src/game/state.rs start=263
/// Check if the current piece is in a valid position
pub fn is_piece_valid(&self, piece: &Tetromino) -> bool {
    // Borrow the piece immutably - we don't need to own it
    for (x, y) in piece.absolute_blocks() {
        // Check each block position for validity
        if !self.board.is_position_valid(x, y) {
            return false;  // Early return if any block is invalid
        }
    }
    true  // All blocks are in valid positions
}
```

**Key Learning Points:**
- `Option<T>` replaces null pointers - either `Some(value)` or `None`
- `.take()` moves a value out of Option, replacing it with None
- Borrowing (`&T`) allows reading without taking ownership
- `if let Some(value) = option` is idiomatic for extracting Option values

#### 6. Error Handling with Result<T, E>

**File Operations with Error Handling:**
```rust path=/home/xenocide/rust-projects/rust-tetris/src/game/state.rs start=672
/// Save the game state to a file
pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
    // Convert the game state to JSON - may fail
    let json = serde_json::to_string_pretty(self)?;  // ? propagates errors
    
    // Write to file - may also fail
    fs::write(path, json)?;                          // ? propagates errors
    
    log::info!("Game saved successfully");
    Ok(())  // Return success value
}

/// Load the game state from a file  
pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
    // Read file contents - may fail if file doesn't exist
    let json = fs::read_to_string(path)?;
    
    // Parse JSON - may fail if format is invalid
    let game: Game = serde_json::from_str(&json)?;
    
    log::info!("Game loaded successfully");
    Ok(game)  // Return the loaded game
}
```

**Graceful Error Handling in Audio System:**
```rust path=/home/xenocide/rust-projects/rust-tetris/src/main.rs start=52
// Initialize and load audio system
let mut audio_system = AudioSystem::new();
if let Err(e) = audio_system.load_sounds().await {
    // Don't crash the game if audio fails - just log a warning
    log::warn!("Failed to initialize audio system: {}", e);
    // Game continues without audio
}
```

**Key Learning Points:**
- `Result<T, E>` represents either success `Ok(T)` or failure `Err(E)`
- `?` operator propagates errors up the call stack automatically
- `Box<dyn std::error::Error>` accepts any error type (trait object)
- Graceful degradation allows programs to continue despite partial failures

#### 7. Modules and Visibility

**Module Declaration:**
```rust path=/home/xenocide/rust-projects/rust-tetris/src/lib.rs start=1
//! Rust Tetris Game Library
//! 
//! A high-performance Tetris implementation focusing on smooth 60fps gameplay,
//! clean architecture, and extensible design.

// Declare public modules - these can be used by external code
pub mod audio;      // Audio system for sound effects and music
pub mod board;      // Game board and cell management
pub mod game;       // Core game state and logic
pub mod graphics;   // Colors and visual constants
pub mod input;      // Input handling (placeholder)
pub mod tetromino;  // Tetromino pieces and movement

// Additional modules for extended functionality
pub mod leaderboard;  // High score tracking
pub mod menu;         // Menu system and settings
pub mod rotation;     // SRS rotation system
pub mod scoring;      // Advanced scoring with combos and T-spins

// Re-export commonly used items for convenience
pub use game::Game;           // Main game struct
pub use board::Board;         // Game board
pub use menu::MenuSystem;     // Menu system
```

**Module Usage:**
```rust path=/home/xenocide/rust-projects/rust-tetris/src/main.rs start=1
// Import specific items from modules
use rust_tetris::game::config::*;                    // Import all constants
use rust_tetris::graphics::colors::*;                // Import all colors
use rust_tetris::game::{Game, GameState};            // Import specific types
use rust_tetris::audio::system::{AudioSystem, SoundType};
use rust_tetris::{MenuSystem, MenuAction};           // Use re-exports from lib.rs
```

**Key Learning Points:**
- Modules organize code into logical units
- `pub mod` makes modules accessible to external code
- `pub use` creates convenient re-exports
- `use` statements bring items into scope
- `::` is the path separator for nested modules

### 🗺️ Current Architecture Overview

Your Tetris game demonstrates several advanced Rust patterns:

**State Management:**
- App-level state machine (`AppState`: Menu, Playing, GameOver)
- Game-level state machine (`GameState`: Playing, Paused, GameOver) 
- Complex state transitions with validation

**Memory Management:**
- Stack-allocated 2D arrays for the game board (cache-friendly)
- Owned vs borrowed data with clear ownership patterns
- Option<T> for nullable game pieces

**Error Resilience:**
- Graceful degradation (game works without audio)
- File I/O with proper error handling
- State persistence with rollback on failure

**Performance Features:**
- Zero-cost abstractions (iterators compile to loops)
- Compile-time constants for game parameters
- Efficient data structures (arrays vs vectors where appropriate)

**Modern Rust Features:**
- Async/await for resource loading
- Trait derivation for common functionality
- Pattern matching for complex control flow
- Module system for code organization

---

## 🏗️ Project Overview

### Project Structure

Your Tetris implementation is organized into logical modules that demonstrate Rust's module system:

```rust
src/
├── main.rs              // Entry point and game loop
├── lib.rs               // Library exports
├── game/
│   ├── mod.rs          // Module declarations
│   ├── state.rs        // Game state management
│   └── config.rs       // Constants and configuration
├── board/
│   ├── mod.rs          // Board module exports
│   └── board.rs        // Core game board logic
├── tetromino/
│   ├── mod.rs          // Tetromino module exports
│   ├── types.rs        // Piece definitions
│   └── data.rs         // Shape data
├── graphics/
│   ├── mod.rs          // Graphics module exports
│   └── colors.rs       // Color definitions
├── audio/
│   ├── mod.rs          // Audio module exports
│   └── system.rs       // Audio system
└── input/
    ├── mod.rs          // Input module exports
    └── handler.rs      // Input handling (placeholder)
```

### Key Dependencies

```toml
[dependencies]
# Graphics and rendering - demonstrates external crate usage
macroquad = { version = "0.4", features = ["audio"] }

# Serialization - shows trait derivation and generic programming
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Random number generation - demonstrates external API integration
rand = "0.8"

# Configuration management
config = "0.13"

# Logging - shows structured error handling
log = "0.4"
env_logger = "0.10"
```

---

## 🔍 Core Rust Concepts Demonstrated

### 1. **Ownership System**

The project brilliantly demonstrates Rust's ownership system:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/game/state.rs start=274
/// Spawn the next piece
pub fn spawn_next_piece(&mut self) {
    let new_piece = Tetromino::new(self.next_piece);
    self.next_piece = TetrominoType::random();
    
    // Reset hold usage for the new piece
    self.hold_used_this_piece = false;
    
    // Reset lock delay state for new piece
    self.piece_is_locking = false;
    self.lock_delay_timer = 0.0;
    self.lock_resets = 0;
    
    // Check if the new piece can be placed
    if self.is_piece_valid(&new_piece) {
        self.current_piece = Some(new_piece);
    } else {
        // Game over - can't spawn new piece
        self.state = GameState::GameOver;
    }
}
```

**Key Learning Points:**
- `new_piece` is **owned** by the function
- When we assign `Some(new_piece)`, ownership **moves** to `self.current_piece`
- No explicit memory management needed - Rust handles cleanup automatically
- `&new_piece` creates a **borrow** for validation without moving ownership

### 2. **Borrowing and References**

Smart use of references throughout the codebase:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/game/state.rs start=227
/// Check if the current piece is in a valid position
pub fn is_piece_valid(&self, piece: &Tetromino) -> bool {
    for (x, y) in piece.absolute_blocks() {
        if !self.board.is_position_valid(x, y) {
            return false;
        }
    }
    true
}
```

**Key Learning Points:**
- `&self` - immutable borrow of the game state
- `&Tetromino` - we don't need to own the piece, just inspect it
- The function can't modify the game state or the piece
- Prevents accidental mutation while checking validity

### 3. **Enums and Pattern Matching**

Rust's enums are powerful sum types, demonstrated perfectly:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/tetromino/types.rs start=9
/// Seven standard Tetris pieces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TetrominoType {
    I, // Line piece (cyan)
    O, // Square piece (yellow)
    T, // T-piece (purple)
    S, // S-piece (green)
    Z, // Z-piece (red)
    J, // J-piece (blue)
    L, // L-piece (orange)
}
```

And sophisticated pattern matching:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/tetromino/types.rs start=36
/// Get the color associated with this tetromino type
pub fn color(self) -> Color {
    match self {
        TetrominoType::I => TETROMINO_I,
        TetrominoType::O => TETROMINO_O,
        TetrominoType::T => TETROMINO_T,
        TetrominoType::S => TETROMINO_S,
        TetrominoType::Z => TETROMINO_Z,
        TetrominoType::J => TETROMINO_J,
        TetrominoType::L => TETROMINO_L,
    }
}
```

**Key Learning Points:**
- Enums can contain data or be simple variants
- Pattern matching is **exhaustive** - compiler ensures all cases are handled
- `#[derive(...)]` automatically implements common traits
- Copy types can be passed by value efficiently

### 4. **Structs and Methods**

Rich struct definitions with associated methods:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/tetromino/types.rs start=63
/// Represents a tetromino piece in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tetromino {
    /// The type of tetromino
    pub piece_type: TetrominoType,
    /// Current position (x, y) of the piece center
    pub position: (i32, i32),
    /// Current rotation state (0-3)
    pub rotation: u8,
    /// The blocks that make up this piece (relative to position)
    pub blocks: Vec<(i32, i32)>,
}
```

With methods that demonstrate different types of self:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/tetromino/types.rs start=105
/// Move the tetromino by the specified offset
pub fn move_by(&mut self, dx: i32, dy: i32) {
    self.position.0 += dx;
    self.position.1 += dy;
}

/// Rotate the tetromino clockwise
pub fn rotate_clockwise(&mut self) {
    self.rotation = (self.rotation + 1) % 4;
    self.update_blocks();
}

/// Get the color of this tetromino
pub fn color(&self) -> Color {
    self.piece_type.color()
}
```

**Key Learning Points:**
- `&mut self` - mutable borrow for methods that modify
- `&self` - immutable borrow for read-only methods
- `self` - takes ownership (used sparingly)
- Methods are namespaced and provide clean APIs

---

## 🏛️ Project Architecture Deep Dive

### Module System

The project demonstrates Rust's module system excellently:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/lib.rs start=1
//! Rust Tetris Game Library
//! 
//! A high-performance Tetris implementation focusing on smooth 60fps gameplay,
//! clean architecture, and extensible design.

pub mod audio;
pub mod board;
pub mod game;
pub mod graphics;
pub mod input;
pub mod tetromino;

// Re-export commonly used items
pub use game::Game;
pub use board::Board;
```

**Key Learning Points:**
- `pub mod` declares public modules
- `pub use` re-exports items for convenience
- Documentation comments (`//!` and `///`) are first-class citizens
- Library vs binary crate structure

### Constants and Configuration

Centralized configuration demonstrates good Rust practices:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/game/config.rs start=1
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
```

**Key Learning Points:**
- `const` values are inlined at compile time
- Type annotations ensure correctness
- Grouping related constants in modules
- Using meaningful names and comments

---

## 🔒 Ownership and Borrowing in Action

### Game State Management

The game state demonstrates sophisticated ownership patterns:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/game/state.rs start=204
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
```

**Key Learning Points:**
- `Option<T>` provides safe null handling
- `clone()` creates owned copies when needed
- Pattern matching with `if let Some(...)`
- Clear ownership transfer with assignment

### Board Memory Management

The board demonstrates zero-cost abstractions:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/board/board.rs start=78
/// Get the cell at the specified position
/// Returns None if coordinates are out of bounds
pub fn get_cell(&self, x: i32, y: i32) -> Option<Cell> {
    if x < 0 || y < 0 {
        return None;
    }
    
    let x = x as usize;
    let y = y as usize;
    
    if x >= BOARD_WIDTH || y >= (BOARD_HEIGHT + BUFFER_HEIGHT) {
        return None;
    }
    
    Some(self.grid[y][x])
}
```

**Key Learning Points:**
- Bounds checking prevents undefined behavior
- `Option<T>` encodes possibility of failure in the type system
- Array access is bounds-checked at runtime
- Type casting with `as` when bounds are verified

---

## ⚠️ Error Handling Patterns

### Robust Audio Loading

The audio system shows real-world error handling:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/audio/system.rs start=58
/// Load all game sounds asynchronously
pub async fn load_sounds(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Loading game audio assets...");
    
    // Sound file mappings
    let sound_files = [
        (SoundType::UiClick, "assets/sounds/ui-click.wav"),
        (SoundType::PieceSnap, "assets/sounds/piece-snap.wav"),
        (SoundType::HardDrop, "assets/sounds/hard-drop.wav"),
        (SoundType::HoldPiece, "assets/sounds/hold-piece.wav"),
        (SoundType::LineClear, "assets/sounds/line-clear.wav"),
        (SoundType::LevelComplete, "assets/sounds/level-complete.wav"),
        (SoundType::Pause, "assets/sounds/pause.wav"),
        (SoundType::GameOver, "assets/sounds/game-over.wav"),
        (SoundType::PowerAction, "assets/sounds/place-ghost-block.wav"),
        (SoundType::BackgroundMusic, "assets/sounds/tetris-background-music.wav"),
    ];
    
    for (sound_type, file_path) in sound_files {
        match load_sound(file_path).await {
            Ok(sound) => {
                self.sounds.insert(sound_type, sound);
                log::debug!("Loaded sound: {:?} from {}", sound_type, file_path);
            }
            Err(e) => {
                log::warn!("Failed to load sound {:?} from {}: {} - continuing without this sound", sound_type, file_path, e);
                // Continue loading other sounds even if one fails
            }
        }
    }
    
    log::info!("Audio system initialized with {} sounds loaded", self.sounds.len());
    Ok(())
}
```

**Key Learning Points:**
- `Result<T, E>` type for error handling
- `Box<dyn std::error::Error>` for trait object error handling
- Graceful degradation - continue on partial failures
- Structured logging with different levels

### Safe Array Access

The board provides multiple safety mechanisms:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/board/board.rs start=113
/// Check if a position is valid and empty
pub fn is_position_valid(&self, x: i32, y: i32) -> bool {
    // Check bounds
    if x < 0 || x >= BOARD_WIDTH as i32 {
        return false;
    }
    
    // Allow pieces to spawn above the visible area
    if y < 0 {
        return true;
    }
    
    if y >= (BOARD_HEIGHT + BUFFER_HEIGHT) as i32 {
        return false;
    }
    
    // Check if cell is empty
    match self.get_cell(x, y) {
        Some(Cell::Empty) => true,
        _ => false,
    }
}
```

**Key Learning Points:**
- Multiple validation layers prevent crashes
- Early returns for clarity
- Pattern matching handles all cases
- Game-specific logic (spawn area) encoded safely

---

## 🎭 Trait System Usage

### Automatic Trait Derivation

The project makes extensive use of derived traits:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/board/board.rs start=28
/// Represents a single cell on the game board
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Cell {
    /// Empty cell
    Empty,
    /// Filled cell with a specific color
    Filled(#[serde(with = "color_serde")] Color),
}
```

**Key Learning Points:**
- `Debug` enables `{:?}` formatting
- `Clone` provides explicit copying
- `Copy` enables implicit copying for simple types
- `PartialEq` enables `==` comparisons
- `Serialize/Deserialize` for data persistence

### Custom Serialization

Advanced trait usage with custom serialization:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/board/board.rs start=8
// Custom serialization module for macroquad Color
mod color_serde {
    use super::*;
    use serde::{Serializer, Deserializer, Deserialize};
    
    pub fn serialize<S>(color: &Color, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (color.r, color.g, color.b, color.a).serialize(serializer)
    }
    
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color, D::Error>
    where
        D: Deserializer<'de>,
    {
        let (r, g, b, a) = <(f32, f32, f32, f32)>::deserialize(deserializer)?;
        Ok(Color::new(r, g, b, a))
    }
}
```

**Key Learning Points:**
- Generic functions with trait bounds
- Lifetime parameters (`'de`)
- Error propagation with `?` operator
- Module-level helper functions

### Default Implementation

Convenient defaults throughout:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/tetromino/types.rs start=149
impl Default for Tetromino {
    fn default() -> Self {
        Self::new(TetrominoType::T)
    }
}
```

**Key Learning Points:**
- `Default` trait provides sensible starting values
- Used by containers and initialization functions
- Can be derived for simple cases

---

## 🎲 Pattern Matching Excellence

### Complex Game Logic

Sophisticated pattern matching in game updates:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/main.rs start=176
// Draw game state overlays
match game.state {
    GameState::GameOver => draw_game_over_overlay(&game),
    GameState::Paused => draw_pause_overlay(&game),
    _ => {}, // No overlay for Playing or Menu
}
```

### Tetromino Shape Definition

Pattern matching with computed values:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/tetromino/data.rs start=22
/// I-piece (line) - 4 blocks in a line
fn get_i_piece_blocks(rotation: u8) -> Vec<(i32, i32)> {
    match rotation {
        0 | 2 => vec![(-1, 0), (0, 0), (1, 0), (2, 0)], // Horizontal
        1 | 3 => vec![(0, -1), (0, 0), (0, 1), (0, 2)], // Vertical
        _ => vec![],
    }
}
```

**Key Learning Points:**
- Multiple pattern matching with `|`
- Catch-all patterns with `_`
- Return different types based on patterns
- Compile-time exhaustiveness checking

### Audio System Pattern Matching

Enum-driven behavior selection:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/audio/system.rs start=94
/// Play a sound effect
pub fn play_sound(&self, sound_type: SoundType) {
    if !self.audio_enabled {
        return;
    }
    
    if let Some(sound) = self.sounds.get(&sound_type) {
        let volume = match sound_type {
            SoundType::BackgroundMusic => self.master_volume * self.music_volume,
            _ => self.master_volume * self.sfx_volume,
        };
        
        let params = PlaySoundParams {
            looped: sound_type == SoundType::BackgroundMusic,
            volume,
        };
        
        play_sound(sound, params);
        log::trace!("Playing sound: {:?} at volume {:.2}", sound_type, volume);
    } else {
        log::warn!("Sound not loaded: {:?}", sound_type);
    }
}
```

**Key Learning Points:**
- `if let` for optional pattern matching
- HashMap lookups return `Option<T>`
- Different behavior based on enum variants
- Early returns for validation

---

## 🚀 Memory Safety and Performance

### Zero-Cost Abstractions

The game demonstrates Rust's zero-cost abstractions:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/tetromino/types.rs start=98
/// Get the absolute positions of all blocks
pub fn absolute_blocks(&self) -> Vec<(i32, i32)> {
    self.blocks.iter()
        .map(|(dx, dy)| (self.position.0 + dx, self.position.1 + dy))
        .collect()
}
```

**Key Learning Points:**
- Iterator chains compile to efficient loops
- No runtime overhead from abstraction
- Lazy evaluation until `.collect()`
- Functional programming style without garbage collection

### Efficient Board Operations

Smart data structure design:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/board/board.rs start=168
/// Clear the specified lines and drop rows above
pub fn clear_lines(&mut self, lines_to_clear: &[usize]) -> u32 {
    if lines_to_clear.is_empty() {
        return 0;
    }
    
    let lines_cleared_count = lines_to_clear.len() as u32;
    
    // Sort lines in ascending order
    let mut sorted_lines = lines_to_clear.to_vec();
    sorted_lines.sort();
    
    // Create a new grid by copying non-cleared lines
    let mut new_grid = [[Cell::Empty; BOARD_WIDTH]; BOARD_HEIGHT + BUFFER_HEIGHT];
    let mut new_y = (BOARD_HEIGHT + BUFFER_HEIGHT) - 1; // Start from bottom
    
    // Copy lines from bottom to top, skipping cleared lines
    for y in (0..(BOARD_HEIGHT + BUFFER_HEIGHT)).rev() {
        if !sorted_lines.contains(&y) {
            // This line is not being cleared, copy it
            new_grid[new_y] = self.grid[y];
            if new_y > 0 {
                new_y -= 1;
            }
        }
        // If this line is being cleared, skip it (don't copy)
    }
    
    // Replace the old grid with the new one
    self.grid = new_grid;
    
    // Update statistics
    self.lines_cleared += lines_cleared_count;
    self.level = (self.lines_cleared / LINES_PER_LEVEL) + 1;
    
    lines_cleared_count
}
```

**Key Learning Points:**
- Stack-allocated arrays for performance
- Efficient copying with simple assignment
- No dynamic allocation during gameplay
- Clear algorithm implementation

---

## 🔧 Advanced Rust Features

### Serialization and Persistence

Complex serialization with custom handling:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/board/board.rs start=57
/// The main Tetris game board
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Board {
    /// The game grid - includes buffer rows above visible area
    grid: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT + BUFFER_HEIGHT],
    /// Lines cleared this game
    lines_cleared: u32,
    /// Current level
    level: u32,
}
```

### Async Programming

Asynchronous code for resource loading:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/main.rs start=21
#[macroquad::main(window_conf)]
async fn main() {
    // Initialize logging
    env_logger::init();
    log::info!("Starting Rust Tetris v{}", env!("CARGO_PKG_VERSION"));
    
    // Log layout calculations for debugging
    log::info!("Window: {}x{}", WINDOW_WIDTH, WINDOW_HEIGHT);
    log::info!("Board: {}x{} cells = {}x{} pixels", BOARD_WIDTH, VISIBLE_HEIGHT, BOARD_WIDTH_PX, BOARD_HEIGHT_PX);
    log::info!("Board position: ({}, {})", BOARD_OFFSET_X, BOARD_OFFSET_Y);
    log::info!("Required height: {} + {} = {}", BOARD_OFFSET_Y, BOARD_HEIGHT_PX, BOARD_OFFSET_Y + BOARD_HEIGHT_PX);

    // Load background texture
    let background_texture = Texture2D::from_image(&create_chess_background());
    
    // Initialize and load audio system
    let mut audio_system = AudioSystem::new();
    if let Err(e) = audio_system.load_sounds().await {
        log::warn!("Failed to initialize audio system: {}", e);
    }
```

**Key Learning Points:**
- `async fn` for asynchronous functions
- `.await` for waiting on futures
- Error handling with async code
- Macros for compile-time information

### Generic Collections

Smart use of standard library collections:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/audio/system.rs start=32
/// Audio system managing all game sounds
#[derive(Debug)]
pub struct AudioSystem {
    /// Loaded sound effects
    sounds: HashMap<SoundType, Sound>,
    /// Master volume (0.0 to 1.0)
    master_volume: f32,
    /// Sound effects volume (0.0 to 1.0)
    sfx_volume: f32,
    /// Music volume (0.0 to 1.0)
    music_volume: f32,
    /// Whether audio is enabled
    audio_enabled: bool,
}
```

**Key Learning Points:**
- `HashMap<K, V>` for key-value storage
- Enums as hash keys
- Logical grouping of related data
- `Debug` trait for debugging output

---

## 🎮 Game Logic Implementation

### Game Loop Architecture

The main game loop demonstrates real-time programming:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/main.rs start=65
// Main game loop
loop {
    let delta_time = get_frame_time();
    frame_count += 1;

    // Calculate FPS
    let current_time = get_time();
    if current_time - last_fps_time >= 1.0 {
        fps = frame_count as f64 / (current_time - last_fps_time);
        frame_count = 0;
        last_fps_time = current_time;
    }

    // Handle input
    handle_input(&mut game, &audio_system);
    
    // Store previous state for audio event detection
    let prev_score = game.score;
    let prev_level = game.level();
    let prev_lines_cleared = game.lines_cleared();
    let was_clearing_lines = game.is_clearing_lines();
    let prev_state = game.state;
    
    // Update game logic
    game.update(delta_time as f64);
    
    // Detect and play audio for game events
    detect_and_play_audio_events(&game, &audio_system, prev_score, prev_level, prev_lines_cleared, was_clearing_lines, prev_state);
    
    // Auto-save periodically during gameplay (optimized with state change detection)
    if game.state == GameState::Playing && current_time - last_save_time >= auto_save_interval {
        let current_hash = game.get_state_hash();
        if current_hash != last_game_state_hash {
            // Only save if game state has actually changed
            if let Err(e) = game.save_to_file(&save_path) {
                log::warn!("Auto-save failed: {}", e);
            } else {
                last_game_state_hash = current_hash;
                log::debug!("Auto-save completed (state changed)");
            }
        } else {
            log::debug!("Auto-save skipped (no state change)");
        }
        last_save_time = current_time;
    }

    // Clear screen with dark background
    clear_background(BACKGROUND_COLOR);
    
    // Draw background image
    draw_texture(
        &background_texture,
        0.0,
        0.0,
        WHITE,
    );
    
    // ... rendering code ...
    
    next_frame().await;
}
```

**Key Learning Points:**
- Fixed timestep game loops
- Mutable and immutable borrows in complex scenarios
- Performance optimization (state change detection)
- Error handling in loops
- Async frame timing

### State Machine Implementation

Clean state management with enums:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/game/state.rs start=140
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
    
    // ... more update logic ...
}
```

**Key Learning Points:**
- Early returns for different game states
- Timer-based logic with delta time
- State-dependent behavior
- Performance optimization (skip unnecessary work)

---

## 🧪 Step-by-Step Labs and Exercises

### Lab 1: Understanding and Modifying Constants (Beginner)

**Learning Goals**: Basic Rust syntax, constants, type system, compilation

**Task**: Modify your game's behavior by changing constants in `src/game/config.rs`.

**Step 1**: Open `src/game/config.rs` and examine the constants:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/game/config.rs start=3
/// Game board dimensions
pub const BOARD_WIDTH: usize = 10;       // Width in cells
pub const BOARD_HEIGHT: usize = 20;      // Height in cells  
pub const VISIBLE_HEIGHT: usize = 20;    // How many rows player sees
pub const BUFFER_HEIGHT: usize = 4;      // Extra spawn area above board

/// Game timing (in seconds)
pub const INITIAL_DROP_TIME: f64 = 1.0;  // Seconds between automatic drops
pub const LOCK_DELAY: f64 = 0.5;         // Time before piece locks in place
pub const SOFT_DROP_INTERVAL: f64 = 0.05; // Time between soft drop steps
```

**Step 2**: Make the game easier by changing drop timing:
- Change `INITIAL_DROP_TIME` from `1.0` to `2.0` (slower piece falling)
- Change `LOCK_DELAY` from `0.5` to `1.0` (more time to position pieces)

**Step 3**: Test your changes:
```bash
cargo run
```

**Step 4**: Understanding the type system - try changing `BOARD_WIDTH` to `15` and see how the rendering adapts automatically.

**Key Rust Concepts Learned:**
- Constants vs variables (`const` vs `let`)
- Type annotations (`f64`, `usize`) and their importance
- Public visibility (`pub const`)
- How constants are used throughout the codebase

---

### Lab 2: Adding a New Tetromino Shape (Intermediate)

**Learning Goals**: Enums, pattern matching, Vec operations, code organization

**Task**: Add a new tetromino shape (Plus "+" sign) to demonstrate enum extension.

**Step 1**: Add the new enum variant in `src/tetromino/types.rs`:

```rust path=/home/xenocide/rust-projects/rust-tetris/src/tetromino/types.rs start=9
/// Seven standard Tetris pieces + one custom
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TetrominoType {
    I, // Line piece (cyan)
    O, // Square piece (yellow)  
    T, // T-piece (purple)
    S, // S-piece (green)
    Z, // Z-piece (red)
    J, // J-piece (blue)
    L, // L-piece (orange)
    Plus, // Plus piece (custom - magenta) - ADD THIS LINE
}
```

**Step 2**: Update the `all()` method to include your new piece:

```rust
pub fn all() -> [TetrominoType; 8] {  // Changed from 7 to 8
    [TetrominoType::I, TetrominoType::O, TetrominoType::T, 
     TetrominoType::S, TetrominoType::Z, TetrominoType::J, 
     TetrominoType::L, TetrominoType::Plus]  // Added Plus
}
```

**Step 3**: Add color for the new piece in `src/graphics/colors.rs`:

```rust
// Add this line to colors.rs
pub const TETROMINO_PLUS: Color = Color::new(1.0, 0.0, 1.0, 1.0); // Magenta
```

**Step 4**: Update the color matching in `src/tetromino/types.rs`:

```rust
pub fn color(self) -> Color {
    match self {
        TetrominoType::I => TETROMINO_I,
        TetrominoType::O => TETROMINO_O,
        TetrominoType::T => TETROMINO_T,
        TetrominoType::S => TETROMINO_S,
        TetrominoType::Z => TETROMINO_Z,
        TetrominoType::J => TETROMINO_J,
        TetrominoType::L => TETROMINO_L,
        TetrominoType::Plus => TETROMINO_PLUS, // ADD THIS LINE
    }
}
```

**Step 5**: Define the shape in `src/tetromino/data.rs`:

```rust
// Add this function to data.rs
/// Plus-piece - doesn't rotate, always same shape  
fn get_plus_piece_blocks(_rotation: u8) -> Vec<(i32, i32)> {
    vec![
        (0, 0),   // Center block
        (-1, 0),  // Left block
        (1, 0),   // Right block  
        (0, -1),  // Top block
        (0, 1),   // Bottom block
    ]
}
```

**Step 6**: Update the main shape function:

```rust
// In get_tetromino_blocks function, add this case:
pub fn get_tetromino_blocks(piece_type: TetrominoType, rotation: u8) -> Vec<(i32, i32)> {
    match piece_type {
        TetrominoType::I => get_i_piece_blocks(rotation),
        TetrominoType::O => get_o_piece_blocks(rotation),
        TetrominoType::T => get_t_piece_blocks(rotation),
        TetrominoType::S => get_s_piece_blocks(rotation),
        TetrominoType::Z => get_z_piece_blocks(rotation),
        TetrominoType::J => get_j_piece_blocks(rotation),
        TetrominoType::L => get_l_piece_blocks(rotation),
        TetrominoType::Plus => get_plus_piece_blocks(rotation), // ADD THIS
    }
}
```

**Step 7**: Test your new piece:
```bash
cargo run
```

**Key Rust Concepts Learned:**
- Enum extension and exhaustive pattern matching
- Vec creation with `vec!` macro
- Import statements and module organization
- How the compiler ensures all cases are handled
- Coordinate systems and relative positioning

---

### Lab 3: Implementing a Simple Timer Struct (Intermediate)

**Learning Goals**: Struct design, methods, ownership, state management

**Task**: Create a reusable Timer struct that can be paused and resumed.

**Step 1**: Create a new file `src/game/timer.rs` with this Timer implementation:

```rust
//! Pause-safe timer implementation

#[derive(Debug, Clone)]
pub struct Timer {
    /// Current accumulated time
    elapsed: f64,
    /// Whether this timer is currently paused
    paused: bool,
}

impl Timer {
    /// Create a new timer starting at zero
    pub fn new() -> Self {
        Self {
            elapsed: 0.0,
            paused: false,
        }
    }
    
    /// Update the timer with delta time (only if not paused)
    pub fn update(&mut self, delta_time: f64) {
        if !self.paused {
            self.elapsed += delta_time;
        }
    }
    
    /// Get current elapsed time
    pub fn elapsed(&self) -> f64 {
        self.elapsed
    }
    
    /// Pause this timer
    pub fn pause(&mut self) {
        self.paused = true;
    }
    
    /// Resume this timer
    pub fn resume(&mut self) {
        self.paused = false;
    }
    
    /// Reset timer to zero
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
    }
    
    /// Check if enough time has passed for an interval
    pub fn check_interval(&mut self, interval: f64) -> bool {
        if self.elapsed >= interval {
            self.elapsed -= interval; // Reset for next interval
            true
        } else {
            false
        }
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
```

**Step 2**: Add the timer module to `src/game/mod.rs`:

```rust
pub mod config;
pub mod state;
pub mod timer;  // ADD THIS LINE

pub use state::{Game, GameState};
pub use timer::Timer;  // ADD THIS LINE
```

**Step 3**: Replace a raw timer in Game struct. In `src/game/state.rs`, find the Game struct and replace one of the f64 timers:

```rust
// Change this field:
pub ghost_block_blink_timer: f64,

// To this:
pub ghost_block_blink_timer: Timer,
```

**Step 4**: Update the Game::new() method:

```rust
// In Game::new(), change:
ghost_block_blink_timer: 0.0,

// To:
ghost_block_blink_timer: Timer::new(),
```

**Step 5**: Update the usage in Game::update():

```rust
// Find this line:
self.ghost_block_blink_timer += delta_time;

// Replace with:
self.ghost_block_blink_timer.update(delta_time);
```

**Step 6**: Update any code that reads the timer value:

```rust
// Change references like:
self.ghost_block_blink_timer

// To:
self.ghost_block_blink_timer.elapsed()
```

**Step 7**: Test your timer implementation:
```bash
cargo run
```

**Key Rust Concepts Learned:**
- Struct design with private fields
- Method chaining and fluent APIs
- Default trait implementation
- Module organization and re-exports
- Refactoring existing code safely

---

### Lab 4: Error Handling and File I/O (Advanced)

**Learning Goals**: Result<T, E>, error propagation, file operations, JSON serialization

**Task**: Add a simple settings file that survives game restarts.

**Step 1**: Create a simple settings struct in `src/game/settings.rs`:

```rust
//! Game settings persistence

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    /// Master volume (0.0 to 1.0)
    pub master_volume: f32,
    /// Whether to show FPS counter
    pub show_fps: bool,
    /// Player's preferred drop speed multiplier
    pub speed_multiplier: f64,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.7,
            show_fps: false,
            speed_multiplier: 1.0,
        }
    }
}

impl GameSettings {
    /// Save settings to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        // Serialize to pretty JSON string - may fail
        let json = serde_json::to_string_pretty(self)?;
        
        // Write to file - may fail
        fs::write(path, json)?;
        
        log::info!("Settings saved successfully");
        Ok(()) // Success!
    }
    
    /// Load settings from JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        // Read file contents - may fail
        let json = fs::read_to_string(path)?;
        
        // Parse JSON - may fail
        let settings: GameSettings = serde_json::from_str(&json)?;
        
        log::info!("Settings loaded successfully");
        Ok(settings)
    }
    
    /// Load settings, or create default if file doesn't exist
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(&path) {
            Ok(settings) => {
                log::info!("Loaded existing settings");
                settings
            },
            Err(e) => {
                log::info!("Could not load settings ({}), using defaults", e);
                Self::default()
            }
        }
    }
}
```

**Step 2**: Add the settings module to `src/game/mod.rs`:

```rust
pub mod settings;  // ADD THIS

pub use settings::GameSettings;  // ADD THIS
```

**Step 3**: Use your settings in main.rs. Add this after the game initialization:

```rust
// Load or create settings
let settings_path = "game_settings.json";
let mut game_settings = GameSettings::load_or_default(settings_path);

log::info!("Game settings loaded: volume={}, fps={}, speed={}", 
           game_settings.master_volume, 
           game_settings.show_fps,
           game_settings.speed_multiplier);
```

**Step 4**: Add settings save on exit. In your main loop, add a save before the game ends:

```rust
// Before std::process::exit(0), add:
if let Err(e) = game_settings.save_to_file(settings_path) {
    log::warn!("Failed to save settings: {}", e);
}
```

**Step 5**: Test error handling by trying to save to a invalid path:

```rust
// Try this to see error handling in action:
if let Err(e) = game_settings.save_to_file("/invalid/path/settings.json") {
    println!("Expected error: {}", e);
}
```

**Key Rust Concepts Learned:**
- Result<T, E> for error handling
- ? operator for error propagation
- Trait objects with `Box<dyn Error>`
- Generic functions with `AsRef<Path>`
- Pattern matching on Result variants
- Graceful error recovery strategies

---

### Lab 5: Iterator Mastery and Performance (Advanced)

**Learning Goals**: Iterator chains, closures, performance optimization, functional programming

**Task**: Implement an efficient board analyzer that finds patterns using iterators.

**Step 1**: Add this analyzer to `src/board/analyzer.rs`:

```rust
//! Board pattern analysis using Rust iterators

use crate::board::{Board, Cell};
use crate::game::config::*;

/// Statistics about the current board state
#[derive(Debug, Clone)]
pub struct BoardStats {
    pub filled_cells: usize,
    pub empty_cells: usize,
    pub complete_lines: Vec<usize>,
    pub holes: usize,        // Empty cells with filled cells above
    pub max_height: usize,   // Highest column
    pub bumpiness: usize,    // Height variation between columns
}

impl Board {
    /// Analyze the board efficiently using iterators
    pub fn analyze(&self) -> BoardStats {
        // Count filled and empty cells using iterator chains
        let (filled_cells, empty_cells) = (0..BOARD_HEIGHT + BUFFER_HEIGHT)
            .flat_map(|y| (0..BOARD_WIDTH).map(move |x| (x, y)))
            .map(|(x, y)| self.get_cell(x as i32, y as i32).unwrap_or(Cell::Empty))
            .fold((0, 0), |(filled, empty), cell| {
                match cell {
                    Cell::Empty => (filled, empty + 1),
                    Cell::Filled(_) => (filled + 1, empty),
                }
            });
        
        // Find complete lines using iterator filter
        let complete_lines: Vec<usize> = (0..BOARD_HEIGHT + BUFFER_HEIGHT)
            .filter(|&y| self.is_line_full(y))
            .collect();
        
        // Calculate column heights using iterators
        let column_heights: Vec<usize> = (0..BOARD_WIDTH)
            .map(|x| self.column_height(x))
            .collect();
        
        // Find maximum height
        let max_height = column_heights.iter().max().copied().unwrap_or(0);
        
        // Calculate bumpiness (sum of absolute differences between adjacent columns)
        let bumpiness = column_heights
            .windows(2)  // Look at pairs of adjacent columns
            .map(|pair| (pair[0] as i32 - pair[1] as i32).abs() as usize)
            .sum();
        
        // Count holes (empty cells with filled cells above them)
        let holes = (0..BOARD_WIDTH)
            .map(|x| {
                let mut found_filled = false;
                let mut hole_count = 0;
                
                // Scan from top to bottom
                for y in 0..BOARD_HEIGHT + BUFFER_HEIGHT {
                    match self.get_cell(x as i32, y as i32) {
                        Some(Cell::Filled(_)) => found_filled = true,
                        Some(Cell::Empty) if found_filled => hole_count += 1,
                        _ => {},
                    }
                }
                
                hole_count
            })
            .sum();
        
        BoardStats {
            filled_cells,
            empty_cells,
            complete_lines,
            holes,
            max_height,
            bumpiness,
        }
    }
}
```

**Step 2**: Add the analyzer module to `src/board/mod.rs`:

```rust
pub mod analyzer;  // ADD THIS
```

**Step 3**: Test your analyzer in the game loop by adding this to main.rs:

```rust
// Add this inside your main game loop (in the Playing state):
if frame_count % 300 == 0 {  // Every 5 seconds at 60fps
    let stats = game.board.analyze();
    log::info!("Board stats: filled={}, holes={}, height={}, bumpiness={}", 
               stats.filled_cells, stats.holes, stats.max_height, stats.bumpiness);
}
```

**Key Rust Concepts Learned:**
- Iterator chains with `map`, `filter`, `fold`, `collect`
- Closures and move semantics
- `flat_map` for flattening nested iterators
- `windows` for analyzing adjacent elements
- Zero-cost abstractions - iterators compile to efficient loops
- Functional programming patterns in Rust

---

### 🎯 Quick Experiments to Try Right Now

1. **Change game speed**: In `config.rs`, set `INITIAL_DROP_TIME` to `0.1` for super-fast Tetris
2. **Make a wider board**: Change `BOARD_WIDTH` to `15` and watch everything adapt
3. **Add debug logging**: Add `log::info!("Piece spawned: {:?}", new_piece.piece_type);` in spawn_next_piece()
4. **Experiment with colors**: In `colors.rs`, change `TETROMINO_I` to a new color
5. **Modify scoring**: In the scoring functions, multiply all scores by 10

Each experiment teaches you something about how Rust's type system, modules, and data flow work together!

---

## 🎓 Advanced Topics and Patterns

### Memory Layout Optimization

The game uses efficient memory layouts:

```rust
// Stack-allocated 2D array - very cache friendly
grid: [[Cell; BOARD_WIDTH]; BOARD_HEIGHT + BUFFER_HEIGHT],

// Small, Copy-able types avoid heap allocations
pub struct Tetromino {
    pub piece_type: TetrominoType,  // 1 byte enum
    pub position: (i32, i32),       // 8 bytes
    pub rotation: u8,               // 1 byte  
    pub blocks: Vec<(i32, i32)>,    // Only 4 blocks, small heap allocation
}
```

### Zero-Cost State Machines

Rust's enums enable efficient state machines:

```rust
// No runtime overhead - compiler optimizes to simple integers
enum GameState {
    Playing,
    Paused, 
    GameOver,
}

// Pattern matching compiles to jump tables
match game.state {
    GameState::Playing => { /* playing logic */ }
    GameState::Paused => { /* paused logic */ }
    GameState::GameOver => { /* game over logic */ }
}
```

### RAII and Resource Management

Automatic cleanup without garbage collection:

```rust
// When AudioSystem drops, HashMap drops, Sounds drop automatically
impl Drop for AudioSystem {
    fn drop(&mut self) {
        log::info!("Audio system shutting down");
        // Cleanup happens automatically
    }
}
```

---

## 🚀 Performance Considerations

### Hot Path Optimization

Critical game loop paths are optimized:

```rust
// Called every frame - no allocations
pub fn is_piece_valid(&self, piece: &Tetromino) -> bool {
    for (x, y) in piece.absolute_blocks() {
        if !self.board.is_position_valid(x, y) {
            return false;  // Early exit
        }
    }
    true
}
```

### Memory Access Patterns

Array layout optimized for cache locality:

```rust
// Row-major order matches typical access patterns
// Clearing lines accesses full rows efficiently
for y in 0..(BOARD_HEIGHT + BUFFER_HEIGHT) {
    if self.is_line_full(y) {
        complete_lines.push(y);
    }
}
```

### Compile-Time Optimizations

Constants enable compile-time computation:

```rust
// Computed at compile time
pub const BOARD_WIDTH_PX: f32 = BOARD_WIDTH as f32 * CELL_SIZE;
pub const BOARD_HEIGHT_PX: f32 = VISIBLE_HEIGHT as f32 * CELL_SIZE;

// Inlined in optimized builds  
pub const BOARD_OFFSET_X: f32 = (WINDOW_WIDTH as f32 - BOARD_WIDTH_PX) / 2.0;
```

---

## 📚 Key Rust Learning Outcomes

After studying this codebase, you should understand:

### 1. **Ownership System**
- When to move vs. borrow vs. clone
- How ownership prevents memory leaks and double-frees
- Using `Option<T>` to handle nullable pointers safely

### 2. **Type System**
- Enums as discriminated unions
- Structs for data modeling
- Traits for shared behavior
- Generics for code reuse

### 3. **Error Handling**
- `Result<T, E>` for recoverable errors  
- `Option<T>` for nullable values
- Error propagation with `?` operator
- Graceful degradation strategies

### 4. **Pattern Matching**
- Exhaustive matching prevents bugs
- Destructuring complex data
- Guards and multiple patterns
- `if let` for simple cases

### 5. **Memory Management**
- Stack vs. heap allocation
- RAII for automatic cleanup
- Zero-cost abstractions
- Cache-friendly data structures

### 6. **Concurrency Foundations**
- Ownership prevents data races
- Send and Sync traits
- Async/await for I/O
- Thread safety without garbage collection

---

## 🎯 Next Steps

### Immediate Projects
1. **Add multiplayer**: Network programming with async Rust
2. **AI opponent**: Implement game-playing algorithms
3. **Level editor**: File I/O and serialization
4. **Mobile port**: Cross-compilation and platform-specific code

### Rust Ecosystem Exploration
- **Web development**: Actix-web, Tokio, Serde
- **Systems programming**: Operating system interfaces
- **Game development**: Bevy engine, ECS architecture
- **Performance**: Profiling, benchmarking, optimization

### Advanced Rust Topics
- **Unsafe Rust**: When and how to use raw pointers
- **Macros**: Code generation and DSLs
- **Lifetimes**: Complex borrowing scenarios
- **Embedded**: Rust on microcontrollers

---

## 🏆 Conclusion

This Tetris implementation demonstrates that Rust isn't just about memory safety—it's about building reliable, performant software with confidence. The language's design prevents entire classes of bugs while enabling zero-cost abstractions and predictable performance.

Key takeaways:

- **Rust's ownership system** eliminates memory management bugs without garbage collection
- **Pattern matching** makes complex logic clear and bug-free  
- **The type system** encodes correctness into your program structure
- **Zero-cost abstractions** allow high-level code with low-level performance
- **Fearless concurrency** enables safe parallel programming

Your Tetris game showcases these concepts in a real, working application. It's not just a game—it's a comprehensive demonstration of modern systems programming done right.

Keep building, keep learning, and keep exploring what Rust makes possible! 🦀

---

*This guide was generated to help you learn Rust through your own code. Every concept shown here is demonstrated in your working Tetris implementation, making this a practical, hands-on learning resource.*