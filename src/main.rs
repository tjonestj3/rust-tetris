//! Desktop entry point. The game itself is `rust_tetris::app`, shared with the iOS build.

#[macroquad::main(window_conf)]
async fn main() {
    rust_tetris::app::run().await;
}

fn window_conf() -> macroquad::window::Conf {
    rust_tetris::app::window_conf()
}
