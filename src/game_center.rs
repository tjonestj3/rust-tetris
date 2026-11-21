//! Game Center on iPhone: a global High Score leaderboard next to the one kept on the phone.
//!
//! The calls are in `ios/App/GameCenter.m`. Signing in is GameKit's own sheet, offered once at
//! launch; signed out or offline, the game carries on and nothing here does anything. Off iOS
//! it's never available (`--game-center` pretends it is, to check the leaderboard's layout).

/// Sign in to Game Center (once; GameKit decides whether to show its sheet).
pub fn authenticate() {
    #[cfg(target_os = "ios")]
    unsafe {
        ffi::rt_gc_authenticate()
    }
}

/// Signed in, so scores go up and the global board can be shown.
pub fn available() -> bool {
    #[cfg(target_os = "ios")]
    return unsafe { ffi::rt_gc_available() != 0 };
    #[cfg(not(target_os = "ios"))]
    return std::env::args().any(|a| a == "--game-center");
}

/// Send a finished game's score (Game Center keeps the best).
pub fn submit_score(score: u32) {
    #[cfg(target_os = "ios")]
    unsafe {
        ffi::rt_gc_submit_score(score as i64)
    }
    #[cfg(not(target_os = "ios"))]
    log::debug!("Game Center: would submit {score}");
}

/// Show the global High Score board (Game Center's own screen, over the game).
pub fn show_leaderboard() {
    #[cfg(target_os = "ios")]
    unsafe {
        ffi::rt_gc_show_leaderboard()
    }
    #[cfg(not(target_os = "ios"))]
    log::info!("Game Center: would show the global leaderboard");
}

#[cfg(target_os = "ios")]
mod ffi {
    extern "C" {
        pub fn rt_gc_authenticate();
        pub fn rt_gc_available() -> i32;
        pub fn rt_gc_submit_score(score: i64);
        pub fn rt_gc_show_leaderboard();
    }
}
