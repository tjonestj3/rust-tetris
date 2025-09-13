# Phase 1: Foundation & Board Setup

## Current Branch: `feature/board-setup`

## Objectives
Establish the foundational components for our Tetris game with smooth 60fps performance.

## Phase 1 Tasks

### ✅ Completed
- [x] Project initialization with Rust/Cargo
- [x] Dependency setup in Cargo.toml
- [x] Game design document creation
- [x] Feature branch creation

### 🚧 In Progress
- [ ] Basic project structure setup
- [ ] Core constants and configuration
- [ ] Game window initialization
- [ ] Basic game loop implementation
- [ ] Board data structure
- [ ] Board rendering system
- [ ] Input system foundation
- [ ] Basic tetromino data structures

## Implementation Details

### 1. Project Structure
```
src/
├── main.rs              # Entry point and game loop
├── lib.rs               # Library exports
├── game/
│   ├── mod.rs          # Game module exports
│   ├── state.rs        # Game state management
│   ├── config.rs       # Game configuration and constants
│   └── loop.rs         # Main game loop logic
├── board/
│   ├── mod.rs          # Board module exports
│   ├── board.rs        # Board data structure and logic
│   └── renderer.rs     # Board rendering functionality
├── tetromino/
│   ├── mod.rs          # Tetromino module exports
│   ├── types.rs        # Tetromino type definitions
│   └── data.rs         # Tetromino shape data
├── input/
│   ├── mod.rs          # Input module exports
│   └── handler.rs      # Input handling logic
└── graphics/
    ├── mod.rs          # Graphics module exports
    ├── colors.rs       # Color definitions
    └── utils.rs        # Rendering utilities
```

### 2. Core Constants (config.rs)
```rust
// Game dimensions
pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;
pub const VISIBLE_HEIGHT: usize = 20;
pub const BUFFER_HEIGHT: usize = 4;

// Rendering constants
pub const CELL_SIZE: f32 = 30.0;
pub const BOARD_OFFSET_X: f32 = 50.0;
pub const BOARD_OFFSET_Y: f32 = 50.0;

// Window dimensions
pub const WINDOW_WIDTH: i32 = 800;
pub const WINDOW_HEIGHT: i32 = 600;
pub const TARGET_FPS: i32 = 60;

// Timing
pub const INITIAL_DROP_TIME: f64 = 1.0; // 1 second per drop at level 1
```

### 3. Board Implementation Strategy
- **Data Structure**: 2D array of `Option<Color>` for empty/filled cells
- **Coordinate System**: (0,0) at top-left, standard Tetris board orientation
- **Rendering**: Efficient rectangle drawing with cell borders
- **Bounds Checking**: Safe access methods for grid manipulation

### 4. Game Loop Architecture
- **Fixed Timestep**: 60 FPS with consistent timing
- **Update/Render Separation**: Logic updates separate from rendering
- **Input Processing**: Non-blocking input handling
- **State Management**: Clean state transitions

### 5. Input System Foundation
- **Key Mapping**: Configurable key bindings
- **Response Time**: < 16ms input latency target
- **Repeat Handling**: Proper key repeat for movement
- **State Tracking**: Track pressed/released states

## Success Criteria for Phase 1
- [ ] Game window opens and displays at 60 FPS
- [ ] Empty Tetris board renders correctly with grid
- [ ] Basic input detection works (arrow keys logged)
- [ ] Clean code structure with proper module organization
- [ ] No compiler warnings or errors
- [ ] Performance: Consistent 60 FPS with < 5% CPU usage

## Technical Validation
- [ ] `cargo build` completes without warnings
- [ ] `cargo clippy` passes with no issues
- [ ] `cargo fmt` applied to all code
- [ ] Basic functionality tested manually

## Next Phase Preview
Phase 2 will add:
- Tetromino movement and rotation
- Collision detection
- Piece spawning mechanics
- Drop timing implementation

## Notes
- Focus on clean, maintainable code structure
- Optimize for 60 FPS from the start
- Use Rust's type system for safety
- Document all public APIs
- Profile early to identify bottlenecks