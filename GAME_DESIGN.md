# Tetris Game Design Document

## Overview
A high-performance Tetris implementation in Rust focusing on smooth 60fps gameplay, clean architecture, and extensible design.

## Core Architecture

### 1. Game Engine Components
- **Game Loop**: Fixed timestep with interpolation for smooth rendering
- **Input System**: Responsive key handling with configurable key bindings
- **Rendering System**: Efficient 2D graphics with minimal allocations
- **Audio System**: Sound effects and background music
- **State Management**: Game states (Menu, Playing, Paused, GameOver)

### 2. Game Logic Components
- **Board**: 10x20 grid representation with collision detection
- **Tetromino System**: 7 piece types with rotation mechanics
- **Line Clearing**: Row detection and clearing with animations
- **Scoring System**: Points, levels, and statistics tracking
- **Timing System**: Piece drop timing with level-based acceleration

## Technical Requirements

### Performance Targets
- **Frame Rate**: Consistent 60 FPS
- **Input Latency**: < 16ms response time
- **Memory**: Zero allocations during gameplay loop
- **CPU**: Efficient algorithms for collision detection and line clearing

### Recommended Packages

#### Graphics & Rendering
```toml
[dependencies]
# Option 1: Macroquad (Recommended - Simple, Fast)
macroquad = "0.4"

# Option 2: Bevy (Game Engine - More Complex but Feature Rich)
# bevy = "0.12"

# Option 3: ggez (2D Game Framework)
# ggez = "0.9"

# Option 4: Raylib (C binding - Very Fast)
# raylib = "3.7"
```

#### Additional Dependencies
```toml
# Serialization for save data and settings
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Random number generation for piece spawning
rand = "0.8"

# Audio (if using macroquad)
# macroquad includes audio support

# Configuration management
config = "0.13"

# Logging
log = "0.4"
env_logger = "0.10"
```

## Implementation Phases

### Phase 1: Foundation (Current - feature/board-setup)
- [ ] Project setup with chosen graphics library
- [ ] Basic game loop implementation
- [ ] Board data structure and rendering
- [ ] Input system foundation
- [ ] Basic tetromino representation

### Phase 2: Core Gameplay (feature/tetromino-mechanics)
- [ ] Tetromino movement and rotation
- [ ] Collision detection system
- [ ] Piece spawning and positioning
- [ ] Basic drop mechanics

### Phase 3: Game Logic (feature/game-rules)
- [ ] Line clearing detection and removal
- [ ] Scoring system implementation
- [ ] Level progression mechanics
- [ ] Game over conditions

### Phase 4: Polish & Effects (feature/visual-polish)
- [ ] Smooth animations for line clearing
- [ ] Visual effects for piece placement
- [ ] UI improvements and HUD
- [ ] Sound effects integration

### Phase 5: Advanced Features (feature/enhancements)
- [ ] Hold piece functionality
- [ ] Next piece preview
- [ ] High score system
- [ ] Settings and configuration

## Detailed Component Design

### Board Component
```rust
// Conceptual structure - not final implementation
struct Board {
    grid: [[Option<Color>; BOARD_WIDTH]; BOARD_HEIGHT],
    width: usize,
    height: usize,
}

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;
const VISIBLE_HEIGHT: usize = 20;
const BUFFER_HEIGHT: usize = 4; // Extra rows above visible area
```

### Tetromino System
```rust
// Seven standard Tetris pieces
enum TetrominoType {
    I, O, T, S, Z, J, L
}

struct Tetromino {
    piece_type: TetrominoType,
    rotation: u8, // 0-3 for four rotations
    position: (i32, i32), // x, y coordinates
    blocks: Vec<(i32, i32)>, // Relative block positions
}
```

### Game State Management
```rust
enum GameState {
    MainMenu,
    Playing,
    Paused,
    GameOver,
    Settings,
}

struct GameData {
    state: GameState,
    board: Board,
    current_piece: Option<Tetromino>,
    next_piece: TetrominoType,
    score: u32,
    level: u32,
    lines_cleared: u32,
}
```

### Input System Design
```rust
struct InputConfig {
    move_left: KeyCode,
    move_right: KeyCode,
    soft_drop: KeyCode,
    hard_drop: KeyCode,
    rotate_cw: KeyCode,
    rotate_ccw: KeyCode,
    hold: KeyCode,
    pause: KeyCode,
}

// Default key bindings
// Left/Right arrows: Move
// Down arrow: Soft drop
// Space: Hard drop
// Up arrow/X: Rotate clockwise
// Z: Rotate counterclockwise
// C: Hold piece
// P/Escape: Pause
```

## Graphics Library Comparison

### Macroquad (Recommended)
**Pros:**
- Simple API, easy to learn
- Built-in audio support
- Good performance for 2D games
- Cross-platform
- Active development

**Cons:**
- Less ecosystem than Bevy
- Fewer built-in game engine features

### Bevy
**Pros:**
- Full-featured game engine
- ECS architecture
- Extensive plugin ecosystem
- Professional-grade features

**Cons:**
- Steeper learning curve
- Might be overkill for Tetris
- Larger compile times

### Decision: Macroquad
For this Tetris implementation, **Macroquad** is recommended because:
1. Simple, focused API perfect for 2D games
2. Excellent performance characteristics
3. Built-in audio support
4. Rapid prototyping capabilities
5. Suitable complexity for our project scope

## Next Steps (Phase 1 Implementation)
1. Initialize Cargo.toml with Macroquad dependency
2. Create basic game window and initialization
3. Implement game loop structure
4. Create Board struct and basic rendering
5. Setup input handling foundation
6. Create basic tetromino data structures

## Performance Considerations
- Use object pooling for tetrominoes to avoid allocations
- Implement efficient line-clearing algorithm
- Cache rendered sprites/textures
- Use fixed-point arithmetic where appropriate
- Profile regularly to identify bottlenecks

## Testing Strategy
- Unit tests for game logic components
- Integration tests for input handling
- Performance benchmarks for critical paths
- Visual testing for rendering accuracy