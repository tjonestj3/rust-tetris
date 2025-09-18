//! Audio system for managing game sounds

use macroquad::audio::{Sound, load_sound, play_sound, PlaySoundParams, stop_sound, set_sound_volume};
use std::collections::HashMap;

/// Types of sounds in the game
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoundType {
    /// UI interactions (piece movement, menu navigation)
    UiClick,
    /// Piece placement/locking sound
    PieceSnap,
    /// Hard drop impact
    HardDrop,
    /// Hold piece action
    HoldPiece,
    /// Line clearing effect
    LineClear,
    /// Level completion
    LevelComplete,
    /// Pause/unpause
    Pause,
    /// Game over
    GameOver,
    /// Power-up/special action (for future power-up system)
    PowerAction,
    /// Background music
    BackgroundMusic,
}

/// Audio system managing all game sounds
#[derive(Debug)]
pub struct AudioSystem {
    /// Loaded sound effects
    sounds: HashMap<SoundType, Sound>,
    /// Master volume (0.0 to 1.0)
    master_volume: f32,
    /// Sound effects volume (0.0 to 1.0)
    sfx_volume: f32,
    /// Music volume (0.0 to 1.0)
    music_volume: f32,
    /// Whether audio is enabled
    audio_enabled: bool,
    /// Whether background music is currently playing
    background_music_playing: bool,
}

impl AudioSystem {
    /// Create a new audio system
    pub fn new() -> Self {
        Self {
            sounds: HashMap::new(),
            master_volume: 1.0,
            sfx_volume: 0.7,
            music_volume: 0.5,
            audio_enabled: true,
            background_music_playing: false,
        }
    }
    
    /// Load all game sounds asynchronously
    pub async fn load_sounds(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Loading game audio assets...");
        
        // Sound file mappings
        let sound_files = [
            (SoundType::UiClick, "assets/sounds/ui-click.wav"),
            (SoundType::PieceSnap, "assets/sounds/piece-snap.wav"),
            (SoundType::HardDrop, "assets/sounds/hard-drop.wav"),
            (SoundType::HoldPiece, "assets/sounds/hold-piece.wav"),
            (SoundType::LineClear, "assets/sounds/line-clear.wav"),
            (SoundType::LevelComplete, "assets/sounds/level-complete.wav"),
            (SoundType::Pause, "assets/sounds/pause.wav"),
            (SoundType::GameOver, "assets/sounds/game-over.wav"),
            (SoundType::PowerAction, "assets/sounds/place-ghost-block.wav"),
            (SoundType::BackgroundMusic, "assets/sounds/tetris-background-music.wav"),
        ];
        
        for (sound_type, file_path) in sound_files {
            match load_sound(file_path).await {
                Ok(sound) => {
                    self.sounds.insert(sound_type, sound);
                    log::debug!("Loaded sound: {:?} from {}", sound_type, file_path);
                }
                Err(e) => {
                    log::warn!("Failed to load sound {:?} from {}: {} - continuing without this sound", sound_type, file_path, e);
                    // Continue loading other sounds even if one fails
                }
            }
        }
        
        log::info!("Audio system initialized with {} sounds loaded", self.sounds.len());
        Ok(())
    }
    
    /// Play a sound effect
    pub fn play_sound(&self, sound_type: SoundType) {
        if !self.audio_enabled {
            return;
        }
        
        if let Some(sound) = self.sounds.get(&sound_type) {
            let volume = match sound_type {
                SoundType::BackgroundMusic => self.master_volume * self.music_volume,
                _ => self.master_volume * self.sfx_volume,
            };
            
            let params = PlaySoundParams {
                looped: sound_type == SoundType::BackgroundMusic,
                volume,
            };
            
            play_sound(sound, params);
            log::info!("Playing sound: {:?} at volume {:.2}", sound_type, volume);
        } else {
            log::warn!("Sound not loaded: {:?}", sound_type);
        }
    }
    
    /// Play a sound effect with custom volume
    pub fn play_sound_with_volume(&self, sound_type: SoundType, volume_multiplier: f32) {
        if !self.audio_enabled {
            return;
        }
        
        if let Some(sound) = self.sounds.get(&sound_type) {
            let base_volume = match sound_type {
                SoundType::BackgroundMusic => self.master_volume * self.music_volume,
                _ => self.master_volume * self.sfx_volume,
            };
            
            let final_volume = base_volume * volume_multiplier.clamp(0.0, 1.0);
            
            let params = PlaySoundParams {
                looped: sound_type == SoundType::BackgroundMusic,
                volume: final_volume,
            };
            
            play_sound(sound, params);
            log::info!("Playing sound: {:?} at volume {:.2} ({}x multiplier)", 
                       sound_type, final_volume, volume_multiplier);
        } else {
            log::warn!("Sound not loaded: {:?}", sound_type);
        }
    }
    
    /// Set master volume (0.0 to 1.0)
    pub fn set_master_volume(&mut self, volume: f32) {
        let new_volume = volume.clamp(0.0, 1.0);
        if self.master_volume != new_volume {
            self.master_volume = new_volume;
            log::debug!("Master volume set to {:.2}", self.master_volume);
            // Update background music volume smoothly without restarting
            self.update_background_music_volume();
        }
    }
    
    /// Set sound effects volume (0.0 to 1.0)
    pub fn set_sfx_volume(&mut self, volume: f32) {
        self.sfx_volume = volume.clamp(0.0, 1.0);
        log::debug!("SFX volume set to {:.2}", self.sfx_volume);
    }
    
    /// Set music volume (0.0 to 1.0)
    pub fn set_music_volume(&mut self, volume: f32) {
        let new_volume = volume.clamp(0.0, 1.0);
        if self.music_volume != new_volume {
            self.music_volume = new_volume;
            log::debug!("Music volume set to {:.2}", self.music_volume);
            // Update background music volume smoothly without restarting
            self.update_background_music_volume();
        }
    }
    
    /// Enable or disable audio
    pub fn set_audio_enabled(&mut self, enabled: bool) {
        if self.audio_enabled != enabled {
            self.audio_enabled = enabled;
            log::info!("Audio {}", if enabled { "enabled" } else { "disabled" });
            
            if enabled {
                // When enabled, start background music at current volume
                self.start_background_music();
            } else {
                // When disabled, stop background music
                self.stop_background_music();
            }
        }
    }
    
    /// Get master volume
    pub fn master_volume(&self) -> f32 {
        self.master_volume
    }
    
    /// Get SFX volume
    pub fn sfx_volume(&self) -> f32 {
        self.sfx_volume
    }
    
    /// Get music volume
    pub fn music_volume(&self) -> f32 {
        self.music_volume
    }
    
    /// Check if audio is enabled
    pub fn is_audio_enabled(&self) -> bool {
        self.audio_enabled
    }
    
    /// Start background music
    pub fn start_background_music(&mut self) {
        if !self.background_music_playing {
            log::info!("Starting background music");
            self.play_sound(SoundType::BackgroundMusic);
            self.background_music_playing = true;
        }
    }
    
    /// Stop background music
    pub fn stop_background_music(&mut self) {
        if self.background_music_playing {
            log::info!("Stopping background music");
            if let Some(sound) = self.sounds.get(&SoundType::BackgroundMusic) {
                stop_sound(sound);
            }
            self.background_music_playing = false;
        }
    }
    
    /// Check if background music is playing
    pub fn is_background_music_playing(&self) -> bool {
        self.background_music_playing
    }
    
    /// Update background music volume without restarting
    pub fn update_background_music_volume(&self) {
        if self.background_music_playing && self.audio_enabled {
            if let Some(sound) = self.sounds.get(&SoundType::BackgroundMusic) {
                let volume = self.master_volume * self.music_volume;
                set_sound_volume(sound, volume);
                log::debug!("Updated background music volume to {:.2}", volume);
            }
        }
    }
    
    /// Restart background music with current volume settings if it was playing
    fn restart_background_music_if_playing(&mut self) {
        if self.background_music_playing {
            log::info!("Restarting background music with updated volume");
            // Stop the old sound first
            self.stop_background_music();
            // Start with new settings
            self.start_background_music();
        }
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new()
    }
}