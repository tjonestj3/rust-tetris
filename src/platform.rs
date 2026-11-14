//! Platform differences: where save files go, and the bits of UIKit the game asks about on iPhone.

use std::path::PathBuf;

/// Running on a touch screen device (iPhone/iPad). Desktop can opt in with `TETRIS_TOUCH=1` or
/// `--touch` to try the touch controls with the mouse.
pub fn is_touch_device() -> bool {
    cfg!(target_os = "ios")
        || std::env::var("TETRIS_TOUCH").map(|v| v == "1").unwrap_or(false)
        || std::env::args().any(|a| a == "--touch")
}

/// The directory save, settings and leaderboard files live in.
///
/// Desktop keeps them next to where the game is run from, as it always has. On iOS the working
/// directory is the read-only app bundle, so they go in the app's Documents folder.
pub fn data_dir() -> PathBuf {
    #[cfg(target_os = "ios")]
    {
        if let Ok(home) = std::env::var("HOME") {
            let docs = PathBuf::from(home).join("Documents");
            let _ = std::fs::create_dir_all(&docs);
            return docs;
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// Path for a data file by name (see [`data_dir`]).
pub fn data_file(name: &str) -> PathBuf {
    data_dir().join(name)
}

/// Whether quitting from the menu makes sense. iOS apps don't quit themselves.
pub const CAN_QUIT: bool = !cfg!(target_os = "ios");

/// The key window's safe area insets in points: (top, left, bottom, right).
#[cfg(target_os = "ios")]
pub fn ios_safe_area_insets() -> Option<(f32, f32, f32, f32)> {
    use objc::runtime::Object;
    use objc::{class, msg_send, sel, sel_impl};

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct UIEdgeInsets {
        top: f64,
        left: f64,
        bottom: f64,
        right: f64,
    }

    unsafe {
        let app: *mut Object = msg_send![class!(UIApplication), sharedApplication];
        if app.is_null() {
            return None;
        }
        let windows: *mut Object = msg_send![app, windows];
        if windows.is_null() {
            return None;
        }
        let window: *mut Object = msg_send![windows, firstObject];
        if window.is_null() {
            return None;
        }
        let i: UIEdgeInsets = msg_send![window, safeAreaInsets];
        Some((i.top as f32, i.left as f32, i.bottom as f32, i.right as f32))
    }
}
