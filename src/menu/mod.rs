//! Enhanced start menu system for the Tetris game

use macroquad::prelude::*;
use crate::graphics::text::{draw_text, measure_text};
use crate::game::config::*;
use crate::leaderboard::Leaderboard;
use crate::Game;
use crate::input::touch::{canvas_tap, key_down, key_pressed};
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

/// The touch leaderboard's table: left edge, width and rows, in canvas units.
const TOUCH_TABLE_X: f32 = 250.0;
const TOUCH_TABLE_W: f32 = 400.0;
const TOUCH_TABLE_ROWS: usize = 8;

/// Game settings that persist across sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSettings {
    /// Whether sound is enabled
    pub sound_enabled: bool,
    /// Master volume (0.0 to 1.0)
    pub volume: f32,
    /// The touch gesture hint has been shown (it's shown over the first game only).
    #[serde(default)]
    pub gesture_hint_seen: bool,
    /// Play in the legacy (terminal-style) look. New games start in it; the in-game toggle
    /// (L, or the pause screen's button) changes it too.
    #[serde(default)]
    pub legacy_mode: bool,
    /// Haptics on iPhone (on unless turned off).
    #[serde(default = "on")]
    pub haptics_enabled: bool,
}

fn on() -> bool {
    true
}

impl GameSettings {
    /// Create default settings
    pub fn default() -> Self {
        Self {
            sound_enabled: true,
            volume: 0.7,
            gesture_hint_seen: false,
            legacy_mode: false,
            haptics_enabled: true,
        }
    }
    
    /// Get the default settings file path
    pub fn default_path() -> std::path::PathBuf {
        crate::platform::data_file("tetris_settings.json")
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
        if key_pressed(KeyCode::Up) || key_pressed(KeyCode::W) {
            self.selected_option = if self.selected_option == 0 {
                num_options - 1
            } else {
                self.selected_option - 1
            };
        }
        
        if key_pressed(KeyCode::Down) || key_pressed(KeyCode::S) {
            self.selected_option = (self.selected_option + 1) % num_options;
        }
        
        // Tapping an option picks it (rows are 55 apart, text baseline at 320 + 55i)
        let mut tapped = false;
        if let Some(tap) = canvas_tap() {
            let row = ((tap.y - (320.0 - 14.0)) / 55.0).round();
            if row >= 0.0 && (row as usize) < num_options && (150.0..750.0).contains(&tap.x) {
                self.selected_option = row as usize;
                tapped = true;
            }
        }

        // Select option
        if tapped || key_pressed(KeyCode::Enter) || key_pressed(KeyCode::Space) {
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
        } else if key_pressed(KeyCode::Escape) {
            MenuAction::Quit
        } else {
            MenuAction::None
        }
    }
    
    /// Handle input for the leaderboard screen
    fn handle_leaderboard_input(&mut self) -> MenuAction {
        if key_pressed(KeyCode::Escape) || key_pressed(KeyCode::Enter) || canvas_tap().is_some() {
            self.state = MenuState::Main;
            self.selected_option = 2; // Return to leaderboard option
            return MenuAction::None;
        }
        
        // Scroll leaderboard if needed
        if key_pressed(KeyCode::Up) || key_pressed(KeyCode::W) {
            if self.leaderboard_scroll > 0 {
                self.leaderboard_scroll -= 1;
            }
        }
        
        if key_pressed(KeyCode::Down) || key_pressed(KeyCode::S) {
            let max_scroll = self.leaderboard.entries.len().saturating_sub(7); // Show 7 entries at a time
            if self.leaderboard_scroll < max_scroll {
                self.leaderboard_scroll += 1;
            }
        }
        
        MenuAction::None
    }
    
    /// Handle input for the settings screen
    fn handle_settings_input(&mut self) -> MenuAction {
        // Touch: tap sound, volume, legacy or haptics (see draw_touch_settings) to change it; tap
        // anywhere else to go back.
        let mut tap_back = false;
        if let Some(tap) = canvas_tap() {
            if (200.0..288.0).contains(&tap.y) {
                self.selected_option = 0;
                self.settings.sound_enabled = !self.settings.sound_enabled;
            } else if (288.0..392.0).contains(&tap.y) {
                self.selected_option = 1;
                let next = self.settings.volume + 0.1;
                self.settings.volume = if next > 1.05 { 0.0 } else { next.min(1.0) };
            } else if (392.0..472.0).contains(&tap.y) {
                self.selected_option = 2;
                self.settings.legacy_mode = !self.settings.legacy_mode;
            } else if (472.0..552.0).contains(&tap.y) {
                self.settings.haptics_enabled = !self.settings.haptics_enabled;
            } else {
                tap_back = true;
            }
        }

        if tap_back || key_pressed(KeyCode::Escape) {
            self.state = MenuState::Main;
            self.selected_option = 3; // Return to settings option
            // Save settings when leaving
            if let Err(e) = self.settings.save_to_file(&GameSettings::default_path()) {
                log::warn!("Failed to save settings: {}", e);
            }
        }
        
        // Navigate settings
        if key_pressed(KeyCode::Up) || key_pressed(KeyCode::W) {
            self.selected_option = (self.selected_option + 2) % 3;
        }
        
        if key_pressed(KeyCode::Down) || key_pressed(KeyCode::S) {
            self.selected_option = (self.selected_option + 1) % 3;
        }
        
        // Modify settings
        if key_pressed(KeyCode::Enter) || key_pressed(KeyCode::Space) {
            match self.selected_option {
                0 => {
                    // Toggle sound
                    self.settings.sound_enabled = !self.settings.sound_enabled;
                },
                1 => {
                    // This could cycle through volume levels or we could add left/right for fine control
                    if key_down(KeyCode::LeftShift) {
                        self.settings.volume = (self.settings.volume - 0.1).max(0.0);
                    } else {
                        self.settings.volume = (self.settings.volume + 0.1).min(1.0);
                    }
                },
                2 => self.settings.legacy_mode = !self.settings.legacy_mode,
                _ => {},
            }
        }
        
        // Volume adjustment with left/right arrows
        if self.selected_option == 1 {
            if key_pressed(KeyCode::Left) || key_pressed(KeyCode::A) {
                self.settings.volume = (self.settings.volume - 0.1).max(0.0);
            }
            if key_pressed(KeyCode::Right) || key_pressed(KeyCode::D) {
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
        if key_pressed(KeyCode::Backspace) {
            self.name_input.pop();
        }
        
        // Handle enter (submit name)
        if key_pressed(KeyCode::Enter) {
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
        if key_pressed(KeyCode::Escape) {
            self.state = MenuState::Main;
            self.selected_option = 0;
            self.name_input.clear();
        }
        
        MenuAction::None
    }
    
    /// Get the main menu options based on current state
    fn get_main_menu_options(&self) -> Vec<&str> {
        let mut options = vec!["NEW GAME"];
        
        if Game::save_file_exists(&Game::default_save_path()) {
            options.push("CONTINUE");
        } else {
            options.push("CONTINUE (NO SAVE)");
        }
        
        options.extend_from_slice(&[
            "LEADERBOARD",
            "SETTINGS",
        ]);
        if crate::platform::CAN_QUIT {
            options.push("QUIT");
        }
        
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
        self.draw_background(background_texture, 0.4);
        
        // Draw animated title
        self.draw_animated_title();
        
        // Draw menu options
        let options = self.get_main_menu_options();
        let option_size = 28.0;
        let option_y_start = 320.0;
        let option_spacing = 55.0;
        
        if crate::platform::is_touch_device() {
            self.draw_touch_options(&options, option_y_start, option_spacing, option_size);
            self.draw_menu_particles();
            return;
        }

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
    
    /// Touch screens: each option a glowing glass button (the rows the taps pick, see
    /// handle_main_menu_input), NEW GAME lit up as the main one.
    fn draw_touch_options(&self, options: &[&str], y_start: f32, spacing: f32, size: f32) {
        use crate::graphics::style::{self, alpha, mix, CYAN, GOLD, GREEN, PINK, VIOLET};
        let has_save = Game::save_file_exists(&Game::default_save_path());
        for (i, option) in options.iter().enumerate() {
            let baseline = y_start + i as f32 * spacing;
            let r = Rect::new(WINDOW_WIDTH as f32 / 2.0 - 180.0, baseline - size - 10.0, 360.0, size + 20.0);
            let accent = match i {
                0 => GREEN,
                1 if has_save => CYAN,
                1 => Color::new(0.6, 0.6, 0.75, 1.0),
                2 => GOLD,
                3 => VIOLET,
                _ => PINK,
            };
            let dim = i == 1 && !has_save;
            style::panel(r, if dim { alpha(accent, 0.5) } else { accent }, 1.0);
            if i == 0 {
                let pulse = ((self.animation_timer * 2.5).sin() * 0.5 + 0.5) as f32;
                style::halo(r, 10.0, 10.0, accent, 0.18 + 0.12 * pulse);
                style::round_rect(r, 10.0, alpha(mix(accent, CYAN, 0.3), 0.55), alpha(accent, 0.30));
            }
            let w = measure_text(option, None, size as u16, 1.0).width;
            let x = r.center().x - w / 2.0;
            let color = if dim { Color::new(0.7, 0.7, 0.8, 0.6) } else { mix(accent, WHITE, 0.55) };
            draw_text(option, x + 2.0, baseline + 2.0, size, Color::new(0.0, 0.0, 0.0, 0.45));
            draw_text(option, x, baseline, size, color);
        }
    }

    /// Touch settings: sound, volume, legacy and haptics as glass buttons (the bands the taps use, see
    /// handle_settings_input), then the way back and the display info.
    fn draw_touch_settings(&self) {
        use crate::graphics::style::{self, alpha, mix, CYAN, GREEN, PINK};
        let cx = WINDOW_WIDTH as f32 / 2.0;
        let size = 30.0;

        // A label and an on/off switch.
        let switch = |label: &str, y: f32, on: bool, on_color: Color, off_color: Color| {
            let accent = if on { on_color } else { off_color };
            let r = Rect::new(cx - 180.0, y, 360.0, 64.0);
            style::panel(r, accent, 1.0);
            let baseline = r.center().y + size * 0.35;
            draw_text(label, r.x + 24.0, baseline, size, mix(accent, WHITE, 0.6));
            let sw = Rect::new(r.right() - 24.0 - 84.0, r.center().y - 17.0, 84.0, 34.0);
            style::round_rect(sw, 17.0, alpha(accent, if on { 0.85 } else { 0.30 }), alpha(accent, if on { 0.65 } else { 0.20 }));
            let knob_x = if on { sw.right() - 17.0 } else { sw.x + 17.0 };
            draw_circle(knob_x, sw.center().y, 13.0, WHITE);
            let state = if on { "ON" } else { "OFF" };
            let tw = measure_text(state, None, 16, 1.0).width;
            let tx = if on { sw.x + 12.0 } else { sw.right() - 12.0 - tw };
            draw_text(state, tx, sw.center().y + 6.0, 16.0, WHITE);
        };

        switch("SOUND", 216.0, self.settings.sound_enabled, GREEN, PINK);

        // Volume: the percentage and a bar.
        let r = Rect::new(cx - 180.0, 296.0, 360.0, 88.0);
        style::panel(r, CYAN, 1.0);
        let pct = format!("{:.0}%", self.settings.volume * 100.0);
        draw_text("VOLUME", r.x + 24.0, r.y + 38.0, size, mix(CYAN, WHITE, 0.6));
        let pw = measure_text(&pct, None, size as u16, 1.0).width;
        draw_text(&pct, r.right() - 24.0 - pw, r.y + 38.0, size, WHITE);
        let bar = Rect::new(r.x + 24.0, r.y + 56.0, r.w - 48.0, 12.0);
        style::round_rect(bar, 6.0, Color::new(1.0, 1.0, 1.0, 0.12), Color::new(1.0, 1.0, 1.0, 0.08));
        if self.settings.volume > 0.0 {
            let fill = Rect::new(bar.x, bar.y, (bar.w * self.settings.volume).max(12.0), bar.h);
            style::round_rect(fill, 6.0, mix(CYAN, WHITE, 0.3), style::VIOLET);
        }

        // Legacy: the terminal-style look, for new games.
        switch("LEGACY", 400.0, self.settings.legacy_mode, style::VIOLET, style::VIOLET);

        // Haptics: the iPhone's taps and rumbles on drops, clears and fireballs.
        switch("HAPTICS", 480.0, self.settings.haptics_enabled, style::GOLD, PINK);

        let hint = "Tap to change  -  tap outside to go back";
        let hw = measure_text(hint, None, 18, 1.0).width;
        draw_text(hint, cx - hw / 2.0, 580.0, 18.0, Color::new(0.85, 0.87, 1.0, 0.75));

        let info = crate::platform::display_info();
        for (i, line) in info.lines().enumerate() {
            let w = measure_text(line, None, 11, 1.0).width;
            draw_text(line, cx - w / 2.0, 610.0 + i as f32 * 13.0, 11.0, Color::new(0.75, 0.78, 0.95, 0.6));
        }
    }

    /// Render the leaderboard screen
    fn render_leaderboard(&self, background_texture: &Texture2D) {
        let touch = crate::platform::is_touch_device();
        self.draw_background(background_texture, 0.6);
        
        // Draw title
        let title = "HIGH SCORES";
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
        } else if touch {
            // Touch screens are narrow: rank, name and score, bigger.
            let (rank_x, name_x, score_right) = (TOUCH_TABLE_X, TOUCH_TABLE_X + 60.0, TOUCH_TABLE_X + TOUCH_TABLE_W);
            let header = Color::new(0.6, 0.8, 1.0, 1.0);
            self.draw_text_with_outline("#", rank_x, entry_y_start - 20.0, 18.0, header);
            self.draw_text_with_outline("NAME", name_x, entry_y_start - 20.0, 18.0, header);
            let sw = measure_text("SCORE", None, 18, 1.0).width;
            self.draw_text_with_outline("SCORE", score_right - sw, entry_y_start - 20.0, 18.0, header);
            let size = 28.0;
            for (row, entry) in self.leaderboard.entries.iter().take(TOUCH_TABLE_ROWS).enumerate() {
                let y = entry_y_start + 22.0 + row as f32 * entry_spacing;
                let color = match row {
                    0 => Color::new(1.0, 0.85, 0.0, 1.0),
                    1 => Color::new(0.75, 0.75, 0.75, 1.0),
                    2 => Color::new(0.8, 0.5, 0.2, 1.0),
                    _ => Color::new(0.8, 0.8, 0.8, 0.9),
                };
                let accent = if row < 3 { color } else { crate::graphics::style::VIOLET };
                crate::graphics::style::panel(
                    Rect::new(TOUCH_TABLE_X - 14.0, y - 31.0, TOUCH_TABLE_W + 28.0, 40.0),
                    crate::graphics::style::alpha(accent, if row < 3 { 1.0 } else { 0.6 }),
                    0.8,
                );
                let mut name = entry.name.clone();
                name.truncate(10);
                let score = entry.score.to_string();
                self.draw_text_with_outline(&(row + 1).to_string(), rank_x, y, size, color);
                self.draw_text_with_outline(&name, name_x, y, size, color);
                let w = measure_text(&score, None, size as u16, 1.0).width;
                self.draw_text_with_outline(&score, score_right - w, y, size, color);
            }
        } else {
            // Draw header with fixed column positions
            // The table (about 520 wide) centred on the canvas.
            let base_x = (WINDOW_WIDTH as f32 - 520.0) / 2.0;
            let header_y = entry_y_start - 20.0;
            let rank_x = base_x;
            let name_x = base_x + 50.0;
            let score_x = base_x + 220.0;
            let level_x = base_x + 320.0;
            let lines_x = base_x + 380.0;
            let time_x = base_x + 450.0;
            
            // Draw column headers
            self.draw_text_with_outline("RANK", rank_x, header_y, 18.0, Color::new(0.6, 0.8, 1.0, 1.0));
            self.draw_text_with_outline("PLAYER NAME", name_x, header_y, 18.0, Color::new(0.6, 0.8, 1.0, 1.0));
            self.draw_text_with_outline("SCORE", score_x, header_y, 18.0, Color::new(0.6, 0.8, 1.0, 1.0));
            self.draw_text_with_outline("LVL", level_x, header_y, 18.0, Color::new(0.6, 0.8, 1.0, 1.0));
            self.draw_text_with_outline("LINES", lines_x, header_y, 18.0, Color::new(0.6, 0.8, 1.0, 1.0));
            self.draw_text_with_outline("TIME", time_x, header_y, 18.0, Color::new(0.6, 0.8, 1.0, 1.0));
            
            // Draw entries (with scrolling)
            let visible_entries = 7;
            let start_idx = self.leaderboard_scroll;
            let end_idx = (start_idx + visible_entries).min(self.leaderboard.entries.len());
            
            for (display_idx, entry_idx) in (start_idx..end_idx).enumerate() {
                let entry = &self.leaderboard.entries[entry_idx];
                let rank = entry_idx + 1;
                
                let entry_y = entry_y_start + (display_idx as f32 * entry_spacing);
                
                // Color based on rank
                let color = match rank {
                    1 => Color::new(1.0, 0.85, 0.0, 1.0), // Gold
                    2 => Color::new(0.75, 0.75, 0.75, 1.0), // Silver
                    3 => Color::new(0.8, 0.5, 0.2, 1.0), // Bronze
                    _ => Color::new(0.8, 0.8, 0.8, 0.9), // White
                };
                
                // Draw each column individually for perfect alignment
                self.draw_text_with_outline(&rank.to_string(), rank_x, entry_y, entry_size, color);
                self.draw_text_with_outline(&entry.name, name_x, entry_y, entry_size, color);
                self.draw_text_with_outline(&entry.score.to_string(), score_x, entry_y, entry_size, color);
                self.draw_text_with_outline(&entry.level.to_string(), level_x, entry_y, entry_size, color);
                self.draw_text_with_outline(&entry.lines_cleared.to_string(), lines_x, entry_y, entry_size, color);
                self.draw_text_with_outline(&entry.formatted_time(), time_x, entry_y, entry_size, color);
            }
            
            // Draw scroll indicators if needed
            if self.leaderboard_scroll > 0 {
                let up_arrow = "- more above -";
                self.draw_text_with_outline(up_arrow, base_x, entry_y_start - 50.0, 16.0, Color::new(0.8, 0.8, 0.8, 0.7));
            }
            
            if end_idx < self.leaderboard.entries.len() {
                let down_arrow = "- more below -";
                self.draw_text_with_outline(down_arrow, base_x, entry_y_start + (visible_entries as f32 * entry_spacing) + 20.0, 16.0, Color::new(0.8, 0.8, 0.8, 0.7));
            }
        }
        
        // Draw instructions
        let instruction = if touch { "Tap anywhere to go back" } else { "Press ESCAPE or ENTER to return to main menu" };
        let inst_width = measure_text(instruction, None, 20, 1.0).width;
        let inst_x = (WINDOW_WIDTH as f32 - inst_width) / 2.0;
        let inst_y = if touch { self.touch_leaderboard_bottom() - 25.0 } else { WINDOW_HEIGHT as f32 - 50.0 };
        
        self.draw_text_with_outline(instruction, inst_x, inst_y, 20.0, Color::new(0.7, 0.7, 0.7, 0.8));
    }
    
    /// Render the settings screen
    fn render_settings(&self, background_texture: &Texture2D) {
        let touch = crate::platform::is_touch_device();
        self.draw_background(background_texture, 0.6);
        
        // Draw title
        let title = "SETTINGS";
        let title_size = 48.0;
        let title_width = measure_text(title, None, title_size as u16, 1.0).width;
        let title_x = (WINDOW_WIDTH as f32 - title_width) / 2.0;
        let title_y = 150.0;
        
        self.draw_text_with_outline(title, title_x, title_y, title_size, Color::new(0.8, 0.4, 1.0, 1.0));

        if touch {
            self.draw_touch_settings();
            return;
        }
        
        // Draw settings options
        let option_size = 32.0;
        let option_y_start = 280.0;
        let option_spacing = 80.0;
        
        // Sound setting
        let sound_text = format!("SOUND: {}", if self.settings.sound_enabled { "ON" } else { "OFF" });
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
        let volume_text = format!("VOLUME: {:.0}%", self.settings.volume * 100.0);
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
        
        // Draw volume bar (always on touch screens, where nothing is "selected")
        if volume_selected || touch {
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
            let instruction = if touch { "Tap sound or volume to change them" } else { "Use LEFT/RIGHT arrows to adjust volume" };
            let inst_width = measure_text(instruction, None, 18, 1.0).width;
            let inst_x = (WINDOW_WIDTH as f32 - inst_width) / 2.0;
            let inst_y = bar_y + 40.0;
            
            self.draw_text_with_outline(instruction, inst_x, inst_y, 18.0, Color::new(0.7, 0.7, 0.7, 0.8));
        }
        
        // Legacy look
        let legacy_text = format!("LEGACY: {}", if self.settings.legacy_mode { "ON" } else { "OFF" });
        let legacy_w = measure_text(&legacy_text, None, option_size as u16, 1.0).width;
        let legacy_x = (WINDOW_WIDTH as f32 - legacy_w) / 2.0;
        let legacy_y = option_y_start + option_spacing * 2.75;
        let legacy_selected = self.selected_option == 2;
        if legacy_selected {
            let pulse = (self.animation_timer * 3.0).sin() * 0.3 + 0.7;
            draw_rectangle(legacy_x - 20.0, legacy_y - option_size - 5.0, legacy_w + 40.0, option_size + 10.0, Color::new(0.2, 0.4, 1.0, 0.3 * pulse as f32));
        }
        let legacy_color = if legacy_selected {
            let pulse = (self.animation_timer * 4.0).sin() * 0.2 + 0.8;
            Color::new(1.0, 1.0, 0.8, pulse as f32)
        } else {
            Color::new(0.75, 0.6, 1.0, 0.9)
        };
        self.draw_text_with_outline(&legacy_text, legacy_x, legacy_y, option_size, legacy_color);

        // Draw general instructions
        let instruction = if touch { "Tap anywhere else to go back" } else { "Press ESCAPE to return to main menu" };
        let inst_width = measure_text(instruction, None, 20, 1.0).width;
        let inst_x = (WINDOW_WIDTH as f32 - inst_width) / 2.0;
        // Touch: right under the options (the screen is fitted around them; see content_bounds).
        let inst_y = if touch { 480.0 } else { WINDOW_HEIGHT as f32 - 50.0 };
        
        self.draw_text_with_outline(instruction, inst_x, inst_y, 20.0, Color::new(0.7, 0.7, 0.7, 0.8));
    }
    
    /// Render the name entry screen
    fn render_name_entry(&self, background_texture: &Texture2D, score: u32, level: u32, lines_cleared: u32, game_time: f64) {
        let touch = crate::platform::is_touch_device();
        self.draw_background(background_texture, 0.7);
        
        // Draw congratulations title
        let title = "NEW HIGH SCORE!";
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
            "Score: {}   Level: {}   Lines: {}   Time: {}:{:02}",
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
        let instruction = if touch { "Type your name, then tap Return" } else { "Press ENTER to confirm - ESCAPE to cancel" };
        let inst_y = if touch { 500.0 } else { WINDOW_HEIGHT as f32 - 80.0 };
        let inst_size = if touch { 24.0 } else { 20.0 };
        let inst_x = (WINDOW_WIDTH as f32 - measure_text(instruction, None, inst_size as u16, 1.0).width) / 2.0;

        self.draw_text_with_outline(instruction, inst_x, inst_y, inst_size, Color::new(0.7, 0.7, 0.7, 0.8));
        
        // Show predicted rank
        if let Some(rank) = self.leaderboard.get_rank_for_score(score) {
            let rank_text = format!("This will be rank #{} on the leaderboard!", rank);
            let rank_width = measure_text(&rank_text, None, 22, 1.0).width;
            let rank_x = (WINDOW_WIDTH as f32 - rank_width) / 2.0;
            let rank_y = 450.0;
            
            self.draw_text_with_outline(&rank_text, rank_x, rank_y, 22.0, Color::new(1.0, 0.9, 0.3, 1.0));
        }
    }
    
    /// The screen's background: the game's backdrop, dimmed by `dim`. On touch screens it covers
    /// the whole screen (graphics::view::draw_cover_background); on desktop, the canvas.
    fn draw_background(&self, background_texture: &Texture2D, dim: f32) {
        if crate::platform::is_touch_device() {
            let _ = (background_texture, dim);
            let ppu = crate::graphics::text::pixels_per_unit();
            push_camera_state();
            set_default_camera();
            crate::graphics::style::backdrop(self.animation_timer as f32);
            pop_camera_state();
            crate::graphics::text::set_pixels_per_unit(ppu);
            return;
        }
        clear_background(Color::new(0.02, 0.02, 0.08, 1.0));
        draw_texture(background_texture, 0.0, 0.0, WHITE);
        draw_rectangle(0.0, 0.0, WINDOW_WIDTH as f32, WINDOW_HEIGHT as f32, Color::new(0.0, 0.0, 0.0, dim));
    }

    /// The part of the 900x750 canvas the current screen draws in, in canvas units. On touch
    /// screens just this is fitted to the screen, so menus come out as big as they can.
    pub fn content_bounds(&self) -> Rect {
        match self.state {
            MenuState::Main => {
                // The title (from y 70) down to just under the last option (see render_main_menu).
                let last = 320.0 + (self.get_main_menu_options().len() as f32 - 1.0) * 55.0;
                Rect::new(230.0, 70.0, 440.0, last + 45.0 - 70.0)
            }
            // Touch settings are compact: the "go back" line sits under the haptics switch
            // (y 580), the display info under that.
            MenuState::Settings => Rect::new(250.0, 95.0, 400.0, 555.0),
            MenuState::Leaderboard => Rect::new(
                TOUCH_TABLE_X - 20.0,
                50.0,
                TOUCH_TABLE_W + 40.0,
                self.touch_leaderboard_bottom() - 50.0,
            ),
            // Everything sits in the top part: the keyboard covers the bottom half.
            MenuState::NameEntry { .. } => Rect::new(140.0, 60.0, 620.0, 800.0),
        }
    }

    /// Where the touch leaderboard ends (just under its "go back" line), in canvas units.
    fn touch_leaderboard_bottom(&self) -> f32 {
        let rows = self.leaderboard.entries.len().min(TOUCH_TABLE_ROWS);
        if rows == 0 { 460.0 } else { 180.0 + 22.0 + (rows as f32 - 1.0) * 45.0 + 85.0 }
    }

    /// Draw animated title for main menu
    fn draw_animated_title(&self) {
        let title = "RUST TETRIS";
        let title_size = 72.0;
        
        // Calculate the actual rendered width by summing individual character widths
        let mut total_width = 0.0;
        let mut char_positions = Vec::new();
        let mut current_x = 0.0;
        
        for c in title.chars() {
            if c == ' ' {
                // For spaces, add spacing but don't render
                let space_width = title_size * 0.4; // Adjust space width as needed
                char_positions.push((c, current_x));
                current_x += space_width;
                total_width += space_width;
            } else {
                // For regular characters
                let char_width = measure_text(&c.to_string(), None, title_size as u16, 1.0).width;
                char_positions.push((c, current_x));
                current_x += char_width;
                total_width += char_width;
            }
        }
        
        // Center the entire title
        let title_start_x = (WINDOW_WIDTH as f32 - total_width) / 2.0;
        let title_y = 150.0;
        
        // Draw each letter with a wave effect and rainbow colors
        let time_offset = self.animation_timer * 2.0;
        let mut render_index = 0; // Only count rendered characters for color calculation
        
        for (i, (c, relative_x)) in char_positions.iter().enumerate() {
            if *c == ' ' {
                continue; // Skip rendering spaces
            }
            
            let char_x = title_start_x + relative_x;
            let wave_offset = (self.animation_timer * 2.0 + render_index as f64 * 0.5).sin() * 10.0;
            let char_y = title_y + wave_offset as f32;
            
            // Rainbow color based on render position
            let hue = ((render_index as f64 * 0.3) + time_offset) % 6.0;
            let rainbow_color = self.hsv_to_rgb(hue, 0.9, 1.0);
            
            // Draw with outline
            self.draw_text_with_outline(&c.to_string(), char_x, char_y, title_size, rainbow_color);
            
            render_index += 1;
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
#[cfg(test)]
mod settings_tests {
    use super::GameSettings;

    #[test]
    fn old_settings_have_haptics_on() {
        let old = r#"{"sound_enabled":true,"volume":0.5,"gesture_hint_seen":true,"legacy_mode":false}"#;
        let s: GameSettings = serde_json::from_str(old).unwrap();
        assert!(s.haptics_enabled);
        assert_eq!(s.volume, 0.5);
    }
}
