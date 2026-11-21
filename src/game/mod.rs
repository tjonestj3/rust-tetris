//! Game module containing core game logic and state management

pub mod config;
pub mod state;

#[cfg(test)]
mod movement_tests;
#[cfg(test)]
mod t_spin_tests;

pub use state::{Game, GameState};
