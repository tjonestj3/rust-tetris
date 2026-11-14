//! iOS entry point. The Xcode project's `main.m` calls `rust_tetris_main`, which starts the game
//! the same way `#[macroquad::main]` does on desktop. miniquad then runs UIKit's
//! `UIApplicationMain`, so this never returns.

#[no_mangle]
pub extern "C" fn rust_tetris_main() {
    macroquad::Window::from_config(rust_tetris::app::window_conf(), rust_tetris::app::run());
}
