```
 ████████╗███████╗████████╗██████╗ ██╗███████╗    ██████╗ ██╗   ██╗███████╗████████╗
 ╚══██╔══╝██╔════╝╚══██╔══╝██╔══██╗██║██╔════╝    ██╔══██╗██║   ██║██╔════╝╚══██╔══╝
    ██║   █████╗     ██║   ██████╔╝██║███████╗    ██████╔╝██║   ██║███████╗   ██║   
    ██║   ██╔══╝     ██║   ██╔══██╗██║╚════██║    ██╔══██╗██║   ██║╚════██║   ██║   
    ██║   ███████╗   ██║   ██║  ██║██║███████║    ██║  ██║╚██████╔╝███████║   ██║   
    ╚═╝   ╚══════╝   ╚═╝   ╚═╝  ╚═╝╚═╝╚══════╝    ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝   
```

<div align="center">

# 🎮 **TETRIS RUST** 🔥

*The most epic Tetris implementation in Rust you've ever seen!*

[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-red.svg?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Powered by Macroquad](https://img.shields.io/badge/Powered%20by-Macroquad-blue.svg?style=for-the-badge)](https://macroquad.rs/)
[![Retro Gaming](https://img.shields.io/badge/Style-Retro%20Gaming-purple.svg?style=for-the-badge)]()
[![Magic Enabled](https://img.shields.io/badge/Magic-Enabled-gold.svg?style=for-the-badge)]()

---

*Experience the classic puzzle game with modern enhancements, magical powers, and blazing performance!*

</div>

## 🌟 **FEATURES THAT BLOW YOUR MIND**

### 🎯 **Core Gameplay**
- **Classic Tetris Mechanics** - The timeless puzzle game you know and love
- **Smooth 60fps Performance** - Buttery smooth gameplay with optimized rendering
- **Progressive Difficulty** - Speed increases as you level up for endless challenge
- **Line Clear Animations** - Satisfying visual feedback with every cleared line
- **TETRIS Celebration** - Special effects when you clear 4 lines at once!

### 🔮 **MAGICAL MAGE POWERS**
- **Transform into a Mage** - Your character becomes a powerful magical being
- **Ghost Block Spells** - Earn magical ghost blocks every 4 lines cleared
- **Strategic Spell Casting** - Press `B` to activate ghost block placement mode
- **Smart Targeting System** - Ghost blocks only target strategic positions on partially filled rows
- **Fireball Projectiles** - Watch your ghost blocks fly across the screen as blazing fireballs!
- **Auto-Fire Magic** - Optimal single-block positions trigger instant spell casting

### 💾 **ADVANCED SAVE SYSTEM**
- **Persistent Game State** - Never lose your progress with automatic save/load
- **Startup Menu** - Choose to continue your saved game or start fresh
- **Smart Auto-Save** - Performance-optimized saving that only triggers on state changes
- **Manual Save** - Press `Ctrl+S` anytime to save your current progress
- **Seamless Experience** - Pick up exactly where you left off

### 🎮 **ENHANCED CONTROLS**
- **Responsive Movement** - Smooth piece control with customizable timing
- **Hold Piece System** - Store a piece for later use (press `C`)
- **Ghost Piece Preview** - See exactly where your piece will land
- **Lock Delay** - Advanced piece locking mechanics for precise placement
- **Multiple Input Methods** - Support for both arrow keys and WASD

### 🎨 **VISUAL EXCELLENCE**
- **Retro Chess Background** - Procedurally generated fiery chess pattern backdrop
- **Enhanced UI** - Clean, readable interface with real-time statistics
- **Particle Effects** - Magical fireball trails and explosive animations
- **Color-Coded Pieces** - Each Tetromino has its own distinctive color
- **Visual Feedback** - Animations for line clears, celebrations, and magic spells

## 🎹 **CONTROLS**

### 🎯 **Basic Controls**
| Key | Action |
|-----|--------|
| `←/→` or `A/D` | Move piece left/right |
| `↓` or `S` | Soft drop (faster falling) |
| `↑`, `X`, or `W` | Rotate clockwise |
| `Z` | Rotate counterclockwise |
| `Space` | Hard drop (instant placement) |
| `C` | Hold/swap piece |

### 🔮 **Magic Controls**
| Key | Action |
|-----|--------|
| `B` | Activate ghost block spell casting mode |
| `M` | Next strategic position (in spell mode) |
| `N` | Previous strategic position (in spell mode) |
| `B` (in spell mode) | Cast fireball at target position |
| `Arrow Keys` | Fine-tune spell target position |

### 🎮 **System Controls**
| Key | Action |
|-----|--------|
| `P` | Pause/unpause game |
| `R` | Reset game |
| `Ctrl+S` | Manual save |
| `Esc` | Quit game |

## 🚀 **INSTALLATION & SETUP**

### **Prerequisites**
- Rust 1.70+ installed ([Get Rust](https://rustup.rs/))
- A decent graphics card for smooth 60fps gameplay
- Audio system for epic sound effects

### **Quick Start**
```bash
# Clone this epic repository
git clone https://github.com/tjonestj3/rust-tetris.git
cd rust-tetris

# Build and run the magic
cargo run --release

# For development with debug info
cargo run
```

### **Build Optimized Version**
```bash
cargo build --release
./target/release/rust-tetris
```

## 🎯 **GAMEPLAY STRATEGIES**

### 🧠 **Master the Basics**
1. **Build Flat** - Keep your stack as flat as possible
2. **Leave Wells** - Create narrow gaps for I-pieces (line pieces)
3. **Plan Ahead** - Use the next piece preview to strategize
4. **Use Hold Wisely** - Save pieces for the perfect moment

### 🔮 **Magic Mastery**
1. **Earn Ghost Blocks** - Clear 4 lines to earn magical ammunition
2. **Strategic Casting** - Ghost blocks only target partially filled rows
3. **Auto-Fire Advantage** - Single-block completions cast automatically
4. **Combo Magic** - Use ghost blocks to set up massive line clears

## 🏗️ **TECHNICAL ARCHITECTURE**

### **Performance Features**
- **State Hash Optimization** - Smart auto-save only when game state changes
- **Efficient Rendering** - Optimized draw calls and texture management
- **Memory Management** - Rust's zero-cost abstractions for maximum performance
- **60fps Guarantee** - Consistent frame timing for smooth gameplay

### **Code Structure**
```
src/
├── main.rs                 # Game loop and window management
├── game/
│   ├── state.rs           # Core game logic and state management
│   └── config.rs          # Game constants and configuration
├── board/
│   └── board.rs           # Game board implementation
├── tetromino/
│   ├── types.rs           # Piece definitions and logic
│   └── data.rs            # Piece shape data
├── graphics/
│   └── colors.rs          # Color schemes and visual constants
└── audio/
    └── system.rs          # Sound effects and music
```

## 🎨 **CUSTOMIZATION**

### **Visual Tweaks**
Modify `src/game/config.rs` to adjust:
- Game speed and difficulty progression
- Board dimensions and cell sizes
- Animation timings and effects
- Color schemes and visual styles

### **Gameplay Tweaks**
- Adjust drop intervals and lock delays
- Modify scoring system multipliers
- Change ghost block earning rates
- Customize input response timings

## 🤝 **CONTRIBUTING**

Want to add more magic to this epic Tetris implementation?

1. Fork this repository
2. Create your feature branch (`git checkout -b feature/awesome-magic`)
3. Commit your changes (`git commit -m 'Add some awesome magic'`)
4. Push to the branch (`git push origin feature/awesome-magic`)
5. Open a Pull Request with detailed description

## 📜 **LICENSE**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 **ACKNOWLEDGMENTS**

- **Alexey Pajitnov** - Creator of the original Tetris
- **Macroquad Team** - For the amazing Rust game framework
- **Rust Community** - For creating the most awesome systems programming language
- **Retro Gaming Community** - For keeping the classic spirit alive

---

<div align="center">

### 🎮 **Ready to Experience the Magic?** 🔥

*Clone, build, and prepare for the most epic Tetris adventure of your life!*

**Made with ❤️ and 🔥 in Rust**

</div>
