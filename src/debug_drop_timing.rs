//! Debug utility to analyze drop timing issues

use crate::game::config::*;

/// Debug function to analyze drop interval calculations
pub fn debug_drop_intervals() {
    println!("=== DROP INTERVAL ANALYSIS ===");
    println!("INITIAL_DROP_TIME: {}", INITIAL_DROP_TIME);
    println!("LEVEL_SPEED_MULTIPLIER: {}", LEVEL_SPEED_MULTIPLIER);
    println!();
    
    for level in 1..=20 {
        let level_multiplier = LEVEL_SPEED_MULTIPLIER.powi((level - 1) as i32);
        let drop_interval = INITIAL_DROP_TIME * level_multiplier;
        
        println!("Level {}: multiplier={:.6}, drop_interval={:.6}s ({:.1}ms)", 
                level, level_multiplier, drop_interval, drop_interval * 1000.0);
                
        // Check for problematic values
        if drop_interval < 0.001 {
            println!("  ⚠️  WARNING: Drop interval extremely small!");
        }
        if drop_interval > 10.0 {
            println!("  ⚠️  WARNING: Drop interval extremely large!");
        }
    }
    
    println!();
    println!("=== THEORETICAL FRAME-BASED ANALYSIS ===");
    println!("At 60 FPS, frame time = 16.67ms");
    
    for level in 1..=10 {
        let level_multiplier = LEVEL_SPEED_MULTIPLIER.powi((level - 1) as i32);
        let drop_interval = INITIAL_DROP_TIME * level_multiplier;
        let frames_per_drop = (drop_interval * 60.0).round() as i32;
        
        println!("Level {}: {:.1}ms per drop = {} frames per drop", 
                level, drop_interval * 1000.0, frames_per_drop);
    }
}

/// Check if the current level calculation could cause issues
pub fn validate_drop_timing(level: u32, drop_timer: f64, drop_interval: f64) -> bool {
    // Check for invalid values
    if drop_interval <= 0.0 {
        println!("❌ CRITICAL: drop_interval is zero or negative: {}", drop_interval);
        return false;
    }
    
    if drop_interval < 0.001 {
        println!("⚠️  WARNING: drop_interval very small: {:.6}s", drop_interval);
    }
    
    if drop_timer.is_nan() || drop_interval.is_nan() {
        println!("❌ CRITICAL: NaN detected - timer: {}, interval: {}", drop_timer, drop_interval);
        return false;
    }
    
    if drop_timer.is_infinite() || drop_interval.is_infinite() {
        println!("❌ CRITICAL: Infinite value detected - timer: {}, interval: {}", drop_timer, drop_interval);
        return false;
    }
    
    true
}