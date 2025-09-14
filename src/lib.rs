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