//! Modern Tetris scoring system implementation
//!
//! This module implements the official Tetris Guideline scoring system including:
//! - Standard line clear scoring
//! - T-spin bonuses (Mini and Full)
//! - Combo system for consecutive line clears
//! - Back-to-back bonuses for skill moves
//! - Perfect Clear (All Clear) detection and bonuses
//! - Soft/Hard drop scoring

use serde::{Serialize, Deserialize};

pub mod perfect_clear;

pub use perfect_clear::PerfectClearDetector;

/// Types of line clear actions that can occur
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineClearType {
    /// Single line clear
    Single,
    /// Double line clear  
    Double,
    /// Triple line clear
    Triple,
    /// Tetris (4 lines)
    Tetris,
    /// T-spin Mini Single (1 line with mini T-spin)
    TSpinMiniSingle,
    /// T-spin Single (1 line with full T-spin)
    TSpinSingle,
    /// T-spin Mini Double (2 lines with mini T-spin - very rare)
    TSpinMiniDouble,
    /// T-spin Double (2 lines with full T-spin)
    TSpinDouble,
    /// T-spin Triple (3 lines with full T-spin)
    TSpinTriple,
}

impl LineClearType {
    /// Get the base score for this line clear type (before level multiplier)
    pub fn base_score(self) -> u32 {
        match self {
            LineClearType::Single => 100,
            LineClearType::Double => 300,
            LineClearType::Triple => 500,
            LineClearType::Tetris => 800,
            LineClearType::TSpinMiniSingle => 200,
            LineClearType::TSpinSingle => 800,
            LineClearType::TSpinMiniDouble => 400,
            LineClearType::TSpinDouble => 1200,
            LineClearType::TSpinTriple => 1600,
        }
    }
    
    /// Get the number of lines cleared by this action
    pub fn lines_cleared(self) -> u32 {
        match self {
            LineClearType::Single | 
            LineClearType::TSpinMiniSingle | 
            LineClearType::TSpinSingle => 1,
            LineClearType::Double | 
            LineClearType::TSpinMiniDouble | 
            LineClearType::TSpinDouble => 2,
            LineClearType::Triple | 
            LineClearType::TSpinTriple => 3,
            LineClearType::Tetris => 4,
        }
    }
    
    /// Check if this is a "difficult" move that qualifies for back-to-back bonus
    pub fn is_difficult(self) -> bool {
        matches!(self, 
            LineClearType::Tetris |
            LineClearType::TSpinSingle |
            LineClearType::TSpinDouble |
            LineClearType::TSpinTriple
        )
    }
    
    /// Check if this is a T-spin move
    pub fn is_t_spin(self) -> bool {
        matches!(self,
            LineClearType::TSpinMiniSingle |
            LineClearType::TSpinSingle |
            LineClearType::TSpinMiniDouble |
            LineClearType::TSpinDouble |
            LineClearType::TSpinTriple
        )
    }
    
    /// Get a human-readable name for this line clear type
    pub fn name(self) -> &'static str {
        match self {
            LineClearType::Single => "Single",
            LineClearType::Double => "Double", 
            LineClearType::Triple => "Triple",
            LineClearType::Tetris => "Tetris",
            LineClearType::TSpinMiniSingle => "T-Spin Mini Single",
            LineClearType::TSpinSingle => "T-Spin Single",
            LineClearType::TSpinMiniDouble => "T-Spin Mini Double",
            LineClearType::TSpinDouble => "T-Spin Double",
            LineClearType::TSpinTriple => "T-Spin Triple",
        }
    }
}

/// Perfect Clear (All Clear) bonus types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PerfectClearType {
    Single,   // 800 × level
    Double,   // 1200 × level
    Triple,   // 1800 × level
    Tetris,   // 2000 × level
}

impl PerfectClearType {
    /// Get the base bonus score for this perfect clear type
    pub fn base_bonus(self) -> u32 {
        match self {
            PerfectClearType::Single => 800,
            PerfectClearType::Double => 1200,
            PerfectClearType::Triple => 1800,
            PerfectClearType::Tetris => 2000,
        }
    }
}

/// Scoring action that occurred during gameplay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringAction {
    /// Type of line clear
    pub line_clear_type: LineClearType,
    /// Whether this was a perfect clear (all blocks cleared)
    pub perfect_clear: Option<PerfectClearType>,
    /// Current level when action occurred
    pub level: u32,
    /// Current combo count (0 = no combo)
    pub combo: u32,
    /// Whether back-to-back bonus applies
    pub back_to_back: bool,
}

/// Complete scoring result with breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringResult {
    /// Base score from line clear
    pub base_score: u32,
    /// Bonus from combo system
    pub combo_bonus: u32,
    /// Bonus from back-to-back chain
    pub back_to_back_bonus: u32,
    /// Bonus from perfect clear
    pub perfect_clear_bonus: u32,
    /// Total score awarded
    pub total_score: u32,
    /// Updated combo count
    pub new_combo: u32,
    /// Whether back-to-back chain continues
    pub back_to_back_continues: bool,
}

/// Comprehensive Tetris scoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TetrisScoring {
    /// Current combo count (consecutive non-zero line clears)
    pub combo_count: u32,
    /// Whether the last difficult move enables back-to-back bonus
    pub back_to_back_ready: bool,
    /// Total score accumulated
    pub total_score: u32,
    /// Line clears processed so far (so the app can tell a new one happened).
    #[serde(default)]
    pub clears: u32,
    /// Pieces locked so far (counted by the game at each lock).
    #[serde(default)]
    pub pieces: u32,
    /// The most recent clear.
    #[serde(default)]
    pub last_clear: Option<ClearRecord>,
    /// This game's tallies.
    #[serde(default)]
    pub stats: RunStats,
}

/// A game's tallies, shown beside the board on the open iPhone Duo.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RunStats {
    pub t_spins: u32,
    pub tetrises: u32,
    pub perfect_clears: u32,
    /// The longest combo (as `ClearRecord::chain`).
    pub best_combo: u32,
    /// Difficult clears (Tetrises, T-spins) in a row, now and at most.
    pub b2b_streak: u32,
    pub b2b_best: u32,
}

impl RunStats {
    fn add(&mut self, clear: &ClearRecord) {
        if clear.kind.is_t_spin() {
            self.t_spins += 1;
        }
        if clear.kind == LineClearType::Tetris {
            self.tetrises += 1;
        }
        if clear.perfect {
            self.perfect_clears += 1;
        }
        self.best_combo = self.best_combo.max(clear.chain);
        self.b2b_streak = if clear.kind.is_difficult() { self.b2b_streak + 1 } else { 0 };
        self.b2b_best = self.b2b_best.max(self.b2b_streak);
    }
}

/// What a line clear was, for haptics, stats and callouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClearRecord {
    pub kind: LineClearType,
    /// Clears in a row before this one: 0 for the first, 1 for "combo 1", and so on.
    pub chain: u32,
    /// A difficult clear right after another one (back-to-back bonus).
    pub back_to_back: bool,
    /// The board was left empty.
    pub perfect: bool,
}

impl Default for TetrisScoring {
    fn default() -> Self {
        Self::new()
    }
}

impl TetrisScoring {
    /// Create a new scoring system
    pub fn new() -> Self {
        Self {
            combo_count: 0,
            back_to_back_ready: false,
            total_score: 0,
            clears: 0,
            pieces: 0,
            last_clear: None,
            stats: RunStats::default(),
        }
    }
    
    /// Calculate score for a line clear action
    pub fn calculate_score(&self, action: ScoringAction) -> ScoringResult {
        let base_score = action.line_clear_type.base_score() * action.level;
        
        // Calculate combo bonus
        let combo_bonus = if self.combo_count > 0 {
            50 * self.combo_count * action.level
        } else {
            0
        };
        
        // Calculate back-to-back bonus
        let back_to_back_bonus = if action.back_to_back && self.back_to_back_ready {
            // 50% bonus for back-to-back difficult moves
            base_score / 2
        } else {
            0
        };
        
        // Calculate perfect clear bonus
        let perfect_clear_bonus = if let Some(pc_type) = action.perfect_clear {
            pc_type.base_bonus() * action.level
        } else {
            0
        };
        
        let total_score = base_score + combo_bonus + back_to_back_bonus + perfect_clear_bonus;
        
        // Update combo count
        let new_combo = self.combo_count + 1;
        
        // Update back-to-back status
        let back_to_back_continues = action.line_clear_type.is_difficult();
        
        ScoringResult {
            base_score,
            combo_bonus,
            back_to_back_bonus,
            perfect_clear_bonus,
            total_score,
            new_combo,
            back_to_back_continues,
        }
    }
    
    /// Process a line clear and update internal state
    pub fn process_line_clear(&mut self, action: ScoringAction) -> ScoringResult {
        let result = self.calculate_score(action.clone());
        self.clears += 1;
        let record = ClearRecord {
            kind: action.line_clear_type,
            chain: self.combo_count,
            back_to_back: result.back_to_back_bonus > 0,
            perfect: action.perfect_clear.is_some(),
        };
        self.stats.add(&record);
        self.last_clear = Some(record);
        
        // Update internal state
        self.combo_count = result.new_combo;
        self.back_to_back_ready = result.back_to_back_continues;
        self.total_score += result.total_score;
        
        result
    }
    
    /// Process a piece placement with no line clear (breaks combo)
    pub fn process_no_line_clear(&mut self) {
        self.combo_count = 0;
        // back_to_back_ready status is preserved (only reset by non-difficult moves)
    }
    
    /// Reset combo but preserve back-to-back status
    pub fn break_combo(&mut self) {
        self.combo_count = 0;
    }
    
    /// Reset back-to-back status (called when non-difficult move is made)
    pub fn break_back_to_back(&mut self) {
        self.back_to_back_ready = false;
    }
    
    /// Get current combo count
    pub fn current_combo(&self) -> u32 {
        self.combo_count
    }
    
    /// Check if back-to-back bonus is ready
    pub fn is_back_to_back_ready(&self) -> bool {
        self.back_to_back_ready
    }
    
    /// Get total accumulated score
    pub fn total_score(&self) -> u32 {
        self.total_score
    }
    
    /// Add points for soft/hard drop (separate from line clears)
    pub fn add_drop_points(&mut self, points: u32) {
        self.total_score += points;
    }
    
    /// Line clears processed so far.
    pub fn clears(&self) -> u32 {
        self.clears
    }

    /// A piece locked.
    pub fn count_piece(&mut self) {
        self.pieces += 1;
    }

    /// Pieces locked so far.
    pub fn pieces(&self) -> u32 {
        self.pieces
    }

    /// The most recent clear.
    pub fn last_clear(&self) -> Option<ClearRecord> {
        self.last_clear
    }

    /// This game's tallies.
    pub fn stats(&self) -> RunStats {
        self.stats
    }

    /// Reset the scoring system (for new game)
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

/// Helper function to determine line clear type from basic parameters
pub fn determine_line_clear_type(
    lines_cleared: u32,
    is_t_spin: bool,
    is_mini_t_spin: bool,
) -> Option<LineClearType> {
    match (lines_cleared, is_t_spin, is_mini_t_spin) {
        (0, _, _) => None,
        (1, false, _) => Some(LineClearType::Single),
        (2, false, _) => Some(LineClearType::Double),
        (3, false, _) => Some(LineClearType::Triple),
        (4, false, _) => Some(LineClearType::Tetris),
        (1, true, true) => Some(LineClearType::TSpinMiniSingle),
        (1, true, false) => Some(LineClearType::TSpinSingle),
        (2, true, true) => Some(LineClearType::TSpinMiniDouble),
        (2, true, false) => Some(LineClearType::TSpinDouble),
        (3, true, false) => Some(LineClearType::TSpinTriple),
        (3, true, true) => Some(LineClearType::TSpinTriple), // Mini triple = regular triple
        _ => None, // Invalid combinations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_line_clear_scoring() {
        let mut scoring = TetrisScoring::new();
        let level = 5;
        
        // Test single line clear
        let action = ScoringAction {
            line_clear_type: LineClearType::Single,
            perfect_clear: None,
            level,
            combo: 0,
            back_to_back: false,
        };
        
        let result = scoring.process_line_clear(action);
        assert_eq!(result.base_score, 100 * level); // 500 points
        assert_eq!(result.combo_bonus, 0); // No combo yet
        assert_eq!(result.total_score, 500);
        assert_eq!(scoring.current_combo(), 1);
    }
    
    #[test]
    fn test_combo_system() {
        let mut scoring = TetrisScoring::new();
        let level = 2;
        
        // First line clear - establishes combo
        let action1 = ScoringAction {
            line_clear_type: LineClearType::Single,
            perfect_clear: None,
            level,
            combo: 0,
            back_to_back: false,
        };
        scoring.process_line_clear(action1);
        
        // Second line clear - combo bonus should apply
        let action2 = ScoringAction {
            line_clear_type: LineClearType::Double,
            perfect_clear: None,
            level,
            combo: scoring.current_combo(),
            back_to_back: false,
        };
        
        let result = scoring.process_line_clear(action2);
        assert_eq!(result.base_score, 300 * level); // 600 points base
        assert_eq!(result.combo_bonus, 50 * 1 * level); // 100 points combo bonus
        assert_eq!(result.total_score, 700);
        assert_eq!(scoring.current_combo(), 2);
    }
    
    #[test]
    fn test_back_to_back_tetris() {
        let mut scoring = TetrisScoring::new();
        let level = 3;
        
        // First Tetris - establishes back-to-back readiness
        let action1 = ScoringAction {
            line_clear_type: LineClearType::Tetris,
            perfect_clear: None,
            level,
            combo: 0,
            back_to_back: false,
        };
        scoring.process_line_clear(action1);
        assert!(scoring.is_back_to_back_ready());
        
        // Second Tetris - should get back-to-back bonus
        let action2 = ScoringAction {
            line_clear_type: LineClearType::Tetris,
            perfect_clear: None,
            level,
            combo: scoring.current_combo(),
            back_to_back: true,
        };
        
        let result = scoring.process_line_clear(action2);
        let expected_base = 800 * level; // 2400 points
        let expected_combo = 50 * 1 * level; // 150 points  
        let expected_b2b = expected_base / 2; // 1200 points (50% bonus)
        
        assert_eq!(result.base_score, expected_base);
        assert_eq!(result.combo_bonus, expected_combo);
        assert_eq!(result.back_to_back_bonus, expected_b2b);
        assert_eq!(result.total_score, expected_base + expected_combo + expected_b2b);
    }
    
    #[test]
    fn test_t_spin_scoring() {
        let level = 4;
        
        let t_spin_double = LineClearType::TSpinDouble;
        assert_eq!(t_spin_double.base_score(), 1200);
        assert_eq!(t_spin_double.lines_cleared(), 2);
        assert!(t_spin_double.is_difficult());
        assert!(t_spin_double.is_t_spin());
    }
    
    #[test]
    fn test_perfect_clear_bonus() {
        let mut scoring = TetrisScoring::new();
        let level = 2;
        
        let action = ScoringAction {
            line_clear_type: LineClearType::Single,
            perfect_clear: Some(PerfectClearType::Single),
            level,
            combo: 0,
            back_to_back: false,
        };
        
        let result = scoring.process_line_clear(action);
        assert_eq!(result.base_score, 100 * level); // 200 points
        assert_eq!(result.perfect_clear_bonus, 800 * level); // 1600 points
        assert_eq!(result.total_score, 1800);
    }
    
    #[test]
    fn test_line_clear_type_determination() {
        assert_eq!(determine_line_clear_type(1, false, false), Some(LineClearType::Single));
        assert_eq!(determine_line_clear_type(4, false, false), Some(LineClearType::Tetris));
        assert_eq!(determine_line_clear_type(2, true, false), Some(LineClearType::TSpinDouble));
        assert_eq!(determine_line_clear_type(1, true, true), Some(LineClearType::TSpinMiniSingle));
        assert_eq!(determine_line_clear_type(0, false, false), None);
    }
    fn clear(kind: LineClearType, perfect: bool) -> ScoringAction {
        ScoringAction {
            line_clear_type: kind,
            perfect_clear: if perfect { PerfectClearDetector::determine_perfect_clear_type(kind.lines_cleared()) } else { None },
            level: 1,
            combo: 0,
            back_to_back: kind.is_difficult(),
        }
    }

    #[test]
    fn records_each_clear() {
        let mut scoring = TetrisScoring::new();
        assert_eq!((scoring.clears(), scoring.pieces(), scoring.last_clear()), (0, 0, None));
        scoring.process_line_clear(clear(LineClearType::Double, false));
        assert_eq!(scoring.clears(), 1);
        assert_eq!(scoring.last_clear().map(|c| (c.kind, c.chain)), Some((LineClearType::Double, 0)));
        scoring.process_line_clear(clear(LineClearType::Single, false));
        assert_eq!(scoring.clears(), 2);
        assert_eq!(scoring.last_clear().map(|c| (c.kind, c.chain)), Some((LineClearType::Single, 1)));
    }

    #[test]
    fn no_clear_breaks_chain() {
        let mut scoring = TetrisScoring::new();
        scoring.process_line_clear(clear(LineClearType::Single, false));
        scoring.process_no_line_clear();
        assert_eq!(scoring.clears(), 1);
        scoring.process_line_clear(clear(LineClearType::Single, false));
        assert_eq!(scoring.last_clear().map(|c| c.chain), Some(0));
    }

    #[test]
    fn perfect_and_back_to_back_recorded() {
        let mut scoring = TetrisScoring::new();
        scoring.process_line_clear(clear(LineClearType::Tetris, false));
        let first = scoring.last_clear().unwrap();
        assert!(!first.back_to_back && !first.perfect);
        scoring.process_line_clear(clear(LineClearType::Tetris, true));
        let second = scoring.last_clear().unwrap();
        assert!(second.back_to_back, "{second:?}");
        assert!(second.perfect, "{second:?}");
    }

    #[test]
    fn old_save_deserializes() {
        let old = r#"{"combo_count":2,"back_to_back_ready":true,"total_score":900}"#;
        let scoring: TetrisScoring = serde_json::from_str(old).unwrap();
        assert_eq!((scoring.clears(), scoring.pieces(), scoring.last_clear()), (0, 0, None));
        assert_eq!(scoring.total_score(), 900);
    }

    #[test]
    fn run_stats_tally() {
        let mut scoring = TetrisScoring::new();
        let tspin = |kind| ScoringAction { line_clear_type: kind, perfect_clear: None, level: 1, combo: 0, back_to_back: true };
        scoring.process_line_clear(clear(LineClearType::Tetris, false));
        scoring.process_line_clear(tspin(LineClearType::TSpinDouble));
        scoring.process_line_clear(clear(LineClearType::Tetris, true));
        let s = scoring.stats();
        assert_eq!((s.tetrises, s.t_spins, s.perfect_clears), (2, 1, 1));
        assert_eq!((s.b2b_streak, s.b2b_best), (3, 3));
        assert_eq!(s.best_combo, 2);
        scoring.process_line_clear(clear(LineClearType::Single, false));
        scoring.process_no_line_clear();
        let s = scoring.stats();
        assert_eq!((s.b2b_streak, s.b2b_best, s.best_combo), (0, 3, 3));
    }
}
