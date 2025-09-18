//! Leaderboard system for tracking high scores

use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;
use chrono::{DateTime, Local};

/// Maximum number of high score entries to keep
pub const MAX_LEADERBOARD_ENTRIES: usize = 10;

/// A single high score entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    /// Player name
    pub name: String,
    /// Final score
    pub score: u32,
    /// Level reached
    pub level: u32,
    /// Total lines cleared
    pub lines_cleared: u32,
    /// Game duration in seconds
    pub game_time: f64,
    /// When this score was achieved
    pub timestamp: DateTime<Local>,
}

impl LeaderboardEntry {
    /// Create a new leaderboard entry
    pub fn new(name: String, score: u32, level: u32, lines_cleared: u32, game_time: f64) -> Self {
        Self {
            name,
            score,
            level,
            lines_cleared,
            game_time,
            timestamp: Local::now(),
        }
    }
    
    /// Format the game time as minutes:seconds
    pub fn formatted_time(&self) -> String {
        let minutes = (self.game_time / 60.0) as u32;
        let seconds = (self.game_time % 60.0) as u32;
        format!("{}:{:02}", minutes, seconds)
    }
}

/// The leaderboard containing all high score entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leaderboard {
    /// List of high score entries, sorted by score (highest first)
    pub entries: Vec<LeaderboardEntry>,
}

impl Leaderboard {
    /// Create a new empty leaderboard
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
    
    /// Check if a score qualifies for the leaderboard
    pub fn qualifies_for_leaderboard(&self, score: u32) -> bool {
        // Always qualifies if we have room
        if self.entries.len() < MAX_LEADERBOARD_ENTRIES {
            return true;
        }
        
        // Qualifies if score is higher than the lowest entry
        self.entries.last().map_or(true, |lowest| score > lowest.score)
    }
    
    /// Add a new entry to the leaderboard
    /// Returns the position (1-based) of the new entry, or None if it didn't make the board
    pub fn add_entry(&mut self, entry: LeaderboardEntry) -> Option<usize> {
        // Find the correct position to insert this entry
        let position = self.entries
            .iter()
            .position(|existing| entry.score > existing.score)
            .unwrap_or(self.entries.len());
        
        // Insert at the correct position
        self.entries.insert(position, entry);
        
        // Trim to maximum entries
        if self.entries.len() > MAX_LEADERBOARD_ENTRIES {
            self.entries.truncate(MAX_LEADERBOARD_ENTRIES);
        }
        
        // Return the position if it made it into the leaderboard
        if position < MAX_LEADERBOARD_ENTRIES {
            Some(position + 1) // Convert to 1-based indexing
        } else {
            None
        }
    }
    
    /// Get the rank for a given score (what position it would be at)
    pub fn get_rank_for_score(&self, score: u32) -> Option<usize> {
        if !self.qualifies_for_leaderboard(score) {
            return None;
        }
        
        let position = self.entries
            .iter()
            .position(|existing| score > existing.score)
            .unwrap_or(self.entries.len());
            
        Some(position + 1) // Convert to 1-based indexing
    }
    
    /// Get the default leaderboard file path
    pub fn default_path() -> std::path::PathBuf {
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join("tetris_leaderboard.json")
    }
    
    /// Save leaderboard to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        log::info!("Leaderboard saved successfully");
        Ok(())
    }
    
    /// Load leaderboard from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let json = fs::read_to_string(path)?;
        let leaderboard: Leaderboard = serde_json::from_str(&json)?;
        log::info!("Leaderboard loaded successfully");
        Ok(leaderboard)
    }
    
    /// Load leaderboard from file, or create new one if file doesn't exist
    pub fn load_or_create<P: AsRef<Path>>(path: P) -> Self {
        match Self::load_from_file(&path) {
            Ok(leaderboard) => leaderboard,
            Err(e) => {
                log::info!("Could not load leaderboard ({}), creating new one", e);
                Self::new()
            }
        }
    }
    
    /// Check if leaderboard file exists
    pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }
}

impl Default for Leaderboard {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_leaderboard_qualifies_any_score() {
        let leaderboard = Leaderboard::new();
        assert!(leaderboard.qualifies_for_leaderboard(100));
        assert!(leaderboard.qualifies_for_leaderboard(0));
    }
    
    #[test]
    fn test_add_entry_returns_correct_position() {
        let mut leaderboard = Leaderboard::new();
        
        let entry1 = LeaderboardEntry::new("Player1".to_string(), 1000, 5, 25, 300.0);
        let entry2 = LeaderboardEntry::new("Player2".to_string(), 1500, 7, 40, 450.0);
        let entry3 = LeaderboardEntry::new("Player3".to_string(), 800, 3, 15, 200.0);
        
        assert_eq!(leaderboard.add_entry(entry1), Some(1));
        assert_eq!(leaderboard.add_entry(entry2), Some(1)); // Should be first place
        assert_eq!(leaderboard.add_entry(entry3), Some(3)); // Should be third place
        
        // Check order is correct
        assert_eq!(leaderboard.entries[0].score, 1500);
        assert_eq!(leaderboard.entries[1].score, 1000);
        assert_eq!(leaderboard.entries[2].score, 800);
    }
    
    #[test]
    fn test_leaderboard_max_entries() {
        let mut leaderboard = Leaderboard::new();
        
        // Fill beyond max entries
        for i in 0..15 {
            let entry = LeaderboardEntry::new(format!("Player{}", i), i * 100, 1, 1, 60.0);
            leaderboard.add_entry(entry);
        }
        
        // Should only keep max entries
        assert_eq!(leaderboard.entries.len(), MAX_LEADERBOARD_ENTRIES);
        
        // Highest scores should be kept
        assert_eq!(leaderboard.entries[0].score, 1400);
        assert_eq!(leaderboard.entries[MAX_LEADERBOARD_ENTRIES - 1].score, 500);
    }
    
    #[test]
    fn test_qualify_for_full_leaderboard() {
        let mut leaderboard = Leaderboard::new();
        
        // Fill leaderboard with scores 100, 200, ..., 1000
        for i in 1..=MAX_LEADERBOARD_ENTRIES {
            let entry = LeaderboardEntry::new(format!("Player{}", i), i as u32 * 100, 1, 1, 60.0);
            leaderboard.add_entry(entry);
        }
        
        // Score of 150 should not qualify (lower than lowest entry of 100)
        assert!(!leaderboard.qualifies_for_leaderboard(50));
        
        // Score of 150 should qualify (higher than lowest entry of 100)
        assert!(leaderboard.qualifies_for_leaderboard(150));
        
        // Score of 1500 should definitely qualify
        assert!(leaderboard.qualifies_for_leaderboard(1500));
    }
}