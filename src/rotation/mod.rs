//! Super Rotation System (SRS) implementation
//! 
//! This module implements the Super Rotation System used in modern Tetris games.
//! SRS includes wall kicks that allow pieces to rotate in tight spaces by trying
//! multiple offset positions when the basic rotation would collide.

pub mod srs;
pub mod kick_tables;

#[cfg(test)]
mod integration_tests;

pub use srs::{RotationSystem, SRSRotationSystem, RotationState, RotationResult};
pub use kick_tables::{WallKickData, get_wall_kick_offsets};
