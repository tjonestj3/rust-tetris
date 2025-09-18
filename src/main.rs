use macroquad::prelude::*;
use rust_tetris::game::config::*;
use rust_tetris::graphics::colors::*;
use rust_tetris::board::Board;
use rust_tetris::game::{Game, GameState};
use rust_tetris::tetromino::{Tetromino, TetrominoType};
use rust_tetris::audio::system::{AudioSystem, SoundType};
use rust_tetris::{MenuSystem, MenuAction};

/// Game application state
#[derive(Debug, PartialEq)]
enum AppState {
    /// In the menu system
    Menu,
    /// Playing the game
    Playing,
    /// Game over, checking for high score
    GameOver,
}

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
    
    // Initialize menu system
    let mut menu_system = MenuSystem::new();
    
    // Initialize and load audio system
    let mut audio_system = AudioSystem::new();
    if let Err(e) = audio_system.load_sounds().await {
        log::warn!("Failed to initialize audio system: {}", e);
    }
    
    // Apply audio settings
    if !menu_system.settings.sound_enabled {
        // TODO: Mute audio system based on settings
        log::info!("Audio disabled by user settings");
    }
    
    // Start background music if enabled
    if menu_system.settings.sound_enabled {
        audio_system.start_background_music();
    }
    
    // Application state management
    let mut app_state = AppState::Menu;
    let mut game: Option<Game> = None;
    let save_path = Game::default_save_path();
    
    let mut frame_count = 0u64;
    let mut last_fps_time = get_time();
    let mut fps = 0.0;
    let mut last_save_time = get_time();
    let auto_save_interval = 30.0; // Auto-save every 30 seconds
    let mut last_game_state_hash = 0u64; // Track game state changes for performance
    
    // Main application loop
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

        // Update audio system with current settings (applies to all states)
        audio_system.set_audio_enabled(menu_system.settings.sound_enabled);
        audio_system.set_master_volume(menu_system.settings.volume);
        
        match app_state {
            AppState::Menu => {
                // Update menu system
                menu_system.update(delta_time as f64);
                
                // Handle menu input
                let action = menu_system.handle_input();
                
                match action {
                    MenuAction::NewGame => {
                        log::info!("Starting new game");
                        game = Some(Game::new());
                        app_state = AppState::Playing;
                    },
                    MenuAction::LoadGame => {
                        log::info!("Loading saved game");
                        match Game::load_from_file(&save_path) {
                            Ok(loaded_game) => {
                                game = Some(loaded_game);
                                app_state = AppState::Playing;
                            },
                            Err(e) => {
                                log::warn!("Failed to load save file: {}", e);
                                // Fall back to new game
                                game = Some(Game::new());
                                app_state = AppState::Playing;
                            }
                        }
                    },
                    MenuAction::Quit => {
                        log::info!("Quitting game");
                        std::process::exit(0);
                    },
                    MenuAction::None => {
                        // Continue in menu
                    },
                }
                
                // Render menu
                menu_system.render(&background_texture);
            },
            
            AppState::Playing => {
                if let Some(ref mut current_game) = game {
                    // Handle game input
                    handle_game_input(current_game, &audio_system, &mut app_state, &mut menu_system);
                    
                    // Store previous state for audio event detection
                    let prev_score = current_game.score;
                    let prev_level = current_game.level();
                    let prev_lines_cleared = current_game.lines_cleared();
                    let was_clearing_lines = current_game.is_clearing_lines();
                    let prev_state = current_game.state;
                    
                    // Update game logic
                    current_game.update(delta_time as f64);
                    
                    // Check for game over and high score
                    if current_game.state == GameState::GameOver && prev_state != GameState::GameOver {
                        // Game just ended - check for high score
                        if menu_system.check_high_score(
                            current_game.score,
                            current_game.level(),
                            current_game.lines_cleared(),
                            current_game.game_time
                        ) {
                            app_state = AppState::GameOver;
                        } else {
                            // No high score, return to menu
                            app_state = AppState::Menu;
                        }
                    }
                    
                    // Detect and play audio for game events
                    detect_and_play_audio_events(current_game, &audio_system, prev_score, prev_level, prev_lines_cleared, was_clearing_lines, prev_state);
                    
                    // Auto-save periodically during gameplay
                    if current_game.state == GameState::Playing && current_time - last_save_time >= auto_save_interval {
                        let current_hash = current_game.get_state_hash();
                        if current_hash != last_game_state_hash {
                            // Only save if game state has actually changed
                            if let Err(e) = current_game.save_to_file(&save_path) {
                                log::warn!("Auto-save failed: {}", e);
                            } else {
                                last_game_state_hash = current_hash;
                                log::debug!("Auto-save completed (state changed)");
                            }
                        } else {
                            log::debug!("Auto-save skipped (no state change)");
                        }
                        last_save_time = current_time;
                    }
                    
                    // Render game
                    render_game(current_game, &background_texture, fps);
                } else {
                    // No game instance, return to menu
                    app_state = AppState::Menu;
                }
            },
            
            AppState::GameOver => {
                // Update menu system for name entry
                menu_system.update(delta_time as f64);
                
                // Handle name entry input
                let action = menu_system.handle_input();
                
                if action != MenuAction::None || !matches!(menu_system.state, rust_tetris::menu::MenuState::NameEntry { .. }) {
                    // Name entry complete or cancelled, return to menu
                    app_state = AppState::Menu;
                }
                
                // Render name entry screen
                menu_system.render(&background_texture);
            },
        }
        
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
        
        next_frame().await;
    }
}

/// Handle game input and transitions back to menu
fn handle_game_input(game: &mut Game, audio_system: &AudioSystem, app_state: &mut AppState, _menu_system: &mut MenuSystem) {
    // Quit to menu
    if is_key_pressed(KeyCode::Escape) {
        *app_state = AppState::Menu;
        return;
    }
    
    // Save game (S key) - available in any state
    if is_key_pressed(KeyCode::S) && is_key_down(KeyCode::LeftControl) {
        let save_path = Game::default_save_path();
        match game.save_to_file(&save_path) {
            Ok(_) => {
                log::info!("Game saved manually");
                audio_system.play_sound_with_volume(SoundType::UiClick, 1.0);
            },
            Err(e) => {
                log::warn!("Manual save failed: {}", e);
            }
        }
        return;
    }
    
    // Reset game (R key) - available in any state
    if is_key_pressed(KeyCode::R) {
        game.reset();
        audio_system.play_sound_with_volume(SoundType::UiClick, 1.0);
        return;
    }
    
    // Pause toggle (P key) - available when playing or paused
    if is_key_pressed(KeyCode::P) && (game.state == GameState::Playing || game.state == GameState::Paused) {
        game.toggle_pause();
        audio_system.play_sound(SoundType::Pause);
        return;
    }
    
    // Legacy mode toggle (L key) - available in any state except game over
    if is_key_pressed(KeyCode::L) && game.state != GameState::GameOver {
        game.toggle_legacy_mode();
        audio_system.play_sound_with_volume(SoundType::UiClick, 1.0);
        return;
    }
    
    // Only handle game controls when playing
    if game.state != GameState::Playing {
        return;
    }
    
    // Ghost block controls (available during normal play)
    if is_key_pressed(KeyCode::B) {
        if game.ghost_block_placement_mode {
            // B to place block when in placement mode
            game.place_ghost_block();
        } else {
            // B to activate ghost block placement mode
            game.toggle_ghost_block_mode();
        }
    }
    
    // Ghost block cursor movement (only when in placement mode)
    if game.ghost_block_placement_mode {
        if is_key_pressed(KeyCode::M) {
            // M for next smart position
            game.next_smart_position();
        }
        if is_key_pressed(KeyCode::N) {
            // N for previous smart position
            game.previous_smart_position();
        }
        // Also allow arrow keys for manual fine-tuning
        if is_key_pressed(KeyCode::Up) {
            game.move_ghost_block_cursor(0, -1);
        }
        if is_key_pressed(KeyCode::Down) {
            game.move_ghost_block_cursor(0, 1);
        }
        if is_key_pressed(KeyCode::Left) {
            game.move_ghost_block_cursor(-1, 0);
        }
        if is_key_pressed(KeyCode::Right) {
            game.move_ghost_block_cursor(1, 0);
        }
        return; // Skip normal game controls when in placement mode
    }
    
    // Continuous horizontal movement (Arrow keys + WASD)
    let left_held = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A);
    let right_held = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);
    
    // Play movement sound on initial press only
    if (is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A)) ||
       (is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D)) {
        audio_system.play_sound_with_volume(SoundType::UiClick, 0.6);
    }
    
    game.update_left_movement(left_held);
    game.update_right_movement(right_held);
    
    // Continuous soft drop (Down arrow + S key)
    let soft_drop_held = is_key_down(KeyCode::Down) || is_key_down(KeyCode::S);
    game.update_soft_drop(soft_drop_held);
    
    // Rotation (Up/X/W for clockwise, Z for counterclockwise)
    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::X) || is_key_pressed(KeyCode::W) {
        if game.rotate_piece_clockwise() {
            audio_system.play_sound_with_volume(SoundType::UiClick, 0.8);
        }
    }
    if is_key_pressed(KeyCode::Z) {
        if game.rotate_piece_counterclockwise() {
            audio_system.play_sound_with_volume(SoundType::UiClick, 0.8);
        }
    }
    
    // Hard drop (Space)
    if is_key_pressed(KeyCode::Space) {
        game.hard_drop();
        audio_system.play_sound(SoundType::HardDrop);
    }
    
    // Hold piece (C key)
    if is_key_pressed(KeyCode::C) {
        if game.hold_piece() {
            audio_system.play_sound(SoundType::HoldPiece);
        }
    }
}

/// Render the game state
fn render_game(game: &Game, background_texture: &Texture2D, fps: f64) {
    // Clear screen with appropriate background based on mode
    if game.is_legacy_mode() {
        // Pure black background for authentic terminal look
        clear_background(Color::new(0.0, 0.0, 0.0, 1.0));
    } else {
        // Modern background with effects
        clear_background(BACKGROUND_COLOR);
        
        // Draw background image
        draw_texture(
            background_texture,
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
    }

    // Draw Tetris board with appropriate style (legacy vs modern)
    if game.is_legacy_mode() {
        draw_legacy_board_with_data(&game.board);
    } else {
        draw_enhanced_board_with_data(&game.board);
    }
    
    // Draw line clearing animation if active
    if game.is_clearing_lines() {
        draw_line_clear_animation(&game);
    }
    
    // Draw the current falling piece (only if not clearing lines)
    if !game.is_clearing_lines() {
        // Draw ghost piece first (behind the actual piece)
        if let Some(ghost_piece) = game.calculate_ghost_piece() {
            if game.is_legacy_mode() {
                draw_legacy_ghost_piece(&ghost_piece);
            } else {
                draw_ghost_piece(&ghost_piece);
            }
        }
        
        if let Some(ref piece) = game.current_piece {
            if game.is_legacy_mode() {
                draw_legacy_falling_piece(piece);
            } else {
                draw_falling_piece(piece);
            }
        }
    }
    
    // Draw ghost block cursor if in placement mode
    if game.is_ghost_cursor_visible() {
        draw_ghost_block_cursor(&game);
    }
    
    // Draw next piece preview with appropriate style
    if game.is_legacy_mode() {
        draw_legacy_next_piece_preview(&game.next_piece);
    } else {
        draw_next_piece_preview(&game.next_piece);
    }
    
    // Draw hold piece with appropriate style
    if game.is_legacy_mode() {
        draw_legacy_hold_piece(&game.held_piece, game.can_hold());
    } else {
        draw_hold_piece(&game.held_piece, game.can_hold());
    }
    
    // Draw title with enhanced styling
    if game.is_legacy_mode() {
        draw_legacy_ui(&game);
    } else {
        draw_enhanced_ui(&game);
    }
    
    // Draw TETRIS celebration if active
    if game.is_tetris_celebration_active() {
        draw_tetris_celebration(&game);
    }
    
    // Draw ghost throw animation if active
    if game.is_ghost_throw_active() {
        draw_ghost_throw_animation(&game);
    }
    
    // Draw game state overlays
    match game.state {
        GameState::GameOver => draw_game_over_overlay(&game),
        GameState::Paused => draw_pause_overlay(&game),
        _ => {}, // No overlay for Playing or Menu
    }
    
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
}

/// Create a magical retro gaming background with Tetris theme
fn create_chess_background() -> Image {
    let width = WINDOW_WIDTH as u16;
    let height = WINDOW_HEIGHT as u16;
    let mut image = Image::gen_image_color(width, height, Color::new(0.02, 0.02, 0.08, 1.0));
    
    let center_x = width as f32 / 2.0;
    let center_y = height as f32 / 2.0;
    
    // Create magical background with multiple effects
    for y in 0..height {
        for x in 0..width {
            let fx = x as f32;
            let fy = y as f32;
            
            // Distance from center for radial effects
            let distance = ((fx - center_x).powi(2) + (fy - center_y).powi(2)).sqrt();
            let max_distance = (center_x.powi(2) + center_y.powi(2)).sqrt();
            let normalized_distance = distance / max_distance;
            
            // Create layered magical effects
            let mut final_color = Color::new(0.02, 0.02, 0.08, 1.0); // Deep space blue base
            
            // 1. Radial gradient from center (magical aura)
            let radial_intensity = (1.0 - normalized_distance * 0.7).max(0.0);
            final_color.r = (final_color.r + radial_intensity * 0.1).min(1.0);
            final_color.g = (final_color.g + radial_intensity * 0.05).min(1.0);
            final_color.b = (final_color.b + radial_intensity * 0.15).min(1.0);
            
            // 2. Animated wave patterns (simulating time with position)
            let wave1 = ((fx * 0.02 + fy * 0.01).sin() * 0.5 + 0.5) * 0.08;
            let wave2 = ((fx * 0.015 - fy * 0.02).cos() * 0.5 + 0.5) * 0.06;
            final_color.r = (final_color.r + wave1 * 0.3).min(1.0);
            final_color.g = (final_color.g + wave2 * 0.2).min(1.0);
            final_color.b = (final_color.b + (wave1 + wave2) * 0.4).min(1.0);
            
            // 3. Circuit-like grid pattern (retro gaming aesthetic)
            let grid_size = 40.0;
            let grid_x = (fx / grid_size) % 1.0;
            let grid_y = (fy / grid_size) % 1.0;
            
            // Create grid lines with glow
            if grid_x < 0.05 || grid_x > 0.95 || grid_y < 0.05 || grid_y > 0.95 {
                let grid_glow = 0.15;
                final_color.r = (final_color.r + grid_glow * 0.2).min(1.0);
                final_color.g = (final_color.g + grid_glow * 0.6).min(1.0);
                final_color.b = (final_color.b + grid_glow * 1.0).min(1.0);
            }
            
            // 4. Scattered "stars" or magical particles
            let noise_factor = ((fx * 0.1).sin() * (fy * 0.1).cos() * 1000.0) % 1.0;
            if noise_factor > 0.98 {
                let star_brightness = (noise_factor - 0.98) * 50.0;
                final_color.r = (final_color.r + star_brightness * 0.8).min(1.0);
                final_color.g = (final_color.g + star_brightness * 0.9).min(1.0);
                final_color.b = (final_color.b + star_brightness * 1.0).min(1.0);
            }
            
            // 5. Subtle Tetris block pattern in the background
            let block_size = 80.0;
            let block_x = ((fx / block_size) % 1.0 * 4.0) as i32;
            let block_y = ((fy / block_size) % 1.0 * 4.0) as i32;
            
            // Create subtle Tetris-like shapes
            let tetris_shapes = [
                // I-piece pattern
                [1, 1, 1, 1],
                // T-piece pattern  
                [0, 1, 0, 0],
                [1, 1, 1, 0],
                [0, 1, 0, 0],
            ];
            
            if block_y < 4 && block_x < 4 {
                let shape_index = ((fx / 200.0) as usize + (fy / 200.0) as usize) % tetris_shapes.len();
                if shape_index < tetris_shapes.len() && block_y < tetris_shapes.len() as i32 {
                    let shape_line = tetris_shapes[shape_index];
                    if block_x < shape_line.len() as i32 && shape_line[block_x as usize] == 1 {
                        let tetris_glow = 0.05;
                        final_color.r = (final_color.r + tetris_glow * 0.4).min(1.0);
                        final_color.g = (final_color.g + tetris_glow * 0.2).min(1.0);
                        final_color.b = (final_color.b + tetris_glow * 0.8).min(1.0);
                    }
                }
            }
            
            // 6. Vertical gradient (darker at top, lighter at bottom)
            let vertical_gradient = fy / height as f32;
            final_color.r = (final_color.r + vertical_gradient * 0.03).min(1.0);
            final_color.g = (final_color.g + vertical_gradient * 0.02).min(1.0);
            final_color.b = (final_color.b + vertical_gradient * 0.05).min(1.0);
            
            image.set_pixel(x as u32, y as u32, final_color);
        }
    }
    
    image
}

/// Handle player input with audio feedback
fn handle_input(game: &mut Game, audio_system: &AudioSystem) {
    // Quit game
    if is_key_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
    
    // Save game (S key) - available in any state
    if is_key_pressed(KeyCode::S) && is_key_down(KeyCode::LeftControl) {
        let save_path = Game::default_save_path();
        match game.save_to_file(&save_path) {
            Ok(_) => {
                log::info!("Game saved manually");
                audio_system.play_sound_with_volume(SoundType::UiClick, 1.0);
            },
            Err(e) => {
                log::warn!("Manual save failed: {}", e);
            }
        }
        return;
    }
    
    // Reset game (R key) - available in any state
    if is_key_pressed(KeyCode::R) {
        game.reset();
        audio_system.play_sound_with_volume(SoundType::UiClick, 1.0);
        return;
    }
    
    // Pause toggle (P key) - available when playing or paused
    if is_key_pressed(KeyCode::P) && (game.state == GameState::Playing || game.state == GameState::Paused) {
        game.toggle_pause();
        audio_system.play_sound(SoundType::Pause);
        return;
    }
    
    // Legacy mode toggle (L key) - available in any state except game over
    if is_key_pressed(KeyCode::L) && game.state != GameState::GameOver {
        game.toggle_legacy_mode();
        audio_system.play_sound_with_volume(SoundType::UiClick, 1.0);
        return;
    }
    
    // Only handle game controls when playing
    if game.state != GameState::Playing {
        return;
    }
    
    // Ghost block controls (available during normal play)
    if is_key_pressed(KeyCode::B) {
        if game.ghost_block_placement_mode {
            // B to place block when in placement mode
            game.place_ghost_block();
        } else {
            // B to activate ghost block placement mode
            game.toggle_ghost_block_mode();
        }
    }
    
    // Ghost block cursor movement (only when in placement mode)
    if game.ghost_block_placement_mode {
        if is_key_pressed(KeyCode::M) {
            // M for next smart position
            game.next_smart_position();
        }
        if is_key_pressed(KeyCode::N) {
            // N for previous smart position
            game.previous_smart_position();
        }
        // Also allow arrow keys for manual fine-tuning
        if is_key_pressed(KeyCode::Up) {
            game.move_ghost_block_cursor(0, -1);
        }
        if is_key_pressed(KeyCode::Down) {
            game.move_ghost_block_cursor(0, 1);
        }
        if is_key_pressed(KeyCode::Left) {
            game.move_ghost_block_cursor(-1, 0);
        }
        if is_key_pressed(KeyCode::Right) {
            game.move_ghost_block_cursor(1, 0);
        }
        return; // Skip normal game controls when in placement mode
    }
    
    // Continuous horizontal movement (Arrow keys + WASD)
    let left_held = is_key_down(KeyCode::Left) || is_key_down(KeyCode::A);
    let right_held = is_key_down(KeyCode::Right) || is_key_down(KeyCode::D);
    
    // Play movement sound on initial press only
    if (is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A)) ||
       (is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D)) {
        audio_system.play_sound_with_volume(SoundType::UiClick, 0.6);
    }
    
    game.update_left_movement(left_held);
    game.update_right_movement(right_held);
    
    // Continuous soft drop (Down arrow + S key)
    let soft_drop_held = is_key_down(KeyCode::Down) || is_key_down(KeyCode::S);
    game.update_soft_drop(soft_drop_held);
    
    // Rotation (Up/X/W for clockwise, Z for counterclockwise)
    if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::X) || is_key_pressed(KeyCode::W) {
        if game.rotate_piece_clockwise() {
            audio_system.play_sound_with_volume(SoundType::UiClick, 0.8);
        }
    }
    if is_key_pressed(KeyCode::Z) {
        if game.rotate_piece_counterclockwise() {
            audio_system.play_sound_with_volume(SoundType::UiClick, 0.8);
        }
    }
    
    // Hard drop (Space)
    if is_key_pressed(KeyCode::Space) {
        game.hard_drop();
        audio_system.play_sound(SoundType::HardDrop);
    }
    
    // Hold piece (C key)
    if is_key_pressed(KeyCode::C) {
        if game.hold_piece() {
            audio_system.play_sound(SoundType::HoldPiece);
        }
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

/// Draw the ghost piece (shadow piece showing where current piece will land)
fn draw_ghost_piece(ghost_piece: &Tetromino) {
    for (x, y) in ghost_piece.absolute_blocks() {
        // Only draw blocks that are in the visible area
        if y >= BUFFER_HEIGHT as i32 {
            let visible_y = y - BUFFER_HEIGHT as i32;
            let cell_x = BOARD_OFFSET_X + (x as f32 * CELL_SIZE);
            let cell_y = BOARD_OFFSET_Y + (visible_y as f32 * CELL_SIZE);
            
            let base_color = ghost_piece.color();
            
            // Enhanced ghost piece visibility:
            // 1. Brighter, thicker outer border for better contrast
            let outer_border_color = Color::new(1.0, 1.0, 1.0, 0.8); // Bright white border
            draw_rectangle_lines(
                cell_x + 1.0,
                cell_y + 1.0,
                CELL_SIZE - 2.0,
                CELL_SIZE - 2.0,
                3.0, // Thicker border
                outer_border_color,
            );
            
            // 2. Colored inner border using piece color with higher alpha
            let inner_border_color = Color::new(
                base_color.r,
                base_color.g,
                base_color.b,
                0.6, // More visible than before
            );
            draw_rectangle_lines(
                cell_x + 3.0,
                cell_y + 3.0,
                CELL_SIZE - 6.0,
                CELL_SIZE - 6.0,
                2.0,
                inner_border_color,
            );
            
            // 3. Subtle but more visible fill with pattern
            let fill_color = Color::new(
                (base_color.r + 0.3).min(1.0), // Brighten the fill
                (base_color.g + 0.3).min(1.0),
                (base_color.b + 0.3).min(1.0),
                0.2, // Doubled the alpha from 0.1 to 0.2
            );
            draw_rectangle(
                cell_x + 5.0,
                cell_y + 5.0,
                CELL_SIZE - 10.0,
                CELL_SIZE - 10.0,
                fill_color,
            );
            
            // 4. Add small corner dots for extra visibility
            let dot_color = Color::new(1.0, 1.0, 1.0, 0.7);
            let dot_size = 2.0;
            // Top-left corner dot
            draw_rectangle(
                cell_x + 2.0,
                cell_y + 2.0,
                dot_size,
                dot_size,
                dot_color,
            );
            // Top-right corner dot
            draw_rectangle(
                cell_x + CELL_SIZE - 4.0,
                cell_y + 2.0,
                dot_size,
                dot_size,
                dot_color,
            );
            // Bottom-left corner dot
            draw_rectangle(
                cell_x + 2.0,
                cell_y + CELL_SIZE - 4.0,
                dot_size,
                dot_size,
                dot_color,
            );
            // Bottom-right corner dot
            draw_rectangle(
                cell_x + CELL_SIZE - 4.0,
                cell_y + CELL_SIZE - 4.0,
                dot_size,
                dot_size,
                dot_color,
            );
        }
    }
}

/// Draw the ghost block cursor for placement with rainbow clockwise animation
fn draw_ghost_block_cursor(game: &Game) {
    let (cursor_x, cursor_y) = game.ghost_block_cursor;
    
    // Only draw if cursor is in visible area
    if cursor_y >= BUFFER_HEIGHT as i32 {
        let visible_y = cursor_y - BUFFER_HEIGHT as i32;
        let cell_x = BOARD_OFFSET_X + (cursor_x as f32 * CELL_SIZE);
        let cell_y = BOARD_OFFSET_Y + (visible_y as f32 * CELL_SIZE);
        
        // Draw clockwise rainbow animation around the square
        draw_rainbow_clockwise_border(cell_x, cell_y, CELL_SIZE, game.ghost_block_blink_timer);
        
        // Draw subtle inner glow (constant)
        draw_rectangle(
            cell_x + 6.0,
            cell_y + 6.0,
            CELL_SIZE - 12.0,
            CELL_SIZE - 12.0,
            Color::new(1.0, 1.0, 1.0, 0.15),
        );
    }
}

/// Draw ghost block throwing animation with character and projectile
fn draw_ghost_throw_animation(game: &Game) {
    if let Some((progress, start_pos, target_pos)) = game.get_ghost_throw_info() {
        // Animation phases
        let throw_start = 0.0;
        let throw_peak = 0.3;     // Character throwing at 30%
        let block_flight = 0.8;   // Block reaches target at 80%
        let impact_end = 1.0;     // Impact effects end at 100%
        
        // Draw throwing character (mage or simple stick figure)
        let char_x = start_pos.0;
        let char_y = start_pos.1;
        
        if progress <= throw_peak {
            // Pre-throw and throwing animation
            let throw_progress = (progress / throw_peak) as f32;
            if game.is_legacy_mode() {
                draw_legacy_stick_figure_throwing(char_x, char_y, throw_progress);
            } else {
                draw_stick_figure_throwing(char_x, char_y, throw_progress);
            }
        } else {
            // Post-throw pose
            if game.is_legacy_mode() {
                draw_legacy_stick_figure_thrown(char_x, char_y);
            } else {
                draw_stick_figure_thrown(char_x, char_y);
            }
        }
        
        // Draw flying block
        if progress >= throw_peak && progress <= block_flight {
            let flight_progress = ((progress - throw_peak) / (block_flight - throw_peak)) as f32;
            
            // Calculate parabolic trajectory
            let start_x = start_pos.0 + 30.0; // Offset from character's hand
            let start_y = start_pos.1 - 10.0;
            let end_x = target_pos.0;
            let end_y = target_pos.1;
            
            // Parabolic motion
            let current_x = start_x + (end_x - start_x) * flight_progress;
            let peak_height = 60.0; // Height of arc
            let arc_progress = flight_progress * (1.0 - flight_progress) * 4.0; // Peaks at 0.5 progress
            let current_y = start_y + (end_y - start_y) * flight_progress - peak_height * arc_progress;
            
            // Draw spinning block with trail
            let rotation = flight_progress * 6.28 * 3.0; // 3 full rotations
            if game.is_legacy_mode() {
                draw_legacy_spinning_ghost_block(current_x, current_y, rotation, flight_progress);
                // Draw legacy motion trail
                draw_legacy_block_trail(start_x, start_y, current_x, current_y, flight_progress);
            } else {
                draw_spinning_ghost_block(current_x, current_y, rotation, flight_progress);
                // Draw motion trail
                draw_block_trail(start_x, start_y, current_x, current_y, flight_progress);
            }
        }
        
        // Draw impact effects
        if progress >= block_flight {
            let impact_progress = ((progress - block_flight) / (impact_end - block_flight)) as f32;
            if game.is_legacy_mode() {
                draw_legacy_impact_effects(target_pos.0, target_pos.1, impact_progress);
            } else {
                draw_impact_effects(target_pos.0, target_pos.1, impact_progress);
            }
        }
    }
}

/// Draw magical mage in spell-casting pose
fn draw_stick_figure_throwing(x: f32, y: f32, progress: f32) {
    let skin_color = Color::new(0.95, 0.87, 0.73, 0.9); // Warm skin tone
    let robe_color = Color::new(0.2, 0.1, 0.6, 0.9);   // Deep purple robe
    let staff_color = Color::new(0.6, 0.4, 0.2, 0.9);  // Brown wooden staff
    let magic_color = Color::new(0.8, 0.9, 1.0, 0.8);  // Bright magical energy
    let line_width = 3.0;
    
    // Animate spell-casting motion
    let spell_progress = progress * 1.8; // More dramatic casting motion
    let body_lean = progress * 0.2;      // Slight forward lean
    let magic_intensity = (progress * 3.14).sin().max(0.0); // Pulsing magic
    
    // Draw flowing robe (wider base)
    let robe_width = 25.0 + progress * 5.0; // Robe billows during cast
    let body_center_x = x + body_lean * 10.0;
    let body_center_y = y + 15.0;
    
    // Robe body (triangle shape for flowing effect)
    let robe_points = [
        (body_center_x - robe_width / 2.0, body_center_y + 40.0), // Bottom left
        (body_center_x + robe_width / 2.0, body_center_y + 40.0), // Bottom right
        (body_center_x, body_center_y - 10.0),                     // Top center
    ];
    
    // Draw robe fill
    for i in 0..3 {
        let p1 = robe_points[i];
        let p2 = robe_points[(i + 1) % 3];
        draw_line(p1.0, p1.1, p2.0, p2.1, 8.0, robe_color);
    }
    
    // Head with pointed wizard hat
    draw_circle(body_center_x, y - 8.0, 7.0, skin_color);
    
    // Wizard hat (triangle)
    let hat_points = [
        (body_center_x - 8.0, y - 15.0),  // Left base
        (body_center_x + 8.0, y - 15.0),  // Right base  
        (body_center_x + 3.0, y - 35.0),  // Pointed tip (slightly off-center)
    ];
    
    for i in 0..3 {
        let p1 = hat_points[i];
        let p2 = hat_points[(i + 1) % 3];
        draw_line(p1.0, p1.1, p2.0, p2.1, 4.0, robe_color);
    }
    
    // Magical staff in non-casting hand
    let staff_x = body_center_x - 20.0;
    let staff_y1 = body_center_y - 5.0;
    let staff_y2 = staff_y1 + 35.0;
    
    // Staff shaft
    draw_line(staff_x, staff_y1, staff_x, staff_y2, 4.0, staff_color);
    
    // Magical orb at top of staff
    draw_circle(staff_x, staff_y1 - 8.0, 5.0, magic_color);
    
    // Pulsing magic aura around orb
    if magic_intensity > 0.3 {
        let aura_size = 8.0 + magic_intensity * 4.0;
        draw_circle_lines(staff_x, staff_y1 - 8.0, aura_size, 2.0, 
                         Color::new(magic_color.r, magic_color.g, magic_color.b, magic_intensity * 0.6));
    }
    
    // Casting arm (extended forward with magical energy)
    let cast_arm_x = body_center_x + spell_progress.cos() * 25.0;
    let cast_arm_y = body_center_y - 5.0 + spell_progress.sin() * 12.0;
    
    // Arm to casting position
    draw_line(body_center_x + 5.0, body_center_y - 5.0, cast_arm_x, cast_arm_y, line_width, skin_color);
    
    // Magical energy swirling around casting hand
    if progress > 0.2 {
        let swirl_count = 5;
        for i in 0..swirl_count {
            let swirl_angle = (progress * 6.28 * 2.0) + (i as f32 * 1.256); // Different phase for each swirl
            let swirl_radius = 8.0 + (i as f32 * 2.0);
            let swirl_x = cast_arm_x + swirl_angle.cos() * swirl_radius;
            let swirl_y = cast_arm_y + swirl_angle.sin() * swirl_radius * 0.5;
            
            let swirl_alpha = magic_intensity * (1.0 - i as f32 * 0.2);
            draw_circle(swirl_x, swirl_y, 2.0, Color::new(magic_color.r, magic_color.g, magic_color.b, swirl_alpha));
        }
    }
    
    // Magical runes floating around mage
    if progress > 0.4 {
        let rune_positions = [
            (body_center_x - 30.0, body_center_y - 20.0),
            (body_center_x + 25.0, body_center_y - 15.0),
            (body_center_x - 15.0, body_center_y - 30.0),
        ];
        
        for (i, &(rune_x, rune_y)) in rune_positions.iter().enumerate() {
            let rune_progress = (progress - 0.4) * 2.0; // Start appearing at 40% progress
            let float_offset = ((progress * 4.0 + i as f32).sin() * 3.0) as f32;
            let rune_alpha = (rune_progress * magic_intensity).min(0.8);
            
            if rune_alpha > 0.1 {
                // Simple rune symbols (just geometric shapes)
                match i {
                    0 => { // Circle rune
                        draw_circle_lines(rune_x, rune_y + float_offset, 4.0, 2.0, 
                                        Color::new(1.0, 0.8, 0.2, rune_alpha));
                    },
                    1 => { // Triangle rune
                        let size = 4.0;
                        draw_line(rune_x - size, rune_y + size + float_offset, 
                                rune_x + size, rune_y + size + float_offset, 2.0, 
                                Color::new(0.2, 0.8, 1.0, rune_alpha));
                        draw_line(rune_x + size, rune_y + size + float_offset,
                                rune_x, rune_y - size + float_offset, 2.0,
                                Color::new(0.2, 0.8, 1.0, rune_alpha));
                        draw_line(rune_x, rune_y - size + float_offset,
                                rune_x - size, rune_y + size + float_offset, 2.0,
                                Color::new(0.2, 0.8, 1.0, rune_alpha));
                    },
                    _ => { // Diamond rune
                        let size = 3.0;
                        draw_line(rune_x, rune_y - size + float_offset,
                                rune_x + size, rune_y + float_offset, 2.0,
                                Color::new(0.8, 0.2, 0.8, rune_alpha));
                        draw_line(rune_x + size, rune_y + float_offset,
                                rune_x, rune_y + size + float_offset, 2.0,
                                Color::new(0.8, 0.2, 0.8, rune_alpha));
                        draw_line(rune_x, rune_y + size + float_offset,
                                rune_x - size, rune_y + float_offset, 2.0,
                                Color::new(0.8, 0.2, 0.8, rune_alpha));
                        draw_line(rune_x - size, rune_y + float_offset,
                                rune_x, rune_y - size + float_offset, 2.0,
                                Color::new(0.8, 0.2, 0.8, rune_alpha));
                    }
                }
            }
        }
    }
}

/// Draw mage completing the spell (post-cast pose)
fn draw_stick_figure_thrown(x: f32, y: f32) {
    let skin_color = Color::new(0.95, 0.87, 0.73, 0.7); // Slightly faded warm skin
    let robe_color = Color::new(0.2, 0.1, 0.6, 0.7);   // Faded deep purple robe
    let staff_color = Color::new(0.6, 0.4, 0.2, 0.7);  // Faded brown wooden staff
    let magic_color = Color::new(0.8, 0.9, 1.0, 0.5);  // Faded magical energy
    
    // Mage in post-spell completion pose (more relaxed)
    let body_center_x = x + 15.0;
    let body_center_y = y + 20.0;
    
    // Robe (slightly more relaxed shape)
    let robe_width = 22.0;
    let robe_points = [
        (body_center_x - robe_width / 2.0, body_center_y + 35.0), // Bottom left
        (body_center_x + robe_width / 2.0, body_center_y + 35.0), // Bottom right
        (body_center_x, body_center_y - 5.0),                      // Top center
    ];
    
    // Draw robe outline
    for i in 0..3 {
        let p1 = robe_points[i];
        let p2 = robe_points[(i + 1) % 3];
        draw_line(p1.0, p1.1, p2.0, p2.1, 6.0, robe_color);
    }
    
    // Head with wizard hat
    draw_circle(body_center_x, y - 5.0, 6.0, skin_color);
    
    // Wizard hat (relaxed position)
    let hat_points = [
        (body_center_x - 7.0, y - 12.0),  // Left base
        (body_center_x + 7.0, y - 12.0),  // Right base  
        (body_center_x + 2.0, y - 28.0),  // Pointed tip
    ];
    
    for i in 0..3 {
        let p1 = hat_points[i];
        let p2 = hat_points[(i + 1) % 3];
        draw_line(p1.0, p1.1, p2.0, p2.1, 3.0, robe_color);
    }
    
    // Staff in left hand (still held)
    let staff_x = body_center_x - 18.0;
    let staff_y1 = body_center_y;
    let staff_y2 = staff_y1 + 30.0;
    
    // Staff shaft
    draw_line(staff_x, staff_y1, staff_x, staff_y2, 3.0, staff_color);
    
    // Dimmed magical orb (spell complete)
    draw_circle(staff_x, staff_y1 - 6.0, 4.0, Color::new(magic_color.r, magic_color.g, magic_color.b, 0.3));
    
    // Extended casting arm (follow-through)
    let cast_arm_x = body_center_x + 25.0;
    let cast_arm_y = body_center_y;
    
    draw_line(body_center_x + 3.0, body_center_y, cast_arm_x, cast_arm_y, 2.5, skin_color);
    
    // Residual magical sparkles fading away
    let sparkle_positions = [
        (cast_arm_x + 5.0, cast_arm_y - 3.0),
        (cast_arm_x + 8.0, cast_arm_y + 2.0),
        (cast_arm_x + 3.0, cast_arm_y + 5.0),
    ];
    
    for &(spark_x, spark_y) in &sparkle_positions {
        draw_circle(spark_x, spark_y, 1.5, Color::new(1.0, 1.0, 0.8, 0.4));
    }
    
    // Faint magical aura still emanating (spell aftermath)
    draw_circle_lines(body_center_x, body_center_y, 25.0, 1.0, 
                     Color::new(magic_color.r, magic_color.g, magic_color.b, 0.2));
}

/// Draw legacy terminal-style mage casting spell using ASCII blocks and green colors
fn draw_legacy_stick_figure_throwing(x: f32, y: f32, progress: f32) {
    let terminal_green = Color::new(0.0, 1.0, 0.0, 1.0); // Bright terminal green
    let dim_green = Color::new(0.0, 0.8, 0.0, 0.8);     // Dimmed terminal green
    let block_char = "█"; // ASCII block character
    
    // Simple ASCII art mage using blocks
    let body_center_x = x;
    let body_center_y = y + 15.0;
    let block_size = 8.0; // Size of each ASCII block
    
    // Wizard hat (using ASCII blocks in a triangle pattern)
    draw_text(block_char, body_center_x - 4.0, y - 20.0, block_size, terminal_green);
    draw_text(block_char, body_center_x + 4.0, y - 20.0, block_size, terminal_green);
    draw_text(block_char, body_center_x, y - 28.0, block_size, terminal_green);
    
    // Head
    draw_text(block_char, body_center_x, y - 8.0, block_size, terminal_green);
    
    // Body (robe shape using blocks)
    draw_text(block_char, body_center_x, body_center_y, block_size, terminal_green);
    draw_text(block_char, body_center_x - 8.0, body_center_y + 8.0, block_size, terminal_green);
    draw_text(block_char, body_center_x, body_center_y + 8.0, block_size, terminal_green);
    draw_text(block_char, body_center_x + 8.0, body_center_y + 8.0, block_size, terminal_green);
    draw_text(block_char, body_center_x - 16.0, body_center_y + 16.0, block_size, terminal_green);
    draw_text(block_char, body_center_x - 8.0, body_center_y + 16.0, block_size, terminal_green);
    draw_text(block_char, body_center_x, body_center_y + 16.0, block_size, terminal_green);
    draw_text(block_char, body_center_x + 8.0, body_center_y + 16.0, block_size, terminal_green);
    draw_text(block_char, body_center_x + 16.0, body_center_y + 16.0, block_size, terminal_green);
    
    // Staff (vertical line of blocks)
    let staff_x = body_center_x - 20.0;
    for i in 0..4 {
        draw_text(block_char, staff_x, body_center_y + (i as f32 * 8.0), block_size, terminal_green);
    }
    
    // Magical orb at staff top (blinking based on progress)
    if (progress * 10.0).sin() > 0.0 {
        draw_text(block_char, staff_x, body_center_y - 8.0, block_size, terminal_green);
    }
    
    // Casting arm (extended blocks)
    let arm_extend = progress * 20.0;
    draw_text(block_char, body_center_x + 8.0 + arm_extend, body_center_y, block_size, terminal_green);
    draw_text(block_char, body_center_x + 16.0 + arm_extend, body_center_y, block_size, terminal_green);
    
    // Magical energy around casting hand (pulsing blocks)
    if progress > 0.3 {
        let pulse = (progress * 8.0).sin().abs();
        if pulse > 0.5 {
            draw_text(block_char, body_center_x + 24.0 + arm_extend, body_center_y - 8.0, block_size, dim_green);
            draw_text(block_char, body_center_x + 24.0 + arm_extend, body_center_y + 8.0, block_size, dim_green);
        }
    }
}

/// Draw legacy terminal-style mage in post-cast pose
fn draw_legacy_stick_figure_thrown(x: f32, y: f32) {
    let terminal_green = Color::new(0.0, 1.0, 0.0, 0.8); // Slightly dimmed after casting
    let block_char = "█";
    
    let body_center_x = x + 15.0;
    let body_center_y = y + 20.0;
    let block_size = 8.0;
    
    // Wizard hat
    draw_text(block_char, body_center_x - 4.0, y - 15.0, block_size, terminal_green);
    draw_text(block_char, body_center_x + 4.0, y - 15.0, block_size, terminal_green);
    draw_text(block_char, body_center_x, y - 23.0, block_size, terminal_green);
    
    // Head
    draw_text(block_char, body_center_x, y - 3.0, block_size, terminal_green);
    
    // Body (relaxed robe)
    draw_text(block_char, body_center_x, body_center_y, block_size, terminal_green);
    draw_text(block_char, body_center_x - 8.0, body_center_y + 8.0, block_size, terminal_green);
    draw_text(block_char, body_center_x, body_center_y + 8.0, block_size, terminal_green);
    draw_text(block_char, body_center_x + 8.0, body_center_y + 8.0, block_size, terminal_green);
    draw_text(block_char, body_center_x - 12.0, body_center_y + 16.0, block_size, terminal_green);
    draw_text(block_char, body_center_x - 4.0, body_center_y + 16.0, block_size, terminal_green);
    draw_text(block_char, body_center_x + 4.0, body_center_y + 16.0, block_size, terminal_green);
    draw_text(block_char, body_center_x + 12.0, body_center_y + 16.0, block_size, terminal_green);
    
    // Staff
    let staff_x = body_center_x - 16.0;
    for i in 0..4 {
        draw_text(block_char, staff_x, body_center_y + (i as f32 * 8.0), block_size, terminal_green);
    }
    
    // Dimmed orb (spell complete)
    draw_text(block_char, staff_x, body_center_y - 8.0, block_size, Color::new(0.0, 0.6, 0.0, 0.5));
    
    // Extended arm (follow-through)
    draw_text(block_char, body_center_x + 8.0, body_center_y, block_size, terminal_green);
    draw_text(block_char, body_center_x + 16.0, body_center_y, block_size, terminal_green);
    draw_text(block_char, body_center_x + 24.0, body_center_y, block_size, terminal_green);
}

/// Draw spinning fireball projectile with magical flame effects
fn draw_spinning_ghost_block(x: f32, y: f32, rotation: f32, progress: f32) {
    let base_size = 16.0;
    let alpha = 1.0 - progress * 0.2; // Keep fireball bright during flight
    let intensity = (rotation * 2.0).sin() * 0.3 + 0.7; // Pulsing intensity
    
    // Outer flame ring (orange-red)
    let outer_size = base_size * 1.8 * intensity;
    let outer_color = Color::new(1.0, 0.4, 0.1, alpha * 0.6);
    draw_circle(x, y, outer_size, outer_color);
    
    // Middle flame layer (bright orange)
    let mid_size = base_size * 1.3 * intensity;
    let mid_color = Color::new(1.0, 0.6, 0.1, alpha * 0.8);
    draw_circle(x, y, mid_size, mid_color);
    
    // Inner flame core (yellow-white)
    let core_size = base_size * 0.8;
    let core_color = Color::new(1.0, 0.9, 0.3, alpha);
    draw_circle(x, y, core_size, core_color);
    
    // Central blazing core (bright white-yellow)
    let center_size = base_size * 0.4;
    let center_color = Color::new(1.0, 1.0, 0.8, alpha);
    draw_circle(x, y, center_size, center_color);
    
    // Flame particles swirling around
    let particle_count = 8;
    for i in 0..particle_count {
        let angle = rotation * 3.0 + (i as f32 * 0.785); // Faster spinning particles
        let distance = base_size * 1.2 + (angle * 1.5).sin() * 4.0; // Varying distance
        let particle_x = x + angle.cos() * distance;
        let particle_y = y + angle.sin() * distance;
        
        // Particle size varies with position
        let particle_size = 2.0 + (angle * 2.0).sin().abs() * 2.0;
        
        // Particle color transitions from red to yellow
        let color_phase = i as f32 / particle_count as f32;
        let particle_color = Color::new(
            1.0,
            0.3 + color_phase * 0.6, // Red to yellow transition
            0.1 + color_phase * 0.2, // Slight blue for orange effect
            alpha * (0.5 + (angle).sin().abs() * 0.5),
        );
        
        draw_circle(particle_x, particle_y, particle_size, particle_color);
    }
    
    // Trailing flame wisps
    let wisp_count = 6;
    for i in 0..wisp_count {
        let wisp_angle = rotation + (i as f32 * 1.047); // Different from particles
        let wisp_distance = base_size * 2.0;
        let trail_offset = -(i as f32) * 2.0; // Trail behind
        
        let wisp_x = x + wisp_angle.cos() * wisp_distance + trail_offset;
        let wisp_y = y + wisp_angle.sin() * wisp_distance;
        
        let wisp_size = 3.0 - (i as f32 * 0.3); // Smaller as they trail
        let wisp_alpha = alpha * (1.0 - i as f32 * 0.15); // Fade as they trail
        
        let wisp_color = Color::new(1.0, 0.5, 0.1, wisp_alpha * 0.4);
        draw_circle(wisp_x, wisp_y, wisp_size, wisp_color);
    }
    
    // Heat distortion effect (subtle rings)
    for ring in 0..3 {
        let ring_size = base_size * (1.5 + ring as f32 * 0.5) * intensity;
        let ring_alpha = alpha * 0.1 * (1.0 - ring as f32 * 0.3);
        let ring_color = Color::new(1.0, 0.8, 0.6, ring_alpha);
        
        draw_circle_lines(x, y, ring_size, 1.0, ring_color);
    }
    
    // Magical sparkles around the fireball
    let sparkle_count = 12;
    for i in 0..sparkle_count {
        let sparkle_angle = rotation * 2.0 + (i as f32 * 0.524); // Different rotation speed
        let sparkle_distance = base_size * 2.5 + (sparkle_angle * 3.0).sin() * 8.0;
        let sparkle_x = x + sparkle_angle.cos() * sparkle_distance;
        let sparkle_y = y + sparkle_angle.sin() * sparkle_distance;
        
        // Twinkling effect
        let twinkle = (rotation * 5.0 + i as f32).sin().abs();
        if twinkle > 0.7 {
            let sparkle_size = 1.5 + twinkle * 1.5;
            let sparkle_color = Color::new(1.0, 1.0, 0.8, alpha * twinkle * 0.8);
            draw_circle(sparkle_x, sparkle_y, sparkle_size, sparkle_color);
        }
    }
}

/// Draw motion trail behind the flying block
fn draw_block_trail(start_x: f32, start_y: f32, current_x: f32, current_y: f32, progress: f32) {
    let trail_segments = 8;
    
    for i in 0..trail_segments {
        let segment_progress = i as f32 / trail_segments as f32;
        let trail_progress = progress - segment_progress * 0.3;
        
        if trail_progress > 0.0 {
            let seg_x = start_x + (current_x - start_x) * trail_progress;
            let seg_y = start_y + (current_y - start_y) * trail_progress;
            
            let alpha = (1.0 - segment_progress) * 0.5;
            let size = 8.0 * (1.0 - segment_progress * 0.7);
            
            draw_circle(
                seg_x, 
                seg_y, 
                size, 
                Color::new(0.8, 0.8, 1.0, alpha)
            );
        }
    }
}

/// Draw impact effects when block reaches target
fn draw_impact_effects(x: f32, y: f32, progress: f32) {
    if progress <= 1.0 {
        // Expanding impact rings
        for i in 0..3 {
            let ring_delay = i as f32 * 0.2;
            let ring_progress = (progress - ring_delay).max(0.0);
            
            if ring_progress > 0.0 {
                let radius = ring_progress * 30.0;
                let alpha = (1.0 - ring_progress) * 0.6;
                
                draw_circle_lines(
                    x, y, radius, 3.0,
                    Color::new(0.8, 0.8, 1.0, alpha)
                );
            }
        }
        
        // Particle burst
        let particle_count = 12;
        for i in 0..particle_count {
            let angle = (i as f32 / particle_count as f32) * 6.28;
            let distance = progress * 25.0;
            let particle_x = x + angle.cos() * distance;
            let particle_y = y + angle.sin() * distance;
            
            let alpha = (1.0 - progress) * 0.8;
            let size = 3.0 * (1.0 - progress * 0.5);
            
            draw_circle(
                particle_x, 
                particle_y, 
                size,
                Color::new(1.0, 1.0, 0.8, alpha)
            );
        }
        
        // Central flash
        if progress <= 0.3 {
            let flash_alpha = (1.0 - progress / 0.3) * 0.8;
            draw_circle(x, y, 20.0, Color::new(1.0, 1.0, 1.0, flash_alpha));
        }
    }
}

/// Draw legacy terminal-style spinning ghost block as a simple green ASCII block
fn draw_legacy_spinning_ghost_block(x: f32, y: f32, rotation: f32, progress: f32) {
    let terminal_green = Color::new(0.0, 1.0, 0.0, 1.0 - progress * 0.2);
    let block_char = "█";
    let block_size = 16.0;
    
    // Simple pulsing effect based on rotation
    let pulse = (rotation * 2.0).sin().abs();
    let alpha_modifier = 0.8 + pulse * 0.2;
    let final_color = Color::new(0.0, 1.0, 0.0, alpha_modifier * (1.0 - progress * 0.2));
    
    // Draw the main block
    draw_text(block_char, x - block_size / 2.0, y + block_size / 2.0, block_size, final_color);
    
    // Add a few trailing blocks for motion effect
    if rotation > 1.0 {
        // Trail blocks with decreasing alpha
        let dim_green = Color::new(0.0, 0.8, 0.0, (1.0 - progress * 0.3) * 0.6);
        let very_dim_green = Color::new(0.0, 0.6, 0.0, (1.0 - progress * 0.4) * 0.4);
        
        // Calculate trail positions based on rotation
        let trail_distance = 12.0;
        let trail_x1 = x - rotation.cos() * trail_distance;
        let trail_y1 = y - rotation.sin() * trail_distance;
        let trail_x2 = x - rotation.cos() * trail_distance * 1.5;
        let trail_y2 = y - rotation.sin() * trail_distance * 1.5;
        
        draw_text(block_char, trail_x1 - block_size / 2.0, trail_y1 + block_size / 2.0, block_size * 0.8, dim_green);
        draw_text(block_char, trail_x2 - block_size / 2.0, trail_y2 + block_size / 2.0, block_size * 0.6, very_dim_green);
    }
}

/// Draw legacy terminal-style motion trail using green blocks
fn draw_legacy_block_trail(start_x: f32, start_y: f32, current_x: f32, current_y: f32, progress: f32) {
    let trail_segments = 6;
    let block_char = "█";
    
    for i in 0..trail_segments {
        let segment_progress = i as f32 / trail_segments as f32;
        let trail_progress = progress - segment_progress * 0.4;
        
        if trail_progress > 0.0 {
            let seg_x = start_x + (current_x - start_x) * trail_progress;
            let seg_y = start_y + (current_y - start_y) * trail_progress;
            
            let alpha = (1.0 - segment_progress) * 0.6 * (1.0 - progress * 0.3);
            let size = 12.0 * (1.0 - segment_progress * 0.5);
            let trail_color = Color::new(0.0, 0.8, 0.0, alpha);
            
            draw_text(block_char, seg_x - size / 2.0, seg_y + size / 2.0, size, trail_color);
        }
    }
}

/// Draw legacy terminal-style impact effects using ASCII blocks
fn draw_legacy_impact_effects(x: f32, y: f32, progress: f32) {
    if progress <= 1.0 {
        let terminal_green = Color::new(0.0, 1.0, 0.0, 1.0 - progress);
        let dim_green = Color::new(0.0, 0.8, 0.0, (1.0 - progress) * 0.8);
        let block_char = "█";
        let block_size = 12.0;
        
        // Expanding impact pattern using blocks
        let impact_radius = progress * 40.0;
        let positions = [
            // Central impact
            (0.0, 0.0),
            // Cardinal directions
            (impact_radius, 0.0),
            (-impact_radius, 0.0),
            (0.0, impact_radius),
            (0.0, -impact_radius),
            // Diagonals
            (impact_radius * 0.7, impact_radius * 0.7),
            (-impact_radius * 0.7, impact_radius * 0.7),
            (impact_radius * 0.7, -impact_radius * 0.7),
            (-impact_radius * 0.7, -impact_radius * 0.7),
        ];
        
        for (i, &(offset_x, offset_y)) in positions.iter().enumerate() {
            let block_x = x + offset_x;
            let block_y = y + offset_y;
            
            // Central block is brightest, others fade
            let color = if i == 0 {
                terminal_green
            } else {
                dim_green
            };
            
            // Only draw blocks that are close enough to show impact
            if offset_x.abs() + offset_y.abs() <= impact_radius * 1.2 {
                draw_text(block_char, block_x - block_size / 2.0, block_y + block_size / 2.0, block_size, color);
            }
        }
        
        // Additional scattered blocks for more dramatic effect
        if progress > 0.3 {
            let scatter_positions = [
                (impact_radius * 1.2, impact_radius * 0.3),
                (-impact_radius * 1.1, -impact_radius * 0.4),
                (impact_radius * 0.4, -impact_radius * 1.3),
                (-impact_radius * 0.6, impact_radius * 1.1),
            ];
            
            let scatter_alpha = (1.0 - progress) * 0.6;
            let scatter_color = Color::new(0.0, 0.7, 0.0, scatter_alpha);
            
            for &(scatter_x, scatter_y) in scatter_positions.iter() {
                let block_x = x + scatter_x;
                let block_y = y + scatter_y;
                draw_text(block_char, block_x - block_size / 2.0, block_y + block_size / 2.0, block_size * 0.8, scatter_color);
            }
        }
    }
}

/// Draw a rainbow border that travels clockwise around a square
fn draw_rainbow_clockwise_border(x: f32, y: f32, size: f32, time: f64) {
    let border_width = 3.0;
    let segments_per_side = 8; // Number of color segments per side
    let total_segments = segments_per_side * 4; // 4 sides
    let segment_length = size / segments_per_side as f32;
    
    // Animation speed - how fast the rainbow travels
    let animation_speed = 4.0;
    let time_offset = (time * animation_speed) % (total_segments as f64);
    
    // Draw each segment of the border
    for i in 0..total_segments {
        let progress = i as f64;
        let animated_progress = (progress + time_offset) % (total_segments as f64);
        
        // Create rainbow color based on animated position
        let hue = (animated_progress / total_segments as f64) * 6.0; // 0-6 for full rainbow
        let rainbow_color = hsv_to_rgb(hue, 1.0, 1.0);
        
        // Calculate position around the perimeter clockwise
        let (seg_x, seg_y, seg_width, seg_height) = match i / segments_per_side {
            // Top side (left to right)
            0 => {
                let segment_x = x + (i % segments_per_side) as f32 * segment_length;
                (segment_x, y - border_width, segment_length, border_width)
            },
            // Right side (top to bottom)  
            1 => {
                let segment_y = y + (i % segments_per_side) as f32 * segment_length;
                (x + size, segment_y, border_width, segment_length)
            },
            // Bottom side (right to left)
            2 => {
                let segment_x = x + size - (i % segments_per_side + 1) as f32 * segment_length;
                (segment_x, y + size, segment_length, border_width)
            },
            // Left side (bottom to top)
            _ => {
                let segment_y = y + size - (i % segments_per_side + 1) as f32 * segment_length;
                (x - border_width, segment_y, border_width, segment_length)
            }
        };
        
        // Use full vibrant rainbow colors
        let final_color = Color::new(
            rainbow_color.r,
            rainbow_color.g, 
            rainbow_color.b,
            0.95  // Slightly transparent for nice blending
        );
        
        draw_rectangle(seg_x, seg_y, seg_width, seg_height, final_color);
    }
}

/// Convert HSV to RGB color
fn hsv_to_rgb(h: f64, s: f64, v: f64) -> Color {
    let c = v * s;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = v - c;
    
    let (r_prime, g_prime, b_prime) = if h < 1.0 {
        (c, x, 0.0)
    } else if h < 2.0 {
        (x, c, 0.0)
    } else if h < 3.0 {
        (0.0, c, x)
    } else if h < 4.0 {
        (0.0, x, c)
    } else if h < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    
    Color::new(
        (r_prime + m) as f32,
        (g_prime + m) as f32,
        (b_prime + m) as f32,
        1.0
    )
}

/// Draw enhanced line clearing animation with multiple effects
fn draw_line_clear_animation(game: &Game) {
    let progress = game.get_clear_animation_progress();
    let clearing_lines = game.get_clearing_lines();
    
    for (line_idx, &line_y) in clearing_lines.iter().enumerate() {
        // Only animate lines in visible area
        if line_y >= BUFFER_HEIGHT {
            let visible_y = line_y - BUFFER_HEIGHT;
            let anim_y = BOARD_OFFSET_Y + (visible_y as f32 * CELL_SIZE);
            
            // Phase 1: Initial flash with expanding energy wave (0.0 - 0.3)
            if progress <= 0.3 {
                let phase_progress = (progress / 0.3) as f32;
                
                // Bright energy flash
                let flash_intensity = (1.0 - phase_progress) * 0.9;
                let energy_color = Color::new(1.0, 0.9, 0.3, flash_intensity);
                
                draw_rectangle(
                    BOARD_OFFSET_X,
                    anim_y,
                    BOARD_WIDTH_PX,
                    CELL_SIZE,
                    energy_color,
                );
                
                // Expanding wave effect from center
                let wave_width = phase_progress * BOARD_WIDTH_PX;
                let wave_center = BOARD_OFFSET_X + BOARD_WIDTH_PX / 2.0;
                let wave_color = Color::new(0.3, 0.8, 1.0, (1.0 - phase_progress) * 0.6);
                
                draw_rectangle(
                    wave_center - wave_width / 2.0,
                    anim_y - 2.0,
                    wave_width,
                    CELL_SIZE + 4.0,
                    wave_color,
                );
            }
            
            // Phase 2: Particle disintegration effect (0.3 - 0.8)
            else if progress <= 0.8 {
                let phase_progress = ((progress - 0.3) / 0.5) as f32;
                
                // Simulate blocks breaking apart into particles
                for i in 0..BOARD_WIDTH {
                    let base_x = BOARD_OFFSET_X + (i as f32 * CELL_SIZE);
                    
                    // Multiple particles per cell
                    for particle_idx in 0..4 {
                        let particle_offset_x = (particle_idx % 2) as f32 * CELL_SIZE / 2.0;
                        let particle_offset_y = (particle_idx / 2) as f32 * CELL_SIZE / 2.0;
                        
                        let particle_x = base_x + particle_offset_x + CELL_SIZE / 4.0;
                        let particle_y = anim_y + particle_offset_y + CELL_SIZE / 4.0;
                        
                        // Add some randomness based on position
                        let seed = (line_idx + i + particle_idx) as f32 * 0.1;
                        let drift_x = seed.sin() * phase_progress * 20.0;
                        let drift_y = (seed.cos() * phase_progress * 15.0) + (phase_progress * phase_progress * 30.0);
                        
                        let final_x = particle_x + drift_x;
                        let final_y = particle_y + drift_y;
                        
                        // Particle size shrinks over time
                        let particle_size = CELL_SIZE / 4.0 * (1.0 - phase_progress * 0.7);
                        
                        // Color fades from original to orange/red
                        let fade_alpha = 1.0 - phase_progress;
                        let heat_intensity = phase_progress;
                        let particle_color = Color::new(
                            1.0,
                            1.0 - heat_intensity * 0.5,
                            0.3 * (1.0 - heat_intensity),
                            fade_alpha * 0.8,
                        );
                        
                        draw_rectangle(
                            final_x - particle_size / 2.0,
                            final_y - particle_size / 2.0,
                            particle_size,
                            particle_size,
                            particle_color,
                        );
                        
                        // Add glow effect
                        if particle_size > 2.0 {
                            draw_rectangle(
                                final_x - particle_size / 4.0,
                                final_y - particle_size / 4.0,
                                particle_size / 2.0,
                                particle_size / 2.0,
                                Color::new(1.0, 1.0, 0.8, fade_alpha * 0.4),
                            );
                        }
                    }
                }
            }
            
            // Phase 3: Final sparkle fade out (0.8 - 1.0)
            else {
                let phase_progress = ((progress - 0.8) / 0.2) as f32;
                
                // Residual sparkles
                for i in 0..BOARD_WIDTH * 2 {
                    let sparkle_x = BOARD_OFFSET_X + (i as f32 * CELL_SIZE / 2.0);
                    let sparkle_y = anim_y + CELL_SIZE / 2.0;
                    
                    let sparkle_seed = (line_idx + i) as f64 * 0.7 + progress * 8.0;
                    let sparkle_alpha = (sparkle_seed.sin() * 0.5 + 0.5) as f32 * (1.0 - phase_progress);
                    
                    if sparkle_alpha > 0.3 {
                        let sparkle_size = 2.0 + sparkle_alpha * 3.0;
                        draw_rectangle(
                            sparkle_x - sparkle_size / 2.0,
                            sparkle_y - sparkle_size / 2.0,
                            sparkle_size,
                            sparkle_size,
                            Color::new(1.0, 1.0, 0.9, sparkle_alpha * 0.6),
                        );
                    }
                }
            }
            
            // Add screen-shake effect visualization (subtle border pulse)
            if progress <= 0.4 {
                let shake_intensity = ((1.0 - progress / 0.4) * 2.0) as f32;
                let border_color = Color::new(1.0, 0.8, 0.2, shake_intensity * 0.3);
                
                draw_rectangle_lines(
                    BOARD_OFFSET_X - shake_intensity,
                    anim_y - shake_intensity,
                    BOARD_WIDTH_PX + shake_intensity * 2.0,
                    CELL_SIZE + shake_intensity * 2.0,
                    shake_intensity.max(1.0),
                    border_color,
                );
            }
        }
    }
}

/// Draw the next piece preview
fn draw_next_piece_preview(next_piece_type: &TetrominoType) {
    let preview_x = PREVIEW_OFFSET_X;
    let preview_y = PREVIEW_OFFSET_Y;
    
    // Draw preview panel background - retro style
    draw_rectangle(
        preview_x - 10.0,
        preview_y - 30.0,
        PREVIEW_SIZE + 20.0,
        PREVIEW_SIZE + 40.0,
        Color::new(0.0, 0.0, 0.2, 0.8), // Dark blue retro background
    );
    
    // Draw preview panel border - cyan retro
    draw_rectangle_lines(
        preview_x - 10.0,
        preview_y - 30.0,
        PREVIEW_SIZE + 20.0,
        PREVIEW_SIZE + 40.0,
        2.0,
        Color::new(0.0, 1.0, 1.0, 0.8), // Cyan border
    );
    
    // Draw "NEXT" label - retro yellow
    draw_text(
        "NEXT",
        preview_x,
        preview_y - 10.0,
        TEXT_SIZE,
        Color::new(1.0, 1.0, 0.0, 1.0), // Yellow retro style
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

/// Draw the hold piece preview
fn draw_hold_piece(held_piece: &Option<TetrominoType>, can_hold: bool) {
    let hold_x = HOLD_OFFSET_X;
    let hold_y = HOLD_OFFSET_Y;
    
    // Draw hold panel background - retro style
    let bg_alpha = if can_hold { 0.8 } else { 0.4 }; // Dimmed when can't hold
    draw_rectangle(
        hold_x - 10.0,
        hold_y - 30.0,
        HOLD_SIZE + 20.0,
        HOLD_SIZE + 40.0,
        Color::new(0.0, 0.0, 0.2, bg_alpha), // Dark blue retro background
    );
    
    // Draw hold panel border - retro cyan
    let border_alpha = if can_hold { 0.8 } else { 0.4 };
    draw_rectangle_lines(
        hold_x - 10.0,
        hold_y - 30.0,
        HOLD_SIZE + 20.0,
        HOLD_SIZE + 40.0,
        2.0,
        Color::new(0.0, 1.0, 1.0, border_alpha), // Cyan border
    );
    
    // Draw "HOLD" label with retro styling
    let label_color = if can_hold {
        Color::new(1.0, 1.0, 0.0, 1.0) // Yellow retro style
    } else {
        Color::new(0.6, 0.6, 0.0, 0.6) // Dimmed yellow when can't hold
    };
    
    draw_text(
        "HOLD",
        hold_x,
        hold_y - 10.0,
        TEXT_SIZE,
        label_color,
    );
    
    // Draw the held piece if there is one
    if let Some(piece_type) = held_piece {
        // Create a temporary piece for preview
        let hold_piece = Tetromino::new(*piece_type);
        let blocks = hold_piece.blocks;
        
        // Center the piece in the hold area
        let center_x = hold_x + HOLD_SIZE / 2.0;
        let center_y = hold_y + HOLD_SIZE / 2.0;
        
        // Draw the piece blocks
        let piece_alpha = if can_hold { 1.0 } else { 0.5 };
        for (dx, dy) in blocks {
            let block_x = center_x + (dx as f32 * CELL_SIZE * 0.7); // Smaller size for hold
            let block_y = center_y + (dy as f32 * CELL_SIZE * 0.7);
            let block_size = CELL_SIZE * 0.7;
            
            // Get piece color and apply alpha based on hold availability
            let base_color = piece_type.color();
            let final_color = Color::new(
                base_color.r,
                base_color.g,
                base_color.b,
                piece_alpha,
            );
            
            // Draw filled cell
            draw_rectangle(
                block_x,
                block_y,
                block_size - 1.0,
                block_size - 1.0,
                final_color,
            );
            
            // Draw highlight (only if can hold)
            if can_hold {
                draw_rectangle(
                    block_x + 1.0,
                    block_y + 1.0,
                    block_size - 3.0,
                    4.0,
                    Color::new(1.0, 1.0, 1.0, 0.3),
                );
            }
        }
    } else {
        // Show "C" key hint when no piece is held
        let hint_color = if can_hold {
            Color::new(0.8, 0.8, 0.9, 0.7)
        } else {
            Color::new(0.5, 0.5, 0.5, 0.5)
        };
        
        draw_text(
            "Press C",
            hold_x + 5.0,
            hold_y + HOLD_SIZE / 2.0 - 5.0,
            TEXT_SIZE * 0.7,
            hint_color,
        );
        draw_text(
            "to hold",
            hold_x + 8.0,
            hold_y + HOLD_SIZE / 2.0 + 15.0,
            TEXT_SIZE * 0.7,
            hint_color,
        );
    }
}

/// Draw legacy-style next piece preview using ASCII characters
fn draw_legacy_next_piece_preview(next_piece_type: &TetrominoType) {
    let preview_x = PREVIEW_OFFSET_X;
    let preview_y = PREVIEW_OFFSET_Y;
    
    // Draw preview panel background - terminal style
    draw_rectangle(
        preview_x - 10.0,
        preview_y - 30.0,
        PREVIEW_SIZE + 20.0,
        PREVIEW_SIZE + 40.0,
        Color::new(0.05, 0.05, 0.1, 0.9), // Very dark terminal background
    );
    
    // Draw simple border
    draw_rectangle_lines(
        preview_x - 10.0,
        preview_y - 30.0,
        PREVIEW_SIZE + 20.0,
        PREVIEW_SIZE + 40.0,
        1.0,
        Color::new(0.4, 0.6, 0.6, 0.8), // Dim cyan border
    );
    
    // Draw "NEXT" label
    draw_text(
        "NEXT",
        preview_x,
        preview_y - 10.0,
        TEXT_SIZE,
        Color::new(0.8, 0.8, 0.9, 1.0), // Light gray text
    );
    
    // Create a temporary piece for preview
    let preview_piece = Tetromino::new(*next_piece_type);
    let blocks = preview_piece.blocks;
    
    // Center the piece in the preview area
    let center_x = preview_x + PREVIEW_SIZE / 2.0;
    let center_y = preview_y + PREVIEW_SIZE / 2.0;
    
    // Draw the piece using ASCII blocks
    for (dx, dy) in blocks {
        let block_x = center_x + (dx as f32 * CELL_SIZE * 0.7); // Smaller size for preview
        let block_y = center_y + (dy as f32 * CELL_SIZE * 0.7);
        
            // Draw ASCII block character in terminal green
            let block_char = "█"; // Full block character
            let text_x = block_x - 6.0; // Center the character
            let text_y = block_y + 6.0;
            
            draw_text(
                block_char,
                text_x,
                text_y,
                CELL_SIZE * 0.6,
                Color::new(0.0, 1.0, 0.0, 1.0), // Terminal green instead of piece color
            );
    }
}

/// Draw legacy-style hold piece preview using ASCII characters
fn draw_legacy_hold_piece(held_piece: &Option<TetrominoType>, can_hold: bool) {
    let hold_x = HOLD_OFFSET_X;
    let hold_y = HOLD_OFFSET_Y;
    
    // Draw hold panel background - terminal style
    let bg_alpha = if can_hold { 0.9 } else { 0.4 }; // Dimmed when can't hold
    draw_rectangle(
        hold_x - 10.0,
        hold_y - 30.0,
        HOLD_SIZE + 20.0,
        HOLD_SIZE + 40.0,
        Color::new(0.05, 0.05, 0.1, bg_alpha), // Very dark terminal background
    );
    
    // Draw simple border
    let border_alpha = if can_hold { 0.8 } else { 0.4 };
    draw_rectangle_lines(
        hold_x - 10.0,
        hold_y - 30.0,
        HOLD_SIZE + 20.0,
        HOLD_SIZE + 40.0,
        1.0,
        Color::new(0.4, 0.6, 0.6, border_alpha), // Dim cyan border
    );
    
    // Draw "HOLD" label
    let label_color = if can_hold {
        Color::new(0.8, 0.8, 0.9, 1.0) // Light gray text
    } else {
        Color::new(0.4, 0.4, 0.5, 0.6) // Dimmed when can't hold
    };
    
    draw_text(
        "HOLD",
        hold_x,
        hold_y - 10.0,
        TEXT_SIZE,
        label_color,
    );
    
    // Draw the held piece if there is one
    if let Some(piece_type) = held_piece {
        // Create a temporary piece for preview
        let hold_piece = Tetromino::new(*piece_type);
        let blocks = hold_piece.blocks;
        
        // Center the piece in the hold area
        let center_x = hold_x + HOLD_SIZE / 2.0;
        let center_y = hold_y + HOLD_SIZE / 2.0;
        
        // Draw the piece using ASCII blocks
        let piece_alpha = if can_hold { 1.0 } else { 0.5 };
        for (dx, dy) in blocks {
            let block_x = center_x + (dx as f32 * CELL_SIZE * 0.7); // Smaller size for hold
            let block_y = center_y + (dy as f32 * CELL_SIZE * 0.7);
            
            // Draw ASCII block character
            let block_char = "█"; // Full block character
            let text_x = block_x - 6.0; // Center the character
            let text_y = block_y + 6.0;
            
            // Use terminal green for legacy mode consistency
            let final_color = Color::new(
                0.0,
                1.0,
                0.0,
                piece_alpha, // Keep alpha based on hold availability
            );
            
            draw_text(
                block_char,
                text_x,
                text_y,
                CELL_SIZE * 0.6,
                final_color,
            );
        }
    } else {
        // Show "C" key hint when no piece is held
        let hint_color = if can_hold {
            Color::new(0.6, 0.6, 0.7, 0.7)
        } else {
            Color::new(0.4, 0.4, 0.4, 0.5)
        };
        
        draw_text(
            "Press C",
            hold_x + 5.0,
            hold_y + HOLD_SIZE / 2.0 - 5.0,
            TEXT_SIZE * 0.7,
            hint_color,
        );
        draw_text(
            "to hold",
            hold_x + 8.0,
            hold_y + HOLD_SIZE / 2.0 + 15.0,
            TEXT_SIZE * 0.7,
            hint_color,
        );
    }
}

/// Draw legacy-style falling piece using ASCII block characters
fn draw_legacy_falling_piece(piece: &Tetromino) {
    let terminal_green = Color::new(0.0, 1.0, 0.0, 1.0); // Bright terminal green
    
    // Use the same positioning as the board
    let board_start_x = BOARD_OFFSET_X;
    let board_start_y = BOARD_OFFSET_Y;
    let char_width = CELL_SIZE;
    let char_height = CELL_SIZE;
    let char_size = CELL_SIZE * 0.8;
    
    for (x, y) in piece.absolute_blocks() {
        // Only draw blocks that are in the visible area
        if y >= BUFFER_HEIGHT as i32 {
            let visible_y = y - BUFFER_HEIGHT as i32;
            let cell_x = board_start_x + (x as f32 * char_width) + char_width * 0.25;
            let cell_y = board_start_y + (visible_y as f32 * char_height) + char_height * 0.7;
            
            // Draw ASCII block character in terminal green
            draw_text(
                "█",
                cell_x,
                cell_y,
                char_size,
                terminal_green,
            );
        }
    }
}

/// Draw legacy-style ghost piece using hollow ASCII characters
fn draw_legacy_ghost_piece(ghost_piece: &Tetromino) {
    let dimmed_green = Color::new(0.0, 0.5, 0.0, 0.7); // Dimmed terminal green
    
    // Use the same positioning as the board
    let board_start_x = BOARD_OFFSET_X;
    let board_start_y = BOARD_OFFSET_Y;
    let char_width = CELL_SIZE;
    let char_height = CELL_SIZE;
    let char_size = CELL_SIZE * 0.8;
    
    for (x, y) in ghost_piece.absolute_blocks() {
        // Only draw blocks that are in the visible area
        if y >= BUFFER_HEIGHT as i32 {
            let visible_y = y - BUFFER_HEIGHT as i32;
            let cell_x = board_start_x + (x as f32 * char_width) + char_width * 0.25;
            let cell_y = board_start_y + (visible_y as f32 * char_height) + char_height * 0.7;
            
            // Draw hollow ASCII block character in dimmed green
            draw_text(
                "▢",
                cell_x,
                cell_y,
                char_size,
                dimmed_green,
            );
        }
    }
}

/// Draw authentic terminal-style Tetris board like the original
fn draw_legacy_board_with_data(board: &Board) {
    let terminal_green = Color::new(0.0, 1.0, 0.0, 1.0); // Bright terminal green
    
    // Use the same positioning as modern board for consistency
    let board_start_x = BOARD_OFFSET_X;
    let board_start_y = BOARD_OFFSET_Y;
    let char_width = CELL_SIZE; // Same width as modern cells
    let char_height = CELL_SIZE; // Same height as modern cells
    let char_size = CELL_SIZE * 0.8; // Font size relative to cell size
    
    // Draw ASCII art border like original - top (with proper spacing)
    let top_border = "<================================>";
    draw_text(
        top_border,
        board_start_x - char_width * 1.2,
        board_start_y - char_height * 0.3,
        char_size,
        terminal_green,
    );
    
    // Draw the game board with borders
    for y in 0..VISIBLE_HEIGHT {
        // Left border (moved further from board content)
        draw_text(
            "<",
            board_start_x - char_width * 1.2,
            board_start_y + (y as f32 * char_height) + char_height * 0.7,
            char_size,
            terminal_green,
        );
        
        // Board content (adjusted for better centering)
        for x in 0..BOARD_WIDTH {
            let board_y = (y + BUFFER_HEIGHT) as i32;
            let board_x = x as i32;
            
            let cell_x = board_start_x + (x as f32 * char_width) + char_width * 0.25;
            let cell_y = board_start_y + (y as f32 * char_height) + char_height * 0.7;
            
            if let Some(cell) = board.get_cell(board_x, board_y) {
                if cell.color().is_some() {
                    // Use original terminal blocks
                    draw_text(
                        "█", // Full block for authentic look
                        cell_x,
                        cell_y,
                        char_size,
                        terminal_green,
                    );
                } else {
                    // Empty space with subtle dot
                    draw_text(
                        "·",
                        cell_x,
                        cell_y,
                        char_size,
                        Color::new(0.0, 0.25, 0.0, 0.8), // More subtle dim green for dots
                    );
                }
            } else {
                // Empty space with subtle dot
                draw_text(
                    "·",
                    cell_x,
                    cell_y,
                    char_size,
                    Color::new(0.0, 0.25, 0.0, 0.8), // More subtle dim green for dots
                );
            }
        }
        
        // Right border (moved further from board content)
        draw_text(
            ">",
            board_start_x + (BOARD_WIDTH as f32 * char_width) + char_width * 0.7,
            board_start_y + (y as f32 * char_height) + char_height * 0.7,
            char_size,
            terminal_green,
        );
    }
    
    // Bottom border (with proper spacing)
    let bottom_border = "<================================>";
    draw_text(
        bottom_border,
        board_start_x - char_width * 1.2,
        board_start_y + (VISIBLE_HEIGHT as f32 * char_height) + char_height * 0.3,
        char_size,
        terminal_green,
    );
    
    // Bottom zigzag like original (with proper spacing)
    let zigzag = "VVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVVV";
    draw_text(
        zigzag,
        board_start_x - char_width * 1.2,
        board_start_y + (VISIBLE_HEIGHT as f32 * char_height) + char_height * 0.9,
        char_size,
        terminal_green,
    );
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


/// Detect and play audio for game events
fn detect_and_play_audio_events(
    game: &Game,
    audio_system: &AudioSystem,
    _prev_score: u32,
    prev_level: u32,
    _prev_lines_cleared: u32,
    was_clearing_lines: bool,
    prev_state: GameState,
) {
    // Don't play any gameplay sounds during game over state to prevent spam
    if game.state == GameState::GameOver {
        // Only play game over sound when transitioning to game over
        if prev_state == GameState::Playing {
            audio_system.play_sound(SoundType::GameOver);
        }
        return; // Exit early to prevent other sounds during game over
    }
    
    // Line clearing sound (when lines start clearing)
    if !was_clearing_lines && game.is_clearing_lines() {
        audio_system.play_sound(SoundType::LineClear);
    }
    
    // Piece lock sound (when a piece was just locked, but not during line clearing)
    if game.piece_just_locked && !game.is_clearing_lines() {
        audio_system.play_sound_with_volume(SoundType::PieceSnap, 0.8);
    }
    
    // Level up sound
    if game.level() > prev_level {
        audio_system.play_sound(SoundType::LevelComplete);
    }
}

/// Draw retro-styled TETRIS logo with block letters
fn draw_retro_tetris_logo() {
    let logo_y = 25.0;
    let block_size = 6.0;
    let letter_width = block_size * 4.0;
    let letter_spacing = block_size * 1.5;
    
    // Calculate center position for the entire logo
    let total_width = letter_width * 6.0 + letter_spacing * 5.0; // 6 letters + 5 spaces
    let start_x = (WINDOW_WIDTH as f32 - total_width) / 2.0;
    
    let letters = [
        // T
        [
            [1,1,1,1],
            [0,1,0,0],
            [0,1,0,0],
            [0,1,0,0],
            [0,1,0,0]
        ],
        // E  
        [
            [1,1,1,1],
            [1,0,0,0],
            [1,1,1,0],
            [1,0,0,0],
            [1,1,1,1]
        ],
        // T
        [
            [1,1,1,1],
            [0,1,0,0],
            [0,1,0,0],
            [0,1,0,0],
            [0,1,0,0]
        ],
        // R
        [
            [1,1,1,0],
            [1,0,0,1],
            [1,1,1,0],
            [1,0,1,0],
            [1,0,0,1]
        ],
        // I
        [
            [1,1,1,0],
            [0,1,0,0],
            [0,1,0,0],
            [0,1,0,0],
            [1,1,1,0]
        ],
        // S
        [
            [0,1,1,1],
            [1,0,0,0],
            [0,1,1,0],
            [0,0,0,1],
            [1,1,1,0]
        ]
    ];
    
    // Draw each letter
    for (letter_idx, letter) in letters.iter().enumerate() {
        let letter_x = start_x + (letter_idx as f32 * (letter_width + letter_spacing));
        
        // Draw letter blocks with retro colors
        for (row, line) in letter.iter().enumerate() {
            for (col, &block) in line.iter().enumerate() {
                if block == 1 {
                    let x = letter_x + (col as f32 * block_size);
                    let y = logo_y + (row as f32 * block_size);
                    
                    // Create rainbow effect across letters
                    let hue = (letter_idx as f64 + col as f64 * 0.2) / 6.0 * 6.0; // Full rainbow across 6 letters
                    let letter_color = hsv_to_rgb(hue, 0.9, 1.0);
                    
                    // Draw main block
                    draw_rectangle(
                        x,
                        y,
                        block_size,
                        block_size,
                        letter_color,
                    );
                    
                    // Draw glow effect
                    draw_rectangle(
                        x - 1.0,
                        y - 1.0,
                        block_size + 2.0,
                        block_size + 2.0,
                        Color::new(letter_color.r, letter_color.g, letter_color.b, 0.3),
                    );
                }
            }
        }
    }
}

/// Draw enhanced UI elements with retro theme
fn draw_enhanced_ui(game: &Game) {
    // Draw retro TETRIS title logo
    draw_retro_tetris_logo();
    
    // Draw retro subtitle
    let subtitle = "CLASSIC ARCADE EDITION";
    let subtitle_x = (WINDOW_WIDTH as f32 - measure_text(subtitle, None, (TEXT_SIZE * 0.8) as u16, 1.0).width) / 2.0;
    
    draw_text(
        subtitle,
        subtitle_x,
        75.0,
        TEXT_SIZE * 0.8,
        Color::new(0.0, 1.0, 1.0, 0.9), // Cyan retro color
    );
    
    // Instructions with background - compact retro style
    let instructions = vec![
        "CONTROLS:",
        "← → A D - Move",
        "↓ S - Soft Drop",
        "↑ X W / Z - Rotate",
        "SPACE - Hard Drop",
        "C - Hold Piece",
        "P - Pause / R - Reset",
        "Ctrl+S - Save Game",
    ];
    
    let inst_x = 25.0; // Moderate padding from left edge
    let instruction_height = (instructions.len() as f32 * 18.0) + 35.0; // Moderate internal padding
    let mut inst_y = WINDOW_HEIGHT as f32 - instruction_height - 15.0; // Moderate padding from bottom
    
    // Calculate safe width that won't overlap with board
    let max_safe_width = BOARD_OFFSET_X - inst_x - 10.0; // Leave 10px gap from board
    let panel_width = max_safe_width.min(260.0); // Cap at reasonable width
    
    // Instructions background with retro border
    draw_rectangle(
        inst_x - 12.0, // Moderate left padding
        inst_y - 22.0, // Moderate top padding
        panel_width,
        instruction_height,
        Color::new(0.0, 0.0, 0.2, 0.8), // Dark blue retro background
    );
    
    // Retro border
    draw_rectangle_lines(
        inst_x - 12.0, // Match background padding
        inst_y - 22.0, // Match background padding
        panel_width, // Match background width
        instruction_height,
        2.0,
        Color::new(0.0, 1.0, 1.0, 0.8), // Cyan border
    );
    
    for (i, instruction) in instructions.iter().enumerate() {
        let color = if i == 0 {
            Color::new(1.0, 1.0, 0.0, 1.0) // Yellow header - retro style
        } else {
            Color::new(0.0, 1.0, 0.0, 0.9) // Green text - classic terminal green
        };
        
        draw_text(instruction, inst_x, inst_y, TEXT_SIZE * 0.75, color);
        inst_y += 18.0; // Tighter spacing
    }
    
    // Game statistics panel with retro styling - position on right side (consistent with preview spacing)
    let stats_x = PREVIEW_OFFSET_X; // Use same x position as preview panel
    let mut stats_y = PREVIEW_OFFSET_Y + PREVIEW_SIZE + 60.0; // Below the Next piece panel
    
    // Stats background - retro dark blue
    draw_rectangle(
        stats_x - 10.0,
        stats_y - 30.0,
        200.0,
        160.0, // Slightly smaller height
        Color::new(0.0, 0.0, 0.2, 0.8), // Dark blue retro background
    );
    
    // Stats border - cyan retro style
    draw_rectangle_lines(
        stats_x - 10.0,
        stats_y - 30.0,
        200.0,
        160.0,
        2.0,
        Color::new(0.0, 1.0, 1.0, 0.8), // Cyan border
    );
    
    // Stats title - retro yellow
    draw_text(
        "GAME STATS",
        stats_x,
        stats_y - 10.0,
        TEXT_SIZE * 0.9,
        Color::new(1.0, 1.0, 0.0, 1.0), // Yellow retro header
    );
    stats_y += 15.0;
    
    // Individual stats
    let stats = vec![
        format!("Score: {}", game.score),
        format!("Level: {}", game.level()),
        format!("Lines: {}", game.lines_cleared()),
        format!("Ghost Blocks: {}", game.ghost_blocks_available),
        format!("State: {:?}", game.state),
        format!("Time: {:.0}s", game.game_time),
    ];
    
    for (i, stat) in stats.iter().enumerate() {
        let color = if i == 3 && game.ghost_blocks_available > 0 {
            // Highlight ghost blocks count with pulsing effect when available
            let pulse = (game.game_time * 3.0).sin() as f32 * 0.3 + 0.7;
            Color::new(0.8, 0.8, 1.0, pulse) // Light blue pulsing
        } else {
            Color::new(0.0, 1.0, 0.0, 0.9) // Green terminal-style text
        };
        
        draw_text(
            stat,
            stats_x,
            stats_y,
            TEXT_SIZE * 0.75,
            color,
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
    
    // Ghost block placement mode indicator (if active)
    if game.ghost_block_placement_mode {
        // Main placement mode message
        let placement_info = "GHOST BLOCK PLACEMENT MODE - M/N for smart positions, Arrows to fine-tune, B to place";
        draw_text(
            placement_info,
            BOARD_OFFSET_X,
            BOARD_OFFSET_Y - 50.0,
            TEXT_SIZE * 0.7,
            Color::new(0.8, 0.8, 1.0, 0.9),
        );
        
        // Strategic info about current position
        if let Some((current_pos, total_positions, blocks_needed)) = game.get_current_position_info() {
            let strategy_info = format!(
                "Position {}/{} - {} block{} needed to complete line",
                current_pos,
                total_positions,
                blocks_needed,
                if blocks_needed == 1 { "" } else { "s" }
            );
            
            // Color based on strategic value (fewer blocks needed = better = greener)
            let strategy_color = match blocks_needed {
                1 => Color::new(0.2, 1.0, 0.2, 0.9),       // Bright green - excellent!
                2 => Color::new(0.6, 1.0, 0.2, 0.9),       // Yellow-green - very good
                3 => Color::new(1.0, 0.8, 0.2, 0.9),       // Yellow - good
                4 => Color::new(1.0, 0.6, 0.2, 0.9),       // Orange - okay
                _ => Color::new(1.0, 0.4, 0.4, 0.9),       // Red - not ideal
            };
            
            draw_text(
                &strategy_info,
                BOARD_OFFSET_X,
                BOARD_OFFSET_Y - 30.0,
                TEXT_SIZE * 0.75,
                strategy_color,
            );
        }
    }
}

/// Draw legacy-style UI with terminal-style text and minimal styling
fn draw_legacy_ui(game: &Game) {
    let terminal_green = Color::new(0.0, 1.0, 0.0, 1.0);
    
    // Simple title in terminal green (same position as modern title)
    let title = "TETRIS";
    let title_size = 24.0;
    let title_width = measure_text(title, None, title_size as u16, 1.0).width;
    let title_x = (WINDOW_WIDTH as f32 - title_width) / 2.0;
    
    draw_text(
        title,
        title_x,
        30.0, // Same as modern title area
        title_size,
        terminal_green,
    );
    
    // Instructions - same position as modern UI
    let instructions = vec![
        "CONTROLS:",
        "← → A D - Move",
        "↓ S - Soft Drop",
        "↑ X W / Z - Rotate",
        "SPACE - Hard Drop",
        "C - Hold Piece",
        "P - Pause / R - Reset",
        "L - Modern Mode", // Changed from original
    ];
    
    let inst_x = 25.0; // Same as modern UI
    let instruction_height = (instructions.len() as f32 * 18.0) + 35.0;
    let mut inst_y = WINDOW_HEIGHT as f32 - instruction_height - 15.0; // Same position
    
    // Calculate safe width that won't overlap with board (same as modern)
    let max_safe_width = BOARD_OFFSET_X - inst_x - 10.0;
    let panel_width = max_safe_width.min(260.0);
    
    // No background/border in legacy mode for minimal terminal look
    for (i, instruction) in instructions.iter().enumerate() {
        let color = if i == 0 {
            Color::new(0.8, 0.8, 0.8, 1.0) // White header
        } else {
            terminal_green // Green text
        };
        
        draw_text(instruction, inst_x, inst_y, TEXT_SIZE * 0.75, color);
        inst_y += 18.0;
    }
    
    // Game statistics - consistent positioning with preview panel
    let stats_x = PREVIEW_OFFSET_X; // Use same x position as preview panel
    let mut stats_y = PREVIEW_OFFSET_Y + PREVIEW_SIZE + 60.0; // Same as modern UI
    
    // Stats title
    draw_text(
        "GAME STATS",
        stats_x,
        stats_y - 10.0,
        TEXT_SIZE * 0.9,
        Color::new(0.8, 0.8, 0.8, 1.0), // White header
    );
    stats_y += 15.0;
    
    // Individual stats (same format as modern UI)
    let stats = vec![
        format!("Score: {}", game.score),
        format!("Level: {}", game.level()),
        format!("Lines: {}", game.lines_cleared()),
        format!("Ghost Blocks: {}", game.ghost_blocks_available),
        format!("State: {:?}", game.state),
        format!("Time: {:.0}s", game.game_time),
    ];
    
    for (i, stat) in stats.iter().enumerate() {
        let color = if i == 3 && game.ghost_blocks_available > 0 {
            // Highlight ghost blocks count (terminal green instead of blue)
            terminal_green
        } else {
            terminal_green // All stats in terminal green
        };
        
        draw_text(
            stat,
            stats_x,
            stats_y,
            TEXT_SIZE * 0.75,
            color,
        );
        stats_y += 22.0;
    }
    
    // Current piece info (same position as modern UI)
    if let Some(ref piece) = game.current_piece {
        draw_text(
            &format!("Current: {}", piece.piece_type.name()),
            stats_x,
            stats_y,
            TEXT_SIZE * 0.7,
            terminal_green, // Terminal green instead of piece color
        );
    }
    
    // Ghost block placement mode indicator (same position as modern UI)
    if game.ghost_block_placement_mode {
        let placement_info = "GHOST BLOCK PLACEMENT MODE - M/N for smart positions, Arrows to fine-tune, B to place";
        draw_text(
            placement_info,
            BOARD_OFFSET_X, // Same position as modern UI
            BOARD_OFFSET_Y - 50.0,
            TEXT_SIZE * 0.7,
            Color::new(0.8, 0.8, 0.8, 1.0), // White for visibility
        );
        
        // Strategic info about current position (same as modern UI)
        if let Some((current_pos, total_positions, blocks_needed)) = game.get_current_position_info() {
            let strategy_info = format!(
                "Position {}/{} - {} block{} needed to complete line",
                current_pos,
                total_positions,
                blocks_needed,
                if blocks_needed == 1 { "" } else { "s" }
            );
            
            // Simple terminal green color instead of gradient
            let strategy_color = terminal_green;
            
            draw_text(
                &strategy_info,
                BOARD_OFFSET_X, // Same position as modern UI
                BOARD_OFFSET_Y - 30.0,
                TEXT_SIZE * 0.75,
                strategy_color,
            );
        }
    }
}

/// Draw Game Over overlay
fn draw_game_over_overlay(game: &Game) {
    // Semi-transparent dark overlay
    draw_rectangle(
        0.0,
        0.0,
        WINDOW_WIDTH as f32,
        WINDOW_HEIGHT as f32,
        Color::new(0.0, 0.0, 0.0, 0.7),
    );
    
    // Game Over message
    let message = "GAME OVER";
    let font_size = 60.0;
    let text_width = measure_text(message, None, font_size as u16, 1.0).width;
    let center_x = (WINDOW_WIDTH as f32 - text_width) / 2.0;
    let center_y = WINDOW_HEIGHT as f32 / 2.0 - 80.0;
    
    // Draw outline for better visibility
    let outline_color = Color::new(0.0, 0.0, 0.0, 0.9);
    for offset_x in [-3.0, 0.0, 3.0] {
        for offset_y in [-3.0, 0.0, 3.0] {
            if offset_x != 0.0 || offset_y != 0.0 {
                draw_text(
                    message,
                    center_x + offset_x,
                    center_y + offset_y,
                    font_size,
                    outline_color,
                );
            }
        }
    }
    
    // Main text in bright red
    draw_text(
        message,
        center_x,
        center_y,
        font_size,
        Color::new(1.0, 0.2, 0.2, 1.0),
    );
    
    // Final stats
    let stats_lines = vec![
        format!("Final Score: {}", game.score),
        format!("Level Reached: {}", game.level()),
        format!("Lines Cleared: {}", game.lines_cleared()),
        format!("Time Played: {:.0}s", game.game_time),
    ];
    
    let stats_y_start = center_y + 60.0;
    for (i, stat) in stats_lines.iter().enumerate() {
        let stat_width = measure_text(stat, None, 24, 1.0).width;
        let stat_x = (WINDOW_WIDTH as f32 - stat_width) / 2.0;
        let stat_y = stats_y_start + (i as f32 * 30.0);
        
        // Stat outline
        for offset_x in [-1.0, 0.0, 1.0] {
            for offset_y in [-1.0, 0.0, 1.0] {
                if offset_x != 0.0 || offset_y != 0.0 {
                    draw_text(
                        stat,
                        stat_x + offset_x,
                        stat_y + offset_y,
                        24.0,
                        Color::new(0.0, 0.0, 0.0, 0.8),
                    );
                }
            }
        }
        
        draw_text(
            stat,
            stat_x,
            stat_y,
            24.0,
            Color::new(1.0, 1.0, 0.8, 1.0),
        );
    }
    
    // Instructions
    let instruction = "Press R to restart or ESC to quit";
    let inst_width = measure_text(instruction, None, 20, 1.0).width;
    let inst_x = (WINDOW_WIDTH as f32 - inst_width) / 2.0;
    let inst_y = stats_y_start + 180.0;
    
    // Instruction outline
    for offset_x in [-1.0, 0.0, 1.0] {
        for offset_y in [-1.0, 0.0, 1.0] {
            if offset_x != 0.0 || offset_y != 0.0 {
                draw_text(
                    instruction,
                    inst_x + offset_x,
                    inst_y + offset_y,
                    20.0,
                    Color::new(0.0, 0.0, 0.0, 0.8),
                );
            }
        }
    }
    
    draw_text(
        instruction,
        inst_x,
        inst_y,
        20.0,
        Color::new(0.8, 0.8, 0.9, 1.0),
    );
}

/// Draw Pause overlay
fn draw_pause_overlay(_game: &Game) {
    // Semi-transparent dark overlay
    draw_rectangle(
        0.0,
        0.0,
        WINDOW_WIDTH as f32,
        WINDOW_HEIGHT as f32,
        Color::new(0.0, 0.0, 0.0, 0.5),
    );
    
    // Pause message
    let message = "PAUSED";
    let font_size = 50.0;
    let text_width = measure_text(message, None, font_size as u16, 1.0).width;
    let center_x = (WINDOW_WIDTH as f32 - text_width) / 2.0;
    let center_y = WINDOW_HEIGHT as f32 / 2.0 - 40.0;
    
    // Draw outline for better visibility
    let outline_color = Color::new(0.0, 0.0, 0.0, 0.9);
    for offset_x in [-2.0, 0.0, 2.0] {
        for offset_y in [-2.0, 0.0, 2.0] {
            if offset_x != 0.0 || offset_y != 0.0 {
                draw_text(
                    message,
                    center_x + offset_x,
                    center_y + offset_y,
                    font_size,
                    outline_color,
                );
            }
        }
    }
    
    // Main text in bright cyan
    draw_text(
        message,
        center_x,
        center_y,
        font_size,
        Color::new(0.0, 1.0, 1.0, 1.0),
    );
    
    // Instructions
    let instruction = "Press P to resume";
    let inst_width = measure_text(instruction, None, 24, 1.0).width;
    let inst_x = (WINDOW_WIDTH as f32 - inst_width) / 2.0;
    let inst_y = center_y + 60.0;
    
    // Instruction outline
    for offset_x in [-1.0, 0.0, 1.0] {
        for offset_y in [-1.0, 0.0, 1.0] {
            if offset_x != 0.0 || offset_y != 0.0 {
                draw_text(
                    instruction,
                    inst_x + offset_x,
                    inst_y + offset_y,
                    24.0,
                    Color::new(0.0, 0.0, 0.0, 0.8),
                );
            }
        }
    }
    
    draw_text(
        instruction,
        inst_x,
        inst_y,
        24.0,
        Color::new(1.0, 1.0, 0.8, 1.0),
    );
}

/// Show startup menu with load/new game options
async fn show_startup_menu(save_path: &std::path::Path) -> Game {
    // Create background texture once (same as main game)
    let background_texture = Texture2D::from_image(&create_chess_background());
    
    loop {
        // Clear screen with dark background
        clear_background(BACKGROUND_COLOR);
        
        // Draw magical background (same as main game)
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
            Color::new(0.0, 0.0, 0.0, 0.5), // Slightly darker for menu readability
        );
        
        // Draw enhanced title with glow effect
        let title = "RUST TETRIS";
        let title_size = 70.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        let title_x = (WINDOW_WIDTH as f32 - title_width) / 2.0;
        let title_y = 150.0;
        
        // Title glow/outline effect
        for offset_x in [-2.0, -1.0, 0.0, 1.0, 2.0] {
            for offset_y in [-2.0, -1.0, 0.0, 1.0, 2.0] {
                if offset_x != 0.0 || offset_y != 0.0 {
                    draw_text(
                        title,
                        title_x + offset_x,
                        title_y + offset_y,
                        title_size,
                        Color::new(0.0, 0.0, 0.0, 0.8),
                    );
                }
            }
        }
        
        // Main title with rainbow effect
        draw_text(
            title,
            title_x,
            title_y,
            title_size,
            Color::new(0.0, 1.0, 1.0, 1.0), // Bright cyan
        );
        
        // Draw magical subtitle with pulsing effect
        let subtitle = "⭐ Save file found! ⭐";
        let subtitle_size = 32.0;
        let subtitle_width = measure_text(subtitle, None, subtitle_size as u16, 1.0).width;
        let subtitle_x = (WINDOW_WIDTH as f32 - subtitle_width) / 2.0;
        let subtitle_y = 230.0;
        
        // Subtitle outline
        for offset_x in [-1.0, 0.0, 1.0] {
            for offset_y in [-1.0, 0.0, 1.0] {
                if offset_x != 0.0 || offset_y != 0.0 {
                    draw_text(
                        subtitle,
                        subtitle_x + offset_x,
                        subtitle_y + offset_y,
                        subtitle_size,
                        Color::new(0.0, 0.0, 0.0, 0.9),
                    );
                }
            }
        }
        
        draw_text(
            subtitle,
            subtitle_x,
            subtitle_y,
            subtitle_size,
            Color::new(1.0, 0.9, 0.3, 1.0), // Golden color
        );
        
        // Draw enhanced menu options with magical styling
        let option1 = "🔮 Press L to LOAD saved game";
        let option2 = "⚡ Press N to start NEW game";
        let option3 = "❌ Press ESC to quit";
        
        let option_size = 26.0;
        let option_y_start = 320.0;
        let option_spacing = 50.0;
        
        let options = [option1, option2, option3];
        let colors = [
            Color::new(0.3, 1.0, 0.3, 1.0), // Bright green for load
            Color::new(1.0, 0.8, 0.2, 1.0), // Golden orange for new
            Color::new(1.0, 0.3, 0.3, 1.0), // Bright red for quit
        ];
        
        for (i, (option, color)) in options.iter().zip(colors.iter()).enumerate() {
            let option_width = measure_text(option, None, option_size as u16, 1.0).width;
            let option_x = (WINDOW_WIDTH as f32 - option_width) / 2.0;
            let option_y = option_y_start + (i as f32 * option_spacing);
            
            // Option outline for better readability
            for offset_x in [-1.0, 0.0, 1.0] {
                for offset_y in [-1.0, 0.0, 1.0] {
                    if offset_x != 0.0 || offset_y != 0.0 {
                        draw_text(
                            option,
                            option_x + offset_x,
                            option_y + offset_y,
                            option_size,
                            Color::new(0.0, 0.0, 0.0, 0.9),
                        );
                    }
                }
            }
            
            draw_text(
                option,
                option_x,
                option_y,
                option_size,
                *color,
            );
        }
        
        // Add magical floating particles effect
        let time = get_time() as f32;
        for i in 0..20 {
            let particle_phase = (time * 0.3 + i as f32 * 0.5) % 6.28;
            let x_base = (WINDOW_WIDTH as f32 / 20.0) * (i as f32 + 1.0);
            let y_offset = (particle_phase.sin() * 30.0) + (time * 0.2 + i as f32).sin() * 15.0;
            let y_pos = 100.0 + y_offset;
            
            // Vary particle colors
            let hue = (i as f64 * 0.3 + time as f64 * 0.5) % 6.0;
            let particle_color = hsv_to_rgb(hue, 0.8, 0.9);
            let alpha = (0.3 + 0.4 * ((time * 0.8 + i as f32 * 0.3).sin() * 0.5 + 0.5)) * 0.7;
            
            let size = 2.0 + ((time * 0.6 + i as f32 * 0.4).cos() * 0.5 + 0.5) * 3.0;
            
            draw_rectangle(
                x_base - size / 2.0,
                y_pos - size / 2.0,
                size,
                size,
                Color::new(particle_color.r, particle_color.g, particle_color.b, alpha),
            );
            
            // Add small glow around particles
            draw_rectangle(
                x_base - size,
                y_pos - size,
                size * 2.0,
                size * 2.0,
                Color::new(particle_color.r, particle_color.g, particle_color.b, alpha * 0.2),
            );
        }
        
        // Add magical glow effect around the entire menu area
        let menu_center_x = WINDOW_WIDTH as f32 / 2.0;
        let menu_center_y = WINDOW_HEIGHT as f32 / 2.0;
        let glow_radius = 300.0 + (time * 0.8).sin() * 20.0;
        let glow_alpha = 0.1 + (time * 0.5).sin() * 0.05;
        
        // Draw concentric glow circles
        for ring in 1..6 {
            let ring_radius = glow_radius * (ring as f32 / 6.0);
            let ring_alpha = glow_alpha / (ring as f32);
            
            draw_rectangle(
                menu_center_x - ring_radius,
                menu_center_y - ring_radius,
                ring_radius * 2.0,
                ring_radius * 2.0,
                Color::new(0.2, 0.4, 1.0, ring_alpha),
            );
        }
        
        // Handle input
        if is_key_pressed(KeyCode::L) {
            // Load saved game
            match Game::load_from_file(save_path) {
                Ok(game) => {
                    log::info!("Loaded saved game successfully");
                    return game;
                },
                Err(e) => {
                    log::warn!("Failed to load save file: {}", e);
                    // Fall back to new game
                    return Game::new();
                }
            }
        }
        
        if is_key_pressed(KeyCode::N) {
            // Start new game
            log::info!("Starting new game");
            return Game::new();
        }
        
        if is_key_pressed(KeyCode::Escape) {
            std::process::exit(0);
        }
        
        next_frame().await;
    }
}

/// Draw animated TETRIS celebration message with rainbow colors and effects
fn draw_tetris_celebration(game: &Game) {
    let progress = game.get_tetris_celebration_progress();
    
    // Calculate animation phases
    let fade_in_time = 0.2; // First 20% of animation
    let stable_time = 0.6;  // 60% stable display
    let fade_out_time = 0.2; // Last 20% fade out
    
    let alpha = if progress <= fade_in_time {
        // Fade in phase
        (progress / fade_in_time) as f32
    } else if progress <= fade_in_time + stable_time {
        // Stable phase
        1.0
    } else {
        // Fade out phase
        let fade_progress = (progress - fade_in_time - stable_time) / fade_out_time;
        (1.0 - fade_progress) as f32
    };
    
    // Scale effect - grows slightly then stabilizes
    let scale = if progress <= fade_in_time {
        0.5 + (progress / fade_in_time) as f32 * 0.7 // Grow from 0.5x to 1.2x
    } else if progress <= fade_in_time + 0.1 {
        1.2 - ((progress - fade_in_time) / 0.1) as f32 * 0.2 // Shrink back to 1.0x
    } else {
        1.0
    };
    
    // Center the message on screen (based on our actual per-letter spacing)
    let base_font_size = 80.0;
    let font_size = base_font_size * scale;
    let message = "JONES'IN!";
    let center_y = WINDOW_HEIGHT as f32 / 2.0 - 50.0;
    
    // Per-letter layout parameters
    let time_offset = game.get_tetris_celebration_progress() * 8.0; // Speed of color animation
    let letter_spacing = font_size * 0.7;
    let chars: Vec<char> = message.chars().collect();
    let non_excl_count = chars.iter().filter(|&&ch| ch != '!').count() as f32;
    let total_letters_width = if non_excl_count <= 1.0 { 0.0 } else { (non_excl_count - 1.0) * letter_spacing };
    let base_x = (WINDOW_WIDTH as f32 - total_letters_width) / 2.0;
    let midline_x = base_x + total_letters_width / 2.0;
    
    // Background glow effect centered to our drawn layout
    let glow_size = 400.0 * scale;
    let glow_alpha = alpha * 0.3;
    draw_rectangle(
        midline_x - glow_size / 2.0,
        center_y - glow_size / 4.0,
        glow_size,
        glow_size / 2.0,
        Color::new(1.0, 1.0, 1.0, glow_alpha * 0.1),
    );
    
    // Draw each letter with animated rainbow colors
    let mut draw_index_without_excl = 0usize; // index among non-'!' characters only
    for &c in &chars {
        if c == '!' {
            continue; // Handle exclamation point separately
        }
        
        let letter_x = base_x + (draw_index_without_excl as f32) * letter_spacing;
        draw_index_without_excl += 1;
        
        // Create rainbow effect with time-based animation
        let hue = ((draw_index_without_excl as f64 * 0.5) + time_offset) % 6.0;
        let rainbow_color = hsv_to_rgb(hue, 1.0, 1.0);
        let final_color = Color::new(
            rainbow_color.r,
            rainbow_color.g,
            rainbow_color.b,
            alpha,
        );
        
        // Draw letter with outline for better visibility
        let outline_color = Color::new(0.0, 0.0, 0.0, alpha * 0.8);
        
        // Draw outline (multiple passes for thickness)
        for offset_x in [-2.0, 0.0, 2.0] {
            for offset_y in [-2.0, 0.0, 2.0] {
                if offset_x != 0.0 || offset_y != 0.0 {
                    draw_text(
                        &c.to_string(),
                        letter_x + offset_x,
                        center_y + offset_y,
                        font_size,
                        outline_color,
                    );
                }
            }
        }
        
        // Draw main letter
        draw_text(
            &c.to_string(),
            letter_x,
            center_y,
            font_size,
            final_color,
        );
        
        // Add sparkle effect around letters
        if progress > 0.1 {
            let sparkle_count = 3;
            for j in 0..sparkle_count {
                let sparkle_time = (game.get_tetris_celebration_progress() * 6.0 + (draw_index_without_excl - 1) as f64 * 0.5 + j as f64) % 1.0;
                let sparkle_alpha = (sparkle_time.sin() * 0.5 + 0.5) as f32 * alpha * 0.8;
                
                if sparkle_alpha > 0.3 {
                    let angle = sparkle_time * 6.28 + j as f64; // Full rotation
                    let distance = 40.0 + sparkle_time as f32 * 20.0;
                    let sparkle_x = letter_x + angle.cos() as f32 * distance;
                    let sparkle_y = center_y + angle.sin() as f32 * distance * 0.5;
                    
                    let sparkle_size = 3.0 + sparkle_alpha * 2.0;
                    draw_rectangle(
                        sparkle_x - sparkle_size / 2.0,
                        sparkle_y - sparkle_size / 2.0,
                        sparkle_size,
                        sparkle_size,
                        Color::new(1.0, 1.0, 1.0, sparkle_alpha),
                    );
                }
            }
        }
    }
    
    // Draw exclamation point with special pulsing effect
    // Position it exactly where '!' appears in the message using our per-letter layout
    let excl_draw_pos_without_excl = if let Some(excl_idx) = chars.iter().position(|&ch| ch == '!') {
        // Count how many non-'!' chars come before '!'
        chars[..excl_idx].iter().filter(|&&ch| ch != '!').count()
    } else {
        // If no '!' present, place it after the last character
        non_excl_count as usize
    };
    let excl_x = base_x + (excl_draw_pos_without_excl as f32) * letter_spacing;
    let pulse = (game.get_tetris_celebration_progress() * 12.0).sin() as f32 * 0.2 + 1.0;
    let excl_scale = scale * pulse;
    let excl_font_size = font_size * excl_scale;
    
    // Exclamation point gets extra bright yellow color
    let excl_color = Color::new(1.0, 1.0, 0.3, alpha);
    let excl_outline = Color::new(0.0, 0.0, 0.0, alpha * 0.8);
    
    // Draw exclamation outline
    for offset_x in [-2.0, 0.0, 2.0] {
        for offset_y in [-2.0, 0.0, 2.0] {
            if offset_x != 0.0 || offset_y != 0.0 {
                draw_text(
                    "!",
                    excl_x + offset_x,
                    center_y + offset_y,
                    excl_font_size,
                    excl_outline,
                );
            }
        }
    }
    
    // Draw exclamation point
    draw_text(
        "!",
        excl_x,
        center_y,
        excl_font_size,
        excl_color,
    );
    
    // Subtitle message
    if progress > 0.3 {
        let subtitle = "4 LINES CLEARED!";
        let subtitle_alpha = ((progress - 0.3) / 0.7) as f32 * alpha;
        let subtitle_size = 24.0 * scale;
        let subtitle_width = measure_text(subtitle, None, subtitle_size as u16, 1.0).width;
        let subtitle_x = (WINDOW_WIDTH as f32 - subtitle_width) / 2.0;
        let subtitle_y = center_y + font_size + 20.0;
        
        // Subtitle uses cycling rainbow colors too
        let subtitle_hue = (time_offset * 0.7) % 6.0;
        let subtitle_rainbow = hsv_to_rgb(subtitle_hue, 0.8, 1.0);
        let subtitle_color = Color::new(
            subtitle_rainbow.r,
            subtitle_rainbow.g,
            subtitle_rainbow.b,
            subtitle_alpha,
        );
        
        // Subtitle outline
        let subtitle_outline = Color::new(0.0, 0.0, 0.0, subtitle_alpha * 0.8);
        for offset_x in [-1.0, 0.0, 1.0] {
            for offset_y in [-1.0, 0.0, 1.0] {
                if offset_x != 0.0 || offset_y != 0.0 {
                    draw_text(
                        subtitle,
                        subtitle_x + offset_x,
                        subtitle_y + offset_y,
                        subtitle_size,
                        subtitle_outline,
                    );
                }
            }
        }
        
        draw_text(
            subtitle,
            subtitle_x,
            subtitle_y,
            subtitle_size,
            subtitle_color,
        );
    }
}

