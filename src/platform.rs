//! Platform differences: where save files go, and the bits of UIKit the game asks about on iPhone.

use std::path::PathBuf;

/// Running on a touch screen device (iPhone/iPad). Desktop can opt in with `TETRIS_TOUCH=1` or
/// `--touch` to try the touch controls with the mouse.
pub fn is_touch_device() -> bool {
    static TOUCH: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *TOUCH.get_or_init(|| {
        cfg!(target_os = "ios")
            || std::env::var("TETRIS_TOUCH").map(|v| v == "1").unwrap_or(false)
            || std::env::args().any(|a| a == "--touch")
    })
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

/// The game view's safe area insets in points: (top, left, bottom, right). The view's own, so
/// they follow its window (rotation, the Duo folding) rather than whichever window is first.
#[cfg(target_os = "ios")]
pub fn ios_safe_area_insets() -> Option<(f32, f32, f32, f32)> {
    use objc::runtime::Object;
    use objc::{msg_send, sel, sel_impl};

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct UIEdgeInsets {
        top: f64,
        left: f64,
        bottom: f64,
        right: f64,
    }

    unsafe {
        let view = macroquad::miniquad::window::apple_view() as *mut Object;
        if view.is_null() {
            return None;
        }
        let i: UIEdgeInsets = msg_send![view, safeAreaInsets];
        Some((i.top as f32, i.left as f32, i.bottom as f32, i.right as f32))
    }
}

/// Drawable pixels per UIKit point (the screen's native scale; see the miniquad patch).
#[cfg(target_os = "ios")]
pub fn ios_content_scale() -> f32 {
    use objc::runtime::Object;
    use objc::{msg_send, sel, sel_impl};
    unsafe {
        let view = macroquad::miniquad::window::apple_view() as *mut Object;
        if view.is_null() {
            return 2.0;
        }
        let scale: f64 = msg_send![view, contentScaleFactor];
        if scale >= 1.0 { scale as f32 } else { 2.0 }
    }
}

/// The screen's measurements, two short lines: what the game draws at and what UIKit reports
/// (screen, window, view, drawable, scale, safe area). Shown in small print on touch settings.
pub fn display_info() -> String {
    use macroquad::prelude::{screen_height, screen_width};
    let game = format!("game {}x{}", screen_width().round(), screen_height().round());
    #[cfg(target_os = "ios")]
    {
        return format!("{game}  {}", ios_display_info());
    }
    #[allow(unreachable_code)]
    game
}

#[cfg(target_os = "ios")]
fn ios_display_info() -> String {
    use objc::runtime::Object;
    use objc::{class, msg_send, sel, sel_impl};

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Rect {
        x: f64,
        y: f64,
        w: f64,
        h: f64,
    }
    let fmt = |r: Rect| {
        if r.x == 0.0 && r.y == 0.0 {
            format!("{}x{}", r.w, r.h)
        } else {
            format!("{}x{}@{},{}", r.w, r.h, r.x, r.y)
        }
    };

    unsafe {
        let screen: *mut Object = msg_send![class!(UIScreen), mainScreen];
        let sb: Rect = msg_send![screen, bounds];
        let nb: Rect = msg_send![screen, nativeBounds];
        let scale: f64 = msg_send![screen, scale];
        let native: f64 = msg_send![screen, nativeScale];
        let mut out = format!("screen {} native {} x{scale}/{native}", fmt(sb), fmt(nb));

        let view = macroquad::miniquad::window::apple_view() as *mut Object;
        if view.is_null() {
            return out;
        }
        let window: *mut Object = msg_send![view, window];
        if !window.is_null() {
            let wf: Rect = msg_send![window, frame];
            out += &format!("\nwin {}", fmt(wf));
            let scene: *mut Object = msg_send![window, windowScene];
            if !scene.is_null() {
                let space: *mut Object = msg_send![scene, coordinateSpace];
                let b: Rect = msg_send![space, bounds];
                out += &format!(" scene {}", fmt(b));
            }
        } else {
            out += "\nno window";
        }
        let vf: Rect = msg_send![view, frame];
        let csf: f64 = msg_send![view, contentScaleFactor];
        let dw: isize = msg_send![view, drawableWidth];
        let dh: isize = msg_send![view, drawableHeight];
        let (t, l, b, r) = ios_safe_area_insets().unwrap_or_default();
        out += &format!("\nview {} x{csf} gl {dw}x{dh} safe {t},{l},{b},{r}", fmt(vf));
        out
    }
}
