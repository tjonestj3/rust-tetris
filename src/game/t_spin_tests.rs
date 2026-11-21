//! T-spins and combos as scoring sees them, after a lock and the line-clear animation.

use super::Game;
use crate::board::Cell;
use crate::game::config::{BOARD_HEIGHT, BOARD_WIDTH, BUFFER_HEIGHT};
use crate::scoring::LineClearType;
use crate::tetromino::{Tetromino, TetrominoType};
use macroquad::prelude::RED;

const BOTTOM: i32 = (BOARD_HEIGHT + BUFFER_HEIGHT) as i32 - 1;

fn fill_row_except(game: &mut Game, y: i32, gaps: &[i32]) {
    for x in 0..BOARD_WIDTH as i32 {
        if !gaps.contains(&x) {
            game.board.set_cell(x, y, Cell::Filled(RED));
        }
    }
}

/// Lock the current piece and finish the clear it starts, if any.
fn lock_and_clear(game: &mut Game) {
    game.lock_current_piece();
    if game.is_clearing_lines() {
        game.finish_line_clear();
    }
}

/// A T pointing down into a T-shaped slot, an overhang over one corner, rotated in.
#[test]
fn t_spin_double_is_scored_as_one() {
    let mut game = Game::new();
    fill_row_except(&mut game, BOTTOM, &[4]);
    fill_row_except(&mut game, BOTTOM - 1, &[3, 4, 5]);
    game.board.set_cell(3, BOTTOM - 2, Cell::Filled(RED));
    let mut t = Tetromino::new(TetrominoType::T);
    t.rotation = 2;
    t.update_blocks();
    t.position = (4, BOTTOM - 1);
    game.current_piece = Some(t);
    game.last_action_was_rotation = true;

    lock_and_clear(&mut game);

    let clear = game.scoring_system.last_clear().expect("a clear");
    assert_eq!(clear.kind, LineClearType::TSpinDouble);
}

/// The same drop without a rotation last is a plain double.
#[test]
fn no_rotation_no_t_spin() {
    let mut game = Game::new();
    fill_row_except(&mut game, BOTTOM, &[4]);
    fill_row_except(&mut game, BOTTOM - 1, &[3, 4, 5]);
    game.board.set_cell(3, BOTTOM - 2, Cell::Filled(RED));
    let mut t = Tetromino::new(TetrominoType::T);
    t.rotation = 2;
    t.update_blocks();
    t.position = (4, BOTTOM - 1);
    game.current_piece = Some(t);
    game.last_action_was_rotation = false;

    lock_and_clear(&mut game);

    assert_eq!(game.scoring_system.last_clear().map(|c| c.kind), Some(LineClearType::Double));
}

/// Clear, place a piece without clearing, clear again: the second clear starts a new chain.
#[test]
fn a_lock_without_a_clear_ends_the_combo() {
    let mut game = Game::new();
    let place_i_flat = |game: &mut Game, x: i32, y: i32| {
        let mut i = Tetromino::new(TetrominoType::I);
        i.position = (x, y);
        game.current_piece = Some(i);
    };
    // Clear 1: an I completes the bottom row.
    fill_row_except(&mut game, BOTTOM, &[0, 1, 2, 3]);
    place_i_flat(&mut game, 1, BOTTOM);
    lock_and_clear(&mut game);
    assert_eq!(game.scoring_system.last_clear().map(|c| c.chain), Some(0));
    // A lock that clears nothing.
    place_i_flat(&mut game, 1, BOTTOM);
    lock_and_clear(&mut game);
    assert_eq!(game.scoring_system.current_combo(), 0);
    // Clear 2 (the row above the stray I).
    fill_row_except(&mut game, BOTTOM - 1, &[0, 1, 2, 3]);
    place_i_flat(&mut game, 1, BOTTOM - 1);
    lock_and_clear(&mut game);
    assert_eq!(game.scoring_system.last_clear().map(|c| c.chain), Some(0));
    assert_eq!(game.scoring_system.pieces(), 3);
}

/// A T turned up high and hard-dropped into a spot with three corners covered (here the left
/// wall and a block on the floor) moved after its rotation: not a T-spin.
#[test]
fn rotate_then_hard_drop_is_no_t_spin() {
    let mut game = Game::new();
    fill_row_except(&mut game, BOTTOM, &[0]);
    let mut t = Tetromino::new(TetrominoType::T);
    t.rotation = 1;
    t.update_blocks();
    t.position = (0, BOTTOM - 8);
    game.current_piece = Some(t);
    game.last_action_was_rotation = true;
    fill_row_except(&mut game, BOTTOM - 1, &[0, 1, 2]);

    game.hard_drop();
    if game.is_clearing_lines() {
        game.finish_line_clear();
    }

    assert!(!game.lock_was_t_spin);
}
