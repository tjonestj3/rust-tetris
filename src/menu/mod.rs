//! Enhanced start menu system for the Tetris game

use macroquad::prelude::*;
use crate::game::config::*;
use crate::leaderboard::Leaderboard;
use crate::Game;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

/// Different states the menu system can be in
#[derive(Debug, Clone, PartialEq)]
pub enum MenuState {
    /// Main menu with all options
    Main,
    /// Leaderboard viewing screen
    Leaderboard,
    /// Settings/options menu
    Settings,
    /// High score name entry screen
    NameEntry { score: u32, level: u32, lines_cleared: u32, game_time: f64 },
}

/// Game settings that persist across sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    /// Whether sound is enabled
    pub sound_enabled: bool,
    /// Master volume (0.0 to 1.0)
    pub volume: f32,
}

impl GameSettings {
    /// Create default settings
    pub fn default() -> Self {
        Self {
            sound_enabled: true,
            volume: 0.7,
        }
    }
    
    /// Get the default settings file path
    pub fn default_path() -> std::path::PathBuf {
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("tetris_settings.json")
    }
    
    /// Save settings to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        log::info!("Settings saved successfully");
        Ok(())
    }
    
    /// Load settings from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let json = fs::read_to_string(path)?;
        let settings: GameSettings = serde_json::from_str(&json)?;
        log::info!("Settings loaded successfully");
        Ok(settings)
    }
    
    /// Load settings from file, or create default if file doesn't exist
    pub fn load_or_default<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(&path) {
            Ok(settings) => settings,
            Err(e) => {
                log::info!("Could not load settings ({}), using defaults", e);
                Self::default()
            }
        }
    }
}

/// The main menu system controller
pub struct MenuSystem {
    /// Current menu state
    pub state: MenuState,
    /// Game settings
    pub settings: GameSettings,
    /// Leaderboard data
    pub leaderboard: Leaderboard,
    /// Currently selected menu option
    pub selected_option: usize,
    /// Name being entered for high score
    pub name_input: String,
    /// Leaderboard scroll position
    pub leaderboard_scroll: usize,
    /// Animation timer for various effects
    pub animation_timer: f64,
}

impl MenuSystem {
    /// Create a new menu system
    pub fn new() -> Self {
        let settings_path = GameSettings::default_path();
        let leaderboard_path = Leaderboard::default_path();
        
        Self {
            state: MenuState::Main,
            settings: GameSettings::load_or_default(settings_path),
            leaderboard: Leaderboard::load_or_create(leaderboard_path),
            selected_option: 0,
            name_input: String::new(),
            leaderboard_scroll: 0,
            animation_timer: 0.0,
        }
    }
    
    /// Update the menu system
    pub fn update(&mut self, delta_time: f64) {
        self.animation_timer += delta_time;
    }
    
    /// Handle input for the current menu state
    pub fn handle_input(&mut self) -> MenuAction {
        match self.state {
            MenuState::Main => self.handle_main_menu_input(),
            MenuState::Leaderboard => self.handle_leaderboard_input(),
            MenuState::Settings => self.handle_settings_input(),
            MenuState::NameEntry { .. } => self.handle_name_entry_input(),
        }
    }
    
    /// Handle input for the main menu
    fn handle_main_menu_input(&mut self) -> MenuAction {
        let menu_options = self.get_main_menu_options();
        let num_options = menu_options.len();
        
        // Navigate menu
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.selected_option = if self.selected_option == 0 {
                num_options - 1
            } else {
                self.selected_option - 1
            };
        }
        
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.selected_option = (self.selected_option + 1) % num_options;
        }
        
        // Select option
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            match self.selected_option {
                0 => MenuAction::NewGame,
                1 => {
                    if Game::save_file_exists(&Game::default_save_path()) {
                        MenuAction::LoadGame
                    } else {
                        MenuAction::NewGame
                    }
                },
                2 => {
                    self.state = MenuState::Leaderboard;
                    self.leaderboard_scroll = 0;
                    MenuAction::None
                },
                3 => {
                    self.state = MenuState::Settings;
                    self.selected_option = 0;
                    MenuAction::None
                },
                4 => MenuAction::Quit,
                _ => MenuAction::None,
            }
        } else if is_key_pressed(KeyCode::Escape) {
            MenuAction::Quit
        } else {
            MenuAction::None
        }
    }
    
    /// Handle input for the leaderboard screen
    fn handle_leaderboard_input(&mut self) -> MenuAction {
        if is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Enter) {
            self.state = MenuState::Main;
            self.selected_option = 2; // Return to leaderboard option
        }
        
        // Scroll leaderboard if needed
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            if self.leaderboard_scroll > 0 {
                self.leaderboard_scroll -= 1;
            }
        }
        
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            let max_scroll = self.leaderboard.entries.len().saturating_sub(7); // Show 7 entries at a time
            if self.leaderboard_scroll < max_scroll {
                self.leaderboard_scroll += 1;
            }
        }
        
        MenuAction::None
    }
    
    /// Handle input for the settings screen
    fn handle_settings_input(&mut self) -> MenuAction {
        if is_key_pressed(KeyCode::Escape) {
            self.state = MenuState::Main;
            self.selected_option = 3; // Return to settings option
            // Save settings when leaving
            if let Err(e) = self.settings.save_to_file(&GameSettings::default_path()) {
                log::warn!("Failed to save settings: {}", e);
            }
        }
        
        // Navigate settings
        if is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W) {
            self.selected_option = if self.selected_option == 0 { 1 } else { 0 };
        }
        
        if is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S) {
            self.selected_option = (self.selected_option + 1) % 2;
        }
        
        // Modify settings
        if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
            match self.selected_option {
                0 => {
                    // Toggle sound
                    self.settings.sound_enabled = !self.settings.sound_enabled;
                },
                1 => {
                    // This could cycle through volume levels or we could add left/right for fine control
                    if is_key_down(KeyCode::LeftShift) {
                        self.settings.volume = (self.settings.volume - 0.1).max(0.0);
                    } else {
                        self.settings.volume = (self.settings.volume + 0.1).min(1.0);
                    }
                },
                _ => {},
            }
        }
        
        // Volume adjustment with left/right arrows
        if self.selected_option == 1 {
            if is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A) {
                self.settings.volume = (self.settings.volume - 0.1).max(0.0);
            }
            if is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D) {
                self.settings.volume = (self.settings.volume + 0.1).min(1.0);
            }
        }
        
        MenuAction::None
    }
    
    /// Handle input for name entry screen
    fn handle_name_entry_input(&mut self) -> MenuAction {
        // Handle character input
        if let Some(character) = get_char_pressed() {
            if character.is_ascii_alphanumeric() || character == ' ' {
                if self.name_input.len() < 20 { // Limit name length
                    self.name_input.push(character.to_ascii_uppercase());
                }
            }
        }
        
        // Handle backspace
        if is_key_pressed(KeyCode::Backspace) {
            self.name_input.pop();
        }
        
        // Handle enter (submit name)
        if is_key_pressed(KeyCode::Enter) {
            if let MenuState::NameEntry { score, level, lines_cleared, game_time } = self.state {
                let name = if self.name_input.is_empty() {
                    "ANONYMOUS".to_string()
                } else {
                    self.name_input.clone()
                };
                
                // Add to leaderboard
                let entry = crate::leaderboard::LeaderboardEntry::new(
                    name, score, level, lines_cleared, game_time
                );
                
                if let Some(position) = self.leaderboard.add_entry(entry) {
                    log::info!("New high score! Position: {}", position);
                }
                
                // Save leaderboard
                if let Err(e) = self.leaderboard.save_to_file(&Leaderboard::default_path()) {
                    log::warn!("Failed to save leaderboard: {}", e);
                }
                
                // Return to main menu
                self.state = MenuState::Main;
                self.selected_option = 0;
                self.name_input.clear();
            }
        }
        
        // Handle escape (cancel name entry)
        if is_key_pressed(KeyCode::Escape) {
            self.state = MenuState::Main;
            self.selected_option = 0;
            self.name_input.clear();
        }
        
        MenuAction::None
    }
    
    /// Get the main menu options based on current state
    fn get_main_menu_options(&self) -> Vec<&str> {
        let mut options = vec!["🎮 NEW GAME"];
        
        if Game::save_file_exists(&Game::default_save_path()) {
            options.push("💾 CONTINUE");
        } else {
            options.push("💾 CONTINUE (No Save)");
        }
        
        options.extend_from_slice(&[
            "🏆 LEADERBOARD",
            "⚙️  SETTINGS",
            "❌ QUIT",
        ]);
        
        options
    }
    
    /// Check if a score qualifies for high score entry
    pub fn check_high_score(&mut self, score: u32, level: u32, lines_cleared: u32, game_time: f64) -> bool {
        if self.leaderboard.qualifies_for_leaderboard(score) {
            self.state = MenuState::NameEntry { score, level, lines_cleared, game_time };
            self.name_input.clear();
            true
        } else {
            false
        }
    }
    
    /// Render the current menu state
    pub fn render(&self, background_texture: &Texture2D) {
        match self.state {
            MenuState::Main => self.render_main_menu(background_texture),
            MenuState::Leaderboard => self.render_leaderboard(background_texture),
            MenuState::Settings => self.render_settings(background_texture),
            MenuState::NameEntry { score, level, lines_cleared, game_time } => {
                self.render_name_entry(background_texture, score, level, lines_cleared, game_time)
            },
        }
    }
    
    /// Render the main menu
    fn render_main_menu(&self, background_texture: &Texture2D) {
        // Clear screen and draw background
        clear_background(Color::new(0.02, 0.02, 0.08, 1.0));
        
        draw_texture(background_texture, 0.0, 0.0, WHITE);
        
        // Draw semi-transparent overlay
        draw_rectangle(
            0.0,
            0.0,
            WINDOW_WIDTH as f32,
            WINDOW_HEIGHT as f32,
            Color::new(0.0, 0.0, 0.0, 0.4),
        );
        
        // Draw animated title
        self.draw_animated_title();
        
        // Draw menu options
        let options = self.get_main_menu_options();
        let option_size = 28.0;
        let option_y_start = 320.0;
        let option_spacing = 55.0;
        
        for (i, option) in options.iter().enumerate() {
            let is_selected = i == self.selected_option;
            let option_width = measure_text(option, None, option_size as u16, 1.0).width;
            let option_x = (WINDOW_WIDTH as f32 - option_width) / 2.0;
            let option_y = option_y_start + (i as f32 * option_spacing);
            
            // Draw selection highlight
            if is_selected {
                let pulse = (self.animation_timer * 3.0).sin() * 0.3 + 0.7;
                draw_rectangle(
                    option_x - 20.0,
                    option_y - option_size - 5.0,
                    option_width + 40.0,
                    option_size + 10.0,
                    Color::new(0.2, 0.4, 1.0, 0.3 * pulse as f32),
                );
            }
            
            // Color based on option type and selection
            let color = if is_selected {
                let pulse = (self.animation_timer * 4.0).sin() * 0.2 + 0.8;
                Color::new(1.0, 1.0, 0.8, pulse as f32)
            } else {
                match i {
                    0 => Color::new(0.4, 1.0, 0.4, 0.9), // Green for new game
                    1 => {
                        if Game::save_file_exists(&Game::default_save_path()) {
                            Color::new(0.4, 0.8, 1.0, 0.9) // Blue for continue
                        } else {
                            Color::new(0.6, 0.6, 0.6, 0.6) // Gray for no save
                        }
                    },
                    2 => Color::new(1.0, 0.8, 0.2, 0.9), // Gold for leaderboard
                    3 => Color::new(0.8, 0.4, 1.0, 0.9), // Purple for settings
                    4 => Color::new(1.0, 0.4, 0.4, 0.9), // Red for quit
                    _ => Color::new(0.8, 0.8, 0.8, 0.9),
                }
            };
            
            // Draw option with outline
            self.draw_text_with_outline(option, option_x, option_y, option_size, color);
        }
        
        // Draw animated particles
        self.draw_menu_particles();
    }
    
    /// Render the leaderboard screen
    fn render_leaderboard(&self, background_texture: &Texture2D) {
        // Clear screen and draw background
        clear_background(Color::new(0.02, 0.02, 0.08, 1.0));
        draw_texture(background_texture, 0.0, 0.0, WHITE);
        
        // Draw semi-transparent overlay
        draw_rectangle(
            0.0,
            0.0,
            WINDOW_WIDTH as f32,
            WINDOW_HEIGHT as f32,
            Color::new(0.0, 0.0, 0.0, 0.6),
        );
        
        // Draw title
        let title = "🏆 HIGH SCORES 🏆";
        let title_size = 48.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        let title_x = (WINDOW_WIDTH as f32 - title_width) / 2.0;
        let title_y = 100.0;
        
        self.draw_text_with_outline(title, title_x, title_y, title_size, Color::new(1.0, 0.9, 0.3, 1.0));
        
        // Draw leaderboard entries
        let entry_size = 24.0;
        let entry_y_start = 180.0;
        let entry_spacing = 45.0;
        
        if self.leaderboard.entries.is_empty() {
            // No scores yet
            let no_scores = "No high scores yet! Be the first!";
            let text_width = measure_text(no_scores, None, entry_size as u16, 1.0).width;
            let text_x = (WINDOW_WIDTH as f32 - text_width) / 2.0;
            let text_y = WINDOW_HEIGHT as f32 / 2.0;
            
            self.draw_text_with_outline(no_scores, text_x, text_y, entry_size, Color::new(0.8, 0.8, 0.8, 0.8));
        } else {
            // Draw header
            let header = "RANK  PLAYER NAME        SCORE     LVL  LINES  TIME";
            let header_x = 80.0;
            let header_y = entry_y_start - 20.0;
            
            self.draw_text_with_outline(header, header_x, header_y, 18.0, Color::new(0.6, 0.8, 1.0, 1.0));
            
            // Draw entries (with scrolling)
            let visible_entries = 7;
            let start_idx = self.leaderboard_scroll;
            let end_idx = (start_idx + visible_entries).min(self.leaderboard.entries.len());
            
            for (display_idx, entry_idx) in (start_idx..end_idx).enumerate() {
                let entry = &self.leaderboard.entries[entry_idx];
                let rank = entry_idx + 1;
                
                let entry_x = 80.0;
                let entry_y = entry_y_start + (display_idx as f32 * entry_spacing);
                
                // Format entry text
                let entry_text = format!(
                    "{:2}    {:16} {:8}    {:2}   {:3}   {}",
                    rank,
                    entry.name,
                    entry.score,
                    entry.level,
                    entry.lines_cleared,
                    entry.formatted_time()
                );
                
                // Color based on rank
                let color = match rank {
                    1 => Color::new(1.0, 0.85, 0.0, 1.0), // Gold
                    2 => Color::new(0.75, 0.75, 0.75, 1.0), // Silver
                    3 => Color::new(0.8, 0.5, 0.2, 1.0), // Bronze
                    _ => Color::new(0.8, 0.8, 0.8, 0.9), // White
                };
                
                self.draw_text_with_outline(&entry_text, entry_x, entry_y, entry_size, color);
            }
            
            // Draw scroll indicators if needed
            if self.leaderboard_scroll > 0 {
                let up_arrow = "▲ More above";
                self.draw_text_with_outline(up_arrow, 80.0, entry_y_start - 50.0, 16.0, Color::new(0.8, 0.8, 0.8, 0.7));
            }
            
            if end_idx < self.leaderboard.entries.len() {
                let down_arrow = "▼ More below";
                self.draw_text_with_outline(down_arrow, 80.0, entry_y_start + (visible_entries as f32 * entry_spacing) + 20.0, 16.0, Color::new(0.8, 0.8, 0.8, 0.7));
            }
        }
        
        // Draw instructions
        let instruction = "Press ESCAPE or ENTER to return to main menu";
        let inst_width = measure_text(instruction, None, 20, 1.0).width;
        let inst_x = (WINDOW_WIDTH as f32 - inst_width) / 2.0;
        let inst_y = WINDOW_HEIGHT as f32 - 50.0;
        
        self.draw_text_with_outline(instruction, inst_x, inst_y, 20.0, Color::new(0.7, 0.7, 0.7, 0.8));
    }
    
    /// Render the settings screen
    fn render_settings(&self, background_texture: &Texture2D) {
        // Clear screen and draw background
        clear_background(Color::new(0.02, 0.02, 0.08, 1.0));
        draw_texture(background_texture, 0.0, 0.0, WHITE);
        
        // Draw semi-transparent overlay
        draw_rectangle(
            0.0,
            0.0,
            WINDOW_WIDTH as f32,
            WINDOW_HEIGHT as f32,
            Color::new(0.0, 0.0, 0.0, 0.6),
        );
        
        // Draw title
        let title = "⚙️ SETTINGS ⚙️";
        let title_size = 48.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        let title_x = (WINDOW_WIDTH as f32 - title_width) / 2.0;
        let title_y = 150.0;
        
        self.draw_text_with_outline(title, title_x, title_y, title_size, Color::new(0.8, 0.4, 1.0, 1.0));
        
        // Draw settings options
        let option_size = 32.0;
        let option_y_start = 280.0;
        let option_spacing = 80.0;
        
        // Sound setting
        let sound_text = format!("🔊 SOUND: {}", if self.settings.sound_enabled { "ON" } else { "OFF" });
        let sound_x = (WINDOW_WIDTH as f32 - measure_text(&sound_text, None, option_size as u16, 1.0).width) / 2.0;
        let sound_y = option_y_start;
        let sound_selected = self.selected_option == 0;
        
        if sound_selected {
            let pulse = (self.animation_timer * 3.0).sin() * 0.3 + 0.7;
            draw_rectangle(
                sound_x - 20.0,
                sound_y - option_size - 5.0,
                measure_text(&sound_text, None, option_size as u16, 1.0).width + 40.0,
                option_size + 10.0,
                Color::new(0.2, 0.4, 1.0, 0.3 * pulse as f32),
            );
        }
        
        let sound_color = if sound_selected {
            let pulse = (self.animation_timer * 4.0).sin() * 0.2 + 0.8;
            Color::new(1.0, 1.0, 0.8, pulse as f32)
        } else {
            if self.settings.sound_enabled {
                Color::new(0.4, 1.0, 0.4, 0.9)
            } else {
                Color::new(1.0, 0.4, 0.4, 0.9)
            }
        };
        
        self.draw_text_with_outline(&sound_text, sound_x, sound_y, option_size, sound_color);
        
        // Volume setting
        let volume_text = format!("🎵 VOLUME: {:.0}%", self.settings.volume * 100.0);
        let volume_x = (WINDOW_WIDTH as f32 - measure_text(&volume_text, None, option_size as u16, 1.0).width) / 2.0;
        let volume_y = option_y_start + option_spacing;
        let volume_selected = self.selected_option == 1;
        
        if volume_selected {
            let pulse = (self.animation_timer * 3.0).sin() * 0.3 + 0.7;
            draw_rectangle(
                volume_x - 20.0,
                volume_y - option_size - 5.0,
                measure_text(&volume_text, None, option_size as u16, 1.0).width + 40.0,
                option_size + 10.0,
                Color::new(0.2, 0.4, 1.0, 0.3 * pulse as f32),
            );
        }
        
        let volume_color = if volume_selected {
            let pulse = (self.animation_timer * 4.0).sin() * 0.2 + 0.8;
            Color::new(1.0, 1.0, 0.8, pulse as f32)
        } else {
            Color::new(0.4, 0.8, 1.0, 0.9)
        };
        
        self.draw_text_with_outline(&volume_text, volume_x, volume_y, option_size, volume_color);
        
        // Draw volume bar
        if volume_selected {
            let bar_width = 300.0;
            let bar_height = 10.0;
            let bar_x = (WINDOW_WIDTH as f32 - bar_width) / 2.0;
            let bar_y = volume_y + 30.0;
            
            // Background bar
            draw_rectangle(bar_x, bar_y, bar_width, bar_height, Color::new(0.3, 0.3, 0.3, 0.8));
            
            // Volume fill
            let fill_width = bar_width * self.settings.volume;
            draw_rectangle(bar_x, bar_y, fill_width, bar_height, Color::new(0.4, 0.8, 1.0, 0.9));
            
            // Instructions
            let instruction = "Use LEFT/RIGHT arrows to adjust volume";
            let inst_width = measure_text(instruction, None, 18, 1.0).width;
            let inst_x = (WINDOW_WIDTH as f32 - inst_width) / 2.0;
            let inst_y = bar_y + 40.0;
            
            self.draw_text_with_outline(instruction, inst_x, inst_y, 18.0, Color::new(0.7, 0.7, 0.7, 0.8));
        }
        
        // Draw general instructions
        let instruction = "Press ESCAPE to return to main menu";
        let inst_width = measure_text(instruction, None, 20, 1.0).width;
        let inst_x = (WINDOW_WIDTH as f32 - inst_width) / 2.0;
        let inst_y = WINDOW_HEIGHT as f32 - 50.0;
        
        self.draw_text_with_outline(instruction, inst_x, inst_y, 20.0, Color::new(0.7, 0.7, 0.7, 0.8));
    }
    
    /// Render the name entry screen
    fn render_name_entry(&self, background_texture: &Texture2D, score: u32, level: u32, lines_cleared: u32, game_time: f64) {
        // Clear screen and draw background
        clear_background(Color::new(0.02, 0.02, 0.08, 1.0));
        draw_texture(background_texture, 0.0, 0.0, WHITE);
        
        // Draw semi-transparent overlay
        draw_rectangle(
            0.0,
            0.0,
            WINDOW_WIDTH as f32,
            WINDOW_HEIGHT as f32,
            Color::new(0.0, 0.0, 0.0, 0.7),
        );
        
        // Draw congratulations title
        let title = "🎉 NEW HIGH SCORE! 🎉";
        let title_size = 56.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        let title_x = (WINDOW_WIDTH as f32 - title_width) / 2.0;
        let title_y = 120.0;
        
        // Animated title with rainbow colors
        let time_offset = self.animation_timer * 2.0;
        for (i, c) in title.chars().enumerate() {
            let char_x = title_x + (i as f32 * title_size * 0.6);
            let hue = ((i as f64 * 0.2) + time_offset) % 6.0;
            let rainbow_color = self.hsv_to_rgb(hue, 1.0, 1.0);
            let bounce = (self.animation_timer * 3.0 + i as f64 * 0.3).sin() * 5.0;
            
            self.draw_text_with_outline(&c.to_string(), char_x, title_y + bounce as f32, title_size, rainbow_color);
        }
        
        // Draw score details
        let minutes = (game_time / 60.0) as u32;
        let seconds = (game_time % 60.0) as u32;
        let details = format!(
            "Score: {}  •  Level: {}  •  Lines: {}  •  Time: {}:{:02}",
            score,
            level,
            lines_cleared,
            minutes,
            seconds
        );
        
        let details_size = 24.0;
        let details_width = measure_text(&details, None, details_size as u16, 1.0).width;
        let details_x = (WINDOW_WIDTH as f32 - details_width) / 2.0;
        let details_y = 200.0;
        
        self.draw_text_with_outline(&details, details_x, details_y, details_size, Color::new(0.8, 0.8, 1.0, 1.0));
        
        // Draw name entry prompt
        let prompt = "Enter your name:";
        let prompt_size = 32.0;
        let prompt_width = measure_text(prompt, None, prompt_size as u16, 1.0).width;
        let prompt_x = (WINDOW_WIDTH as f32 - prompt_width) / 2.0;
        let prompt_y = 280.0;
        
        self.draw_text_with_outline(prompt, prompt_x, prompt_y, prompt_size, Color::new(1.0, 1.0, 0.8, 1.0));
        
        // Draw name input box
        let input_box_width = 400.0;
        let input_box_height = 60.0;
        let input_box_x = (WINDOW_WIDTH as f32 - input_box_width) / 2.0;
        let input_box_y = 320.0;
        
        // Input box background
        draw_rectangle(
            input_box_x,
            input_box_y,
            input_box_width,
            input_box_height,
            Color::new(0.1, 0.1, 0.2, 0.8),
        );
        
        // Input box border
        draw_rectangle_lines(
            input_box_x,
            input_box_y,
            input_box_width,
            input_box_height,
            3.0,
            Color::new(0.4, 0.8, 1.0, 1.0),
        );
        
        // Draw typed name or placeholder
        let display_text = if self.name_input.is_empty() {
            "ANONYMOUS"
        } else {
            &self.name_input
        };
        
        let cursor = if (self.animation_timer * 2.0) as i32 % 2 == 0 && !self.name_input.is_empty() {
            "_"
        } else {
            ""
        };
        
        let input_text = format!("{}{}", display_text, cursor);
        let input_size = 28.0;
        let input_width = measure_text(&input_text, None, input_size as u16, 1.0).width;
        let input_x = input_box_x + (input_box_width - input_width) / 2.0;
        let input_y = input_box_y + 40.0;
        
        let input_color = if self.name_input.is_empty() {
            Color::new(0.6, 0.6, 0.6, 0.8) // Gray for placeholder
        } else {
            Color::new(1.0, 1.0, 1.0, 1.0) // White for actual input
        };
        
        self.draw_text_with_outline(&input_text, input_x, input_y, input_size, input_color);
        
        // Draw instructions
        let instruction = "Press ENTER to confirm • ESCAPE to cancel";
        let inst_width = measure_text(instruction, None, 20, 1.0).width;
        let inst_x = (WINDOW_WIDTH as f32 - inst_width) / 2.0;
        let inst_y = WINDOW_HEIGHT as f32 - 80.0;
        
        self.draw_text_with_outline(instruction, inst_x, inst_y, 20.0, Color::new(0.7, 0.7, 0.7, 0.8));
        
        // Show predicted rank
        if let Some(rank) = self.leaderboard.get_rank_for_score(score) {
            let rank_text = format!("This will be rank #{} on the leaderboard!", rank);
            let rank_width = measure_text(&rank_text, None, 22, 1.0).width;
            let rank_x = (WINDOW_WIDTH as f32 - rank_width) / 2.0;
            let rank_y = 450.0;
            
            self.draw_text_with_outline(&rank_text, rank_x, rank_y, 22.0, Color::new(1.0, 0.9, 0.3, 1.0));
        }
    }
    
    /// Draw animated title for main menu
    fn draw_animated_title(&self) {
        let title = "RUST TETRIS";
        let title_size = 72.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        let title_x = (WINDOW_WIDTH as f32 - title_width) / 2.0;
        let title_y = 150.0;
        
        // Draw each letter with a wave effect and rainbow colors
        let time_offset = self.animation_timer * 2.0;
        for (i, c) in title.chars().enumerate() {
            if c == ' ' {
                continue; // Skip spaces
            }
            
            let char_x = title_x + (i as f32 * title_size * 0.65);
            let wave_offset = (self.animation_timer * 2.0 + i as f64 * 0.5).sin() * 10.0;
            let char_y = title_y + wave_offset as f32;
            
            // Rainbow color
            let hue = ((i as f64 * 0.3) + time_offset) % 6.0;
            let rainbow_color = self.hsv_to_rgb(hue, 0.9, 1.0);
            
            // Draw with outline
            self.draw_text_with_outline(&c.to_string(), char_x, char_y, title_size, rainbow_color);
        }
        
        // Draw subtitle
        let subtitle = "Enhanced Edition with Leaderboards";
        let subtitle_size = 24.0;
        let subtitle_width = measure_text(subtitle, None, subtitle_size as u16, 1.0).width;
        let subtitle_x = (WINDOW_WIDTH as f32 - subtitle_width) / 2.0;
        let subtitle_y = 210.0;
        
        self.draw_text_with_outline(subtitle, subtitle_x, subtitle_y, subtitle_size, Color::new(0.8, 0.8, 1.0, 0.9));
    }
    
    /// Draw animated particles for menu background
    fn draw_menu_particles(&self) {
        let time = self.animation_timer as f32;
        for i in 0..30 {
            let particle_phase = (time * 0.2 + i as f32 * 0.3) % 6.28;
            let x_base = (WINDOW_WIDTH as f32 / 30.0) * (i as f32 + 1.0);
            let y_offset = (particle_phase.sin() * 40.0) + (time * 0.15 + i as f32 * 0.2).sin() * 20.0;
            let y_pos = 80.0 + y_offset + (i as f32 * 15.0);
            
            // Vary particle colors
            let hue = (i as f64 * 0.2 + time as f64 * 0.3) % 6.0;
            let particle_color = self.hsv_to_rgb(hue, 0.7, 0.8);
            let alpha = (0.2 + 0.3 * ((time * 0.5 + i as f32 * 0.4).sin() * 0.5 + 0.5)) * 0.6;
            
            let size = 1.5 + ((time * 0.4 + i as f32 * 0.3).cos() * 0.5 + 0.5) * 2.5;
            
            draw_rectangle(
                x_base - size / 2.0,
                y_pos - size / 2.0,
                size,
                size,
                Color::new(particle_color.r, particle_color.g, particle_color.b, alpha),
            );
        }
    }
    
    /// Draw text with outline for better visibility
    fn draw_text_with_outline(&self, text: &str, x: f32, y: f32, size: f32, color: Color) {
        // Draw outline
        let outline_color = Color::new(0.0, 0.0, 0.0, color.a * 0.8);
        for offset_x in [-2.0, -1.0, 0.0, 1.0, 2.0] {
            for offset_y in [-2.0, -1.0, 0.0, 1.0, 2.0] {
                if offset_x != 0.0 || offset_y != 0.0 {
                    draw_text(text, x + offset_x, y + offset_y, size, outline_color);
                }
            }
        }
        
        // Draw main text
        draw_text(text, x, y, size, color);
    }
    
    /// Convert HSV to RGB color (helper function)
    fn hsv_to_rgb(&self, h: f64, s: f64, v: f64) -> Color {
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
}

/// Actions the menu system can request
#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    /// Do nothing
    None,
    /// Start a new game
    NewGame,
    /// Load saved game
    LoadGame,
    /// Quit the application
    Quit,
}

impl Default for MenuSystem {
    fn default() -> Self {
        Self::new()
    }
}