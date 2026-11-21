//! Haptics on iPhone: a tap on a hard drop, a bigger one on a clear, patterns for Tetrises,
//! T-spins, combos, perfect clears and the ghost-block fireball.
//!
//! The game asks for haptics with [`play`] during a frame and [`flush`] plays the strongest one
//! at the end of it, so a hard drop that clears lines gives the clear's pattern, not both. The
//! patterns themselves are in `ios/App/Haptics.m`; off iOS this does nothing.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Something worth feeling. Later variants beat earlier ones in the same frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Haptic {
    /// A piece slammed down.
    HardDrop,
    /// Lines cleared (1 to 3).
    Clear(u32),
    /// A clear that continues a combo: the clears in a row before it (1 = "combo 1").
    Combo(u32),
    /// Four lines at once.
    Tetris,
    /// A T-spin clear.
    TSpin,
    /// A ghost block thrown.
    FireballThrow,
    /// A ghost block landing.
    FireballHit,
    /// The board left empty.
    PerfectClear,
    /// The game ended.
    GameOver,
}

impl Haptic {
    /// Which wins when two come in the same frame.
    fn rank(self) -> u8 {
        match self {
            Haptic::HardDrop => 1,
            Haptic::Clear(_) => 2,
            Haptic::Combo(_) => 3,
            Haptic::Tetris => 4,
            Haptic::TSpin => 5,
            Haptic::FireballThrow => 6,
            Haptic::FireballHit => 7,
            Haptic::PerfectClear => 8,
            Haptic::GameOver => 9,
        }
    }

    /// (kind, amount) for `rt_haptic` (ios/App/Haptics.m); the kinds must match there.
    fn code(self) -> (i32, i32) {
        match self {
            Haptic::HardDrop => (0, 0),
            Haptic::Clear(lines) => (1, lines as i32),
            Haptic::Tetris => (2, 0),
            Haptic::TSpin => (3, 0),
            Haptic::Combo(chain) => (4, chain as i32),
            Haptic::PerfectClear => (5, 0),
            Haptic::FireballThrow => (6, 0),
            Haptic::FireballHit => (7, 0),
            Haptic::GameOver => (8, 0),
        }
    }

    fn from_code(kind: i32, amount: i32) -> Option<Self> {
        let amount = amount.max(0) as u32;
        Some(match kind {
            0 => Haptic::HardDrop,
            1 => Haptic::Clear(amount),
            2 => Haptic::Tetris,
            3 => Haptic::TSpin,
            4 => Haptic::Combo(amount),
            5 => Haptic::PerfectClear,
            6 => Haptic::FireballThrow,
            7 => Haptic::FireballHit,
            8 => Haptic::GameOver,
            _ => return None,
        })
    }
}

/// The haptic for a clear as it starts: `lines` being cleared, whether the piece was a T-spin,
/// and the clears in a row before this one. The most special thing about it wins.
pub fn for_clear(lines: u32, t_spin: bool, chain: u32) -> Haptic {
    if t_spin {
        Haptic::TSpin
    } else if lines >= 4 {
        Haptic::Tetris
    } else if chain >= 1 {
        Haptic::Combo(chain)
    } else {
        Haptic::Clear(lines)
    }
}

static ENABLED: AtomicBool = AtomicBool::new(true);
/// This frame's strongest request, packed as rank << 48 | kind << 32 | amount; 0 = none.
static PENDING: AtomicU64 = AtomicU64::new(0);

/// Haptics on or off (the Settings switch).
pub fn set_enabled(on: bool) {
    ENABLED.store(on, Ordering::Relaxed);
}

/// Ask for a haptic this frame (played by [`flush`]).
pub fn play(h: Haptic) {
    let (kind, amount) = h.code();
    let packed = (h.rank() as u64) << 48 | (kind as u64 & 0xffff) << 32 | amount as u32 as u64;
    PENDING.fetch_max(packed, Ordering::Relaxed);
}

/// Play this frame's haptic, if any, and clear it. Returns what was played (for tests and logs).
pub fn flush() -> Option<Haptic> {
    let packed = PENDING.swap(0, Ordering::Relaxed);
    if packed == 0 || !ENABLED.load(Ordering::Relaxed) {
        return None;
    }
    let h = Haptic::from_code((packed >> 32 & 0xffff) as i32, packed as u32 as i32)?;
    let (kind, amount) = h.code();
    native(kind, amount);
    Some(h)
}

#[cfg(target_os = "ios")]
fn native(kind: i32, amount: i32) {
    extern "C" {
        fn rt_haptic(kind: i32, amount: i32);
    }
    unsafe { rt_haptic(kind, amount) }
}

#[cfg(not(target_os = "ios"))]
fn native(kind: i32, amount: i32) {
    log::debug!("haptic {:?}", Haptic::from_code(kind, amount));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_clear_priority() {
        assert_eq!(for_clear(2, true, 3), Haptic::TSpin);
        assert_eq!(for_clear(4, false, 3), Haptic::Tetris);
        assert_eq!(for_clear(2, false, 1), Haptic::Combo(1));
        assert_eq!(for_clear(3, false, 0), Haptic::Clear(3));
    }

    #[test]
    fn strongest_in_a_frame_wins_and_flush_clears() {
        // One test for the shared state, so parallel tests can't interleave.
        set_enabled(true);
        flush();
        play(Haptic::Clear(2));
        play(Haptic::HardDrop);
        assert_eq!(flush(), Some(Haptic::Clear(2)));
        assert_eq!(flush(), None);
        play(Haptic::Combo(2));
        play(Haptic::Combo(5));
        assert_eq!(flush(), Some(Haptic::Combo(5)));
        set_enabled(false);
        play(Haptic::Tetris);
        assert_eq!(flush(), None);
        set_enabled(true);
        assert_eq!(flush(), None, "a request made while off isn't played later");
    }
}
