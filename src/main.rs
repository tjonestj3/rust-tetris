use macroquad::prelude::*;
use rust_tetris::game::config::*;
use rust_tetris::graphics::colors::*;
use rust_tetris::board::Board;
use rust_tetris::game::{Game, GameState};
use rust_tetris::tetromino::{Tetromino, TetrominoType};

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

#[macroquad::main(window_conf)]
async fn main() {
    // Initialize logging
    env_logger::init();
    log::info!("Starting Rust Tetris v{}", env!("CARGO_PKG_VERSION"));
    
    // Log layout calculations for debugging
    log::info!("Window: {}x{}", WINDOW_WIDTH, WINDOW_HEIGHT);
    log::info!("Board: {}x{} cells = {}x{} pixels", BOARD_WIDTH, VISIBLE_HEIGHT, BOARD_WIDTH_PX, BOARD_HEIGHT_PX);
    log::info!("Board position: ({}, {})", BOARD_OFFSET_X, BOARD_OFFSET_Y);
    log::info!("Required height: {} + {} = {}", BOARD_OFFSET_Y, BOARD_HEIGHT_PX, BOARD_OFFSET_Y + BOARD_HEIGHT_PX);

    // Load background texture
    let background_texture = Texture2D::from_image(&create_chess_background());
    
    // Initialize game state
    let mut game = Game::new();
    let mut frame_count = 0u64;
    let mut last_fps_time = get_time();
    let mut fps = 0.0;
    
    log::info!("Game initialized with first piece: {:?}", 
               game.current_piece.as_ref().map(|p| p.piece_type).unwrap_or(TetrominoType::T));

    // Main game loop
    loop {
        let delta_time = get_frame_time();
        frame_count += 1;

        // Calculate FPS
        let current_time = get_time();
        if current_time - last_fps_time >= 1.0 {
            fps = frame_count as f64 / (current_time - last_fps_time);
            frame_count = 0;
            last_fps_time = current_time;
        }

        // Handle input
        handle_input(&mut game);
        
        // Update game logic
        game.update(delta_time as f64);

        // Clear screen with dark background
        clear_background(BACKGROUND_COLOR);
        
        // Draw background image
        draw_texture(
            &background_texture,
            0.0,
            0.0,
            WHITE,
        );
        
        // Draw semi-transparent overlay for better text readability
        draw_rectangle(
            0.0,
            0.0,
            WINDOW_WIDTH as f32,
            WINDOW_HEIGHT as f32,
            Color::new(0.0, 0.0, 0.0, 0.4),
        );

        // Draw enhanced Tetris board with real data
        draw_enhanced_board_with_data(&game.board);
        
        // Draw the current falling piece
        if let Some(ref piece) = game.current_piece {
            draw_falling_piece(piece);
        }
        
        // Draw next piece preview
        draw_next_piece_preview(&game.next_piece);
        
        // Draw title with enhanced styling
        draw_enhanced_ui(&game);

        // Show FPS in debug mode
        if SHOW_FPS {
            let fps_text = format!("FPS: {:.1}", fps);
            draw_text(
                &fps_text,
                WINDOW_WIDTH as f32 - 100.0,
                30.0,
                TEXT_SIZE,
                TEXT_COLOR,
            );
        }

        // Log basic info periodically (every 60 frames)
        if frame_count % 60 == 0 && DEBUG_MODE {
            log::debug!("Frame: {}, FPS: {:.1}, Delta: {:.4}ms", 
                       frame_count, fps, delta_time * 1000.0);
        }

        next_frame().await;
    }
}

/// Create a procedural chess-like background
fn create_chess_background() -> Image {
    let width = WINDOW_WIDTH as u16;
    let height = WINDOW_HEIGHT as u16;
    let mut image = Image::gen_image_color(width, height, Color::new(0.1, 0.05, 0.0, 1.0));
    
    // Create a fiery chess pattern
    for y in 0..height {
        for x in 0..width {
            let chess_x = (x / 64) % 2;
            let chess_y = (y / 64) % 2;
            
            let base_color = if (chess_x + chess_y) % 2 == 0 {
                Color::new(0.15, 0.08, 0.02, 1.0) // Dark brown
            } else {
                Color::new(0.25, 0.15, 0.05, 1.0) // Light brown
            };
            
            // Add some fire-like gradient
            let gradient = (y as f32 / height as f32) * 0.3;
            let final_color = Color::new(
                (base_color.r + gradient * 0.8).min(1.0),
                (base_color.g + gradient * 0.4).min(1.0),
                (base_color.b + gradient * 0.1).min(1.0),
                1.0,
            );
            
            image.set_pixel(x as u32, y as u32, final_color);
        }
    }
    
    image
}

/// Handle player input
fn handle_input(game: &mut Game) {
    // Quit game
    if is_key_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
    
    // Only handle game input when playing
    if game.state != GameState::Playing {
        return;
    }
    
    // Movement
    if is_key_pressed(KeyCode::Left) {
        game.move_piece(-1, 0);
    }
    if is_key_pressed(KeyCode::Right) {
        game.move_piece(1, 0);
    }
    if is_key_pressed(KeyCode::Down) {
        if game.move_piece(0, 1) {
            game.score += SCORE_SOFT_DROP;
        }
    }
    
    // Rotation
    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::X) {
        game.rotate_piece_clockwise();
    }
    if is_key_pressed(KeyCode::Z) {
        game.rotate_piece_counterclockwise();
    }
    
    // Hard drop
    if is_key_pressed(KeyCode::Space) {
        game.hard_drop();
    }
    
    // Pause
    if is_key_pressed(KeyCode::P) {
        game.toggle_pause();
    }
    
    // Reset game (R key)
    if is_key_pressed(KeyCode::R) {
        game.reset();
    }
}

/// Draw the currently falling piece
fn draw_falling_piece(piece: &Tetromino) {
    for (x, y) in piece.absolute_blocks() {
        // Only draw blocks that are in the visible area
        if y >= BUFFER_HEIGHT as i32 {
            let visible_y = y - BUFFER_HEIGHT as i32;
            let cell_x = BOARD_OFFSET_X + (x as f32 * CELL_SIZE);
            let cell_y = BOARD_OFFSET_Y + (visible_y as f32 * CELL_SIZE);
            
            // Draw filled cell with border
            draw_rectangle(
                cell_x + 1.0,
                cell_y + 1.0,
                CELL_SIZE - 2.0,
                CELL_SIZE - 2.0,
                piece.color(),
            );
            
            // Draw subtle highlight for 3D effect
            draw_rectangle(
                cell_x + 2.0,
                cell_y + 2.0,
                CELL_SIZE - 4.0,
                6.0,
                Color::new(1.0, 1.0, 1.0, 0.3),
            );
            
            // Draw subtle shadow at bottom
            draw_rectangle(
                cell_x + 2.0,
                cell_y + CELL_SIZE - 6.0,
                CELL_SIZE - 4.0,
                4.0,
                Color::new(0.0, 0.0, 0.0, 0.2),
            );
        }
    }
}

/// Draw the next piece preview
fn draw_next_piece_preview(next_piece_type: &TetrominoType) {
    let preview_x = PREVIEW_OFFSET_X;
    let preview_y = PREVIEW_OFFSET_Y;
    
    // Draw preview panel background
    draw_rectangle(
        preview_x - 10.0,
        preview_y - 30.0,
        PREVIEW_SIZE + 20.0,
        PREVIEW_SIZE + 40.0,
        Color::new(0.0, 0.0, 0.0, 0.6),
    );
    
    // Draw preview panel border
    draw_rectangle_lines(
        preview_x - 10.0,
        preview_y - 30.0,
        PREVIEW_SIZE + 20.0,
        PREVIEW_SIZE + 40.0,
        2.0,
        UI_BORDER,
    );
    
    // Draw "NEXT" label
    draw_text(
        "NEXT",
        preview_x,
        preview_y - 10.0,
        TEXT_SIZE,
        Color::new(1.0, 0.9, 0.7, 1.0),
    );
    
    // Create a temporary piece for preview
    let preview_piece = Tetromino::new(*next_piece_type);
    let blocks = preview_piece.blocks;
    
    // Center the piece in the preview area
    let center_x = preview_x + PREVIEW_SIZE / 2.0;
    let center_y = preview_y + PREVIEW_SIZE / 2.0;
    
    // Draw the piece blocks
    for (dx, dy) in blocks {
        let block_x = center_x + (dx as f32 * CELL_SIZE * 0.7); // Smaller size for preview
        let block_y = center_y + (dy as f32 * CELL_SIZE * 0.7);
        let block_size = CELL_SIZE * 0.7;
        
        // Draw filled cell
        draw_rectangle(
            block_x,
            block_y,
            block_size - 1.0,
            block_size - 1.0,
            next_piece_type.color(),
        );
        
        // Draw highlight
        draw_rectangle(
            block_x + 1.0,
            block_y + 1.0,
            block_size - 3.0,
            4.0,
            Color::new(1.0, 1.0, 1.0, 0.3),
        );
    }
}


/// Draw enhanced Tetris board with modern styling and real data
fn draw_enhanced_board_with_data(board: &Board) {
    // Draw board shadow
    draw_rectangle(
        BOARD_OFFSET_X + 5.0,
        BOARD_OFFSET_Y + 5.0,
        BOARD_WIDTH_PX,
        BOARD_HEIGHT_PX,
        BOARD_SHADOW,
    );
    
    // Draw board background with gradient effect
    draw_rectangle(
        BOARD_OFFSET_X,
        BOARD_OFFSET_Y,
        BOARD_WIDTH_PX,
        BOARD_HEIGHT_PX,
        BOARD_BACKGROUND,
    );
    
    // Draw subtle inner glow
    draw_rectangle_lines(
        BOARD_OFFSET_X - 1.0,
        BOARD_OFFSET_Y - 1.0,
        BOARD_WIDTH_PX + 2.0,
        BOARD_HEIGHT_PX + 2.0,
        1.0,
        Color::new(0.6, 0.7, 0.9, 0.3),
    );
    
    // Draw grid lines with improved styling
    for x in 0..=BOARD_WIDTH {
        let line_x = BOARD_OFFSET_X + (x as f32 * CELL_SIZE);
        draw_line(
            line_x,
            BOARD_OFFSET_Y,
            line_x,
            BOARD_OFFSET_Y + BOARD_HEIGHT_PX,
            GRID_LINE_WIDTH,
            GRID_LINE_COLOR,
        );
    }

    for y in 0..=VISIBLE_HEIGHT {
        let line_y = BOARD_OFFSET_Y + (y as f32 * CELL_SIZE);
        draw_line(
            BOARD_OFFSET_X,
            line_y,
            BOARD_OFFSET_X + BOARD_WIDTH_PX,
            line_y,
            GRID_LINE_WIDTH,
            GRID_LINE_COLOR,
        );
    }
    
    // Draw filled cells from the board data
    for y in 0..VISIBLE_HEIGHT {
        for x in 0..BOARD_WIDTH {
            // Convert to board coordinates (includes buffer rows)
            let board_y = (y + BUFFER_HEIGHT) as i32;
            let board_x = x as i32;
            
            if let Some(cell) = board.get_cell(board_x, board_y) {
                if let Some(color) = cell.color() {
                    let cell_x = BOARD_OFFSET_X + (x as f32 * CELL_SIZE);
                    let cell_y = BOARD_OFFSET_Y + (y as f32 * CELL_SIZE);
                    
                    // Draw filled cell with border
                    draw_rectangle(
                        cell_x + 1.0,
                        cell_y + 1.0,
                        CELL_SIZE - 2.0,
                        CELL_SIZE - 2.0,
                        color,
                    );
                    
                    // Draw subtle highlight for 3D effect
                    draw_rectangle(
                        cell_x + 2.0,
                        cell_y + 2.0,
                        CELL_SIZE - 4.0,
                        6.0,
                        Color::new(1.0, 1.0, 1.0, 0.3),
                    );
                    
                    // Draw subtle shadow at bottom
                    draw_rectangle(
                        cell_x + 2.0,
                        cell_y + CELL_SIZE - 6.0,
                        CELL_SIZE - 4.0,
                        4.0,
                        Color::new(0.0, 0.0, 0.0, 0.2),
                    );
                }
            }
        }
    }

    // Draw enhanced border with multiple layers
    draw_rectangle_lines(
        BOARD_OFFSET_X,
        BOARD_OFFSET_Y,
        BOARD_WIDTH_PX,
        BOARD_HEIGHT_PX,
        BOARD_BORDER_WIDTH,
        BOARD_BORDER_COLOR,
    );
}


/// Draw enhanced UI elements
fn draw_enhanced_ui(game: &Game) {
    // Draw title with shadow effect
    let title = "RUST TETRIS";
    let title_x = (WINDOW_WIDTH as f32 - measure_text(title, None, TITLE_TEXT_SIZE as u16, 1.0).width) / 2.0;
    
    // Title shadow
    draw_text(
        title,
        title_x + 2.0,
        42.0,
        TITLE_TEXT_SIZE,
        Color::new(0.0, 0.0, 0.0, 0.8),
    );
    
    // Main title
    draw_text(
        title,
        title_x,
        40.0,
        TITLE_TEXT_SIZE,
        Color::new(1.0, 0.9, 0.7, 1.0),
    );
    
    // Subtitle
    let subtitle = "Phase 1 - Foundation";
    let subtitle_x = (WINDOW_WIDTH as f32 - measure_text(subtitle, None, TEXT_SIZE as u16, 1.0).width) / 2.0;
    
    draw_text(
        subtitle,
        subtitle_x,
        65.0,
        TEXT_SIZE,
        Color::new(0.8, 0.8, 0.9, 0.8),
    );
    
    // Instructions with background
    let instructions = vec![
        "Controls:",
        "ESC - Quit Game",
        "Arrow Keys - Move (Coming Soon!)",
        "Space - Drop (Coming Soon!)",
    ];
    
    let inst_x = 20.0;
    let mut inst_y = WINDOW_HEIGHT as f32 - 120.0;
    
    // Instructions background
    draw_rectangle(
        inst_x - 10.0,
        inst_y - 25.0,
        280.0,
        100.0,
        Color::new(0.0, 0.0, 0.0, 0.6),
    );
    
    for (i, instruction) in instructions.iter().enumerate() {
        let color = if i == 0 {
            Color::new(1.0, 0.9, 0.7, 1.0) // Header color
        } else {
            Color::new(0.9, 0.9, 0.95, 0.9) // Normal text
        };
        
        draw_text(instruction, inst_x, inst_y, TEXT_SIZE * 0.8, color);
        inst_y += 22.0;
    }
    
    // Game statistics panel
    let stats_x = 20.0;
    let mut stats_y = BOARD_OFFSET_Y + BOARD_HEIGHT_PX - 200.0;
    
    // Stats background
    draw_rectangle(
        stats_x - 10.0,
        stats_y - 30.0,
        200.0,
        180.0,
        Color::new(0.0, 0.0, 0.0, 0.7),
    );
    
    // Stats border
    draw_rectangle_lines(
        stats_x - 10.0,
        stats_y - 30.0,
        200.0,
        180.0,
        2.0,
        UI_BORDER,
    );
    
    // Stats title
    draw_text(
        "GAME STATS",
        stats_x,
        stats_y - 10.0,
        TEXT_SIZE * 0.9,
        Color::new(1.0, 0.9, 0.7, 1.0),
    );
    stats_y += 15.0;
    
    // Individual stats
    let stats = vec![
        format!("Score: {}", game.score),
        format!("Level: {}", game.level()),
        format!("Lines: {}", game.lines_cleared()),
        format!("State: {:?}", game.state),
        format!("Time: {:.0}s", game.game_time),
    ];
    
    for stat in stats {
        draw_text(
            &stat,
            stats_x,
            stats_y,
            TEXT_SIZE * 0.75,
            Color::new(0.9, 0.9, 0.95, 0.9),
        );
        stats_y += 22.0;
    }
    
    // Current piece info
    if let Some(ref piece) = game.current_piece {
        draw_text(
            &format!("Current: {}", piece.piece_type.name()),
            stats_x,
            stats_y,
            TEXT_SIZE * 0.7,
            piece.color(),
        );
    }
    
    // Board info label
    let board_info = "Live Game Data";
    draw_text(
        board_info,
        BOARD_OFFSET_X,
        BOARD_OFFSET_Y - 15.0,
        TEXT_SIZE * 0.8,
        Color::new(0.8, 0.9, 1.0, 0.7),
    );
}

