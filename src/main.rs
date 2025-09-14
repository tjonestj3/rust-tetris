use macroquad::prelude::*;
use rust_tetris::game::config::*;
use rust_tetris::graphics::colors::*;
use rust_tetris::board::Board;
use rust_tetris::game::{Game, GameState};
use rust_tetris::tetromino::{Tetromino, TetrominoType};
use rust_tetris::audio::system::{AudioSystem, SoundType};

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
    
    // Initialize and load audio system
    let mut audio_system = AudioSystem::new();
    if let Err(e) = audio_system.load_sounds().await {
        log::warn!("Failed to initialize audio system: {}", e);
    }
    
    // Start background music
    audio_system.start_background_music();
    
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
        handle_input(&mut game, &audio_system);
        
        // Store previous state for audio event detection
        let prev_score = game.score;
        let prev_level = game.level();
        let prev_lines_cleared = game.lines_cleared();
        let was_clearing_lines = game.is_clearing_lines();
        let prev_state = game.state;
        
        // Update game logic
        game.update(delta_time as f64);
        
        // Detect and play audio for game events
        detect_and_play_audio_events(&game, &audio_system, prev_score, prev_level, prev_lines_cleared, was_clearing_lines, prev_state);

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
        
        // Draw line clearing animation if active
        if game.is_clearing_lines() {
            draw_line_clear_animation(&game);
        }
        
        // Draw the current falling piece (only if not clearing lines)
        if !game.is_clearing_lines() {
            // Draw ghost piece first (behind the actual piece)
            if let Some(ghost_piece) = game.calculate_ghost_piece() {
                draw_ghost_piece(&ghost_piece);
            }
            
            if let Some(ref piece) = game.current_piece {
                draw_falling_piece(piece);
            }
        }
        
        // Draw ghost block cursor if in placement mode
        if game.is_ghost_cursor_visible() {
            draw_ghost_block_cursor(&game);
        }
        
        // Draw next piece preview
        draw_next_piece_preview(&game.next_piece);
        
        // Draw hold piece
        draw_hold_piece(&game.held_piece, game.can_hold());
        
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

/// Handle player input with audio feedback
fn handle_input(game: &mut Game, audio_system: &AudioSystem) {
    // Quit game
    if is_key_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
    
    // Only handle game input when playing
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
    
    // Pause
    if is_key_pressed(KeyCode::P) {
        game.toggle_pause();
        audio_system.play_sound(SoundType::Pause);
    }
    
    // Reset game (R key)
    if is_key_pressed(KeyCode::R) {
        game.reset();
        audio_system.play_sound_with_volume(SoundType::UiClick, 1.0);
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
            
            // Get base color and make it translucent
            let base_color = ghost_piece.color();
            let ghost_color = Color::new(
                base_color.r,
                base_color.g,
                base_color.b,
                0.3, // Make it quite transparent
            );
            
            // Draw ghost cell with just a border outline
            draw_rectangle_lines(
                cell_x + 2.0,
                cell_y + 2.0,
                CELL_SIZE - 4.0,
                CELL_SIZE - 4.0,
                2.0,
                ghost_color,
            );
            
            // Add subtle fill for better visibility
            draw_rectangle(
                cell_x + 4.0,
                cell_y + 4.0,
                CELL_SIZE - 8.0,
                CELL_SIZE - 8.0,
                Color::new(base_color.r, base_color.g, base_color.b, 0.1),
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
    
    // Game over sound
    if prev_state == GameState::Playing && game.state == GameState::GameOver {
        audio_system.play_sound(SoundType::GameOver);
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
    
    // Game statistics panel with retro styling - position on right side
    let stats_x = BOARD_OFFSET_X + BOARD_WIDTH_PX + 20.0; // Right side of board
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

