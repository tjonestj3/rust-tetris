use macroquad::prelude::*;
use rust_tetris::game::config::*;
use rust_tetris::{Game, MenuSystem, MenuAction};

/// Window configuration for macroquad
fn window_conf() -> Conf {
    Conf {
        window_title: WINDOW_TITLE.to_owned(),
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        window_resizable: false,
        high_dpi: false,
        ..Default::default()
    }
}

/// Create a simple background image
fn create_simple_background() -> Image {
    let width = WINDOW_WIDTH as u16;
    let height = WINDOW_HEIGHT as u16;
    let mut image = Image::gen_image_color(width, height, Color::new(0.02, 0.02, 0.08, 1.0));
    
    // Add some simple patterns
    for y in 0..height {
        for x in 0..width {
            let fx = x as f32;
            let fy = y as f32;
            
            // Simple grid pattern
            let grid_size = 40.0;
            let grid_x = (fx / grid_size) % 1.0;
            let grid_y = (fy / grid_size) % 1.0;
            
            if grid_x < 0.05 || grid_x > 0.95 || grid_y < 0.05 || grid_y > 0.95 {
                let mut pixel = image.get_pixel(x as u32, y as u32);
                pixel.r = (pixel.r + 0.03).min(1.0);
                pixel.g = (pixel.g + 0.08).min(1.0);
                pixel.b = (pixel.b + 0.12).min(1.0);
                image.set_pixel(x as u32, y as u32, pixel);
            }
        }
    }
    
    image
}

#[macroquad::main(window_conf)]
async fn main() {
    // Initialize logging
    env_logger::init();
    log::info!("Starting Rust Tetris Menu System Demo");
    
    // Create background texture
    let background_texture = Texture2D::from_image(&create_simple_background());
    
    // Initialize menu system
    let mut menu_system = MenuSystem::new();
    
    // Main loop
    loop {
        let delta_time = get_frame_time() as f64;
        
        // Update menu system
        menu_system.update(delta_time);
        
        // Handle menu input
        let action = menu_system.handle_input();
        
        match action {
            MenuAction::NewGame => {
                log::info!("Starting new game!");
                // Here we would transition to game mode
                // For now, just log it
            },
            MenuAction::LoadGame => {
                log::info!("Loading saved game!");
                // Here we would load and start the saved game
                // For now, just log it  
            },
            MenuAction::Quit => {
                log::info!("Quitting game");
                std::process::exit(0);
            },
            MenuAction::None => {
                // Continue with menu
            },
        }
        
        // Render menu system
        menu_system.render(&background_texture);
        
        next_frame().await;
    }
}