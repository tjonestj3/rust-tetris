//! The iPhone Duo's extras: folding, unfolding or turning the phone mid-game pauses it, and the
//! open screen counts 3-2-1 back into play; the callout for a special clear.
//!
//! Only on touch screens (the app doesn't call this on desktop).

use crate::game::GameState;
use crate::graphics::view::HudStyle;
use crate::scoring::{ClearRecord, LineClearType};

/// Seconds of countdown before play resumes.
pub const COUNTDOWN: f32 = 3.0;
/// How long the screen must keep its size before the open screen counts back in (a fold sends a
/// few sizes in a row).
const SETTLE: f32 = 0.4;
/// A change smaller than this (points or pixels) isn't a fold.
const SIZE_SLACK: f32 = 1.0;
/// How long a callout shows, in seconds.
pub const CALLOUT_TIME: f32 = 1.6;

/// What the app should do to the game this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseAction {
    None,
    /// The screen changed size mid-game: pause.
    Pause,
    /// The countdown finished: unpause.
    Resume,
}

/// Watches the screen size and runs the 3-2-1 countdown back into play.
#[derive(Debug, Clone, Default)]
pub struct PauseFlow {
    size: Option<(f32, f32)>,
    /// Paused because the screen changed (not by the player), so it can resume by itself.
    auto_paused: bool,
    /// Seconds since the size last changed.
    settled: f32,
    /// Seconds left on the countdown, while counting.
    countdown: Option<f32>,
}

impl PauseFlow {
    /// Call each frame a game is on screen. `size`: the screen's size; `style`: the layout it got.
    pub fn frame(&mut self, size: (f32, f32), style: HudStyle, state: GameState, dt: f32) -> PauseAction {
        // A size that hadn't settled yet (the first frames of a window, UIKit's launch-time view)
        // changing again isn't a fold.
        let was_settled = self.settled >= SETTLE;
        if self.note_size(size) && was_settled {
            if self.countdown.take().is_some() {
                // Folded again mid-countdown: stay paused, as if it had just happened.
                self.auto_paused = true;
                return PauseAction::None;
            }
            if state == GameState::Playing {
                self.auto_paused = true;
                return PauseAction::Pause;
            }
        } else {
            self.settled += dt;
        }
        // (`settled` restarts at 0 on any change, in `note_size`.)
        if state != GameState::Paused {
            self.auto_paused = false;
            self.countdown = None;
            return PauseAction::None;
        }
        if let Some(left) = self.countdown {
            let left = left - dt;
            if left <= 0.0 {
                self.countdown = None;
                return PauseAction::Resume;
            }
            self.countdown = Some(left);
            return PauseAction::None;
        }
        // Opened up (or turned sideways): the big layout, so count back in.
        if self.auto_paused && self.settled >= SETTLE && style == HudStyle::TwoColumns {
            self.auto_paused = false;
            self.countdown = Some(COUNTDOWN);
        }
        PauseAction::None
    }

    /// Call each frame no game is on screen: keeps up with the size, forgets any pause.
    pub fn idle(&mut self, size: (f32, f32)) {
        self.note_size(size);
        self.auto_paused = false;
        self.countdown = None;
    }

    /// The player tapped Resume: count back in rather than drop straight into play.
    pub fn resume_pressed(&mut self) {
        self.auto_paused = false;
        if self.countdown.is_none() {
            self.countdown = Some(COUNTDOWN);
        }
    }

    /// Seconds left while counting back in.
    pub fn countdown(&self) -> Option<f32> {
        self.countdown
    }

    /// Remember the size; true if it changed (not on the first frame).
    fn note_size(&mut self, size: (f32, f32)) -> bool {
        let changed = match self.size {
            Some((w, h)) => (w - size.0).abs() > SIZE_SLACK || (h - size.1).abs() > SIZE_SLACK,
            None => false,
        };
        if changed || self.size.is_none() {
            self.size = Some(size);
            self.settled = 0.0;
        }
        changed
    }
}

/// The big words for a clear worth calling out, a line each; nothing for a plain clear.
pub fn callout_for(clear: &ClearRecord) -> Vec<String> {
    let mut lines = Vec::new();
    if clear.perfect {
        lines.push("PERFECT CLEAR".to_string());
    }
    let special = clear.kind.is_t_spin() || clear.kind == LineClearType::Tetris;
    if special {
        let name = clear.kind.name().to_uppercase();
        lines.push(if clear.back_to_back { format!("BACK-TO-BACK {name}") } else { name });
    }
    if clear.chain >= 1 {
        lines.push(format!("COMBO {}", clear.chain));
    }
    lines
}

/// A callout on screen: its lines and how long it's been up.
#[derive(Debug, Clone, Default)]
pub struct Callout {
    pub lines: Vec<String>,
    pub age: f32,
}

impl Callout {
    /// Show a new clear's callout (if it has one; otherwise the old one carries on).
    pub fn show(&mut self, clear: &ClearRecord) {
        let lines = callout_for(clear);
        if !lines.is_empty() {
            *self = Callout { lines, age: 0.0 };
        }
    }

    pub fn tick(&mut self, dt: f32) {
        self.age += dt;
        if self.age > CALLOUT_TIME {
            self.lines.clear();
        }
    }

    /// 0..1: fades in fast, holds, fades out over the last third.
    pub fn alpha(&self) -> f32 {
        if self.lines.is_empty() {
            return 0.0;
        }
        (self.age * 8.0).min(1.0).min((CALLOUT_TIME - self.age) / (CALLOUT_TIME / 3.0)).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FOLDED: (f32, f32) = (1374.0, 2001.0);
    const OPEN: (f32, f32) = (1704.0, 1347.0);
    const FRAME: f32 = 1.0 / 60.0;

    fn run(flow: &mut PauseFlow, size: (f32, f32), style: HudStyle, state: GameState, secs: f32) -> Vec<PauseAction> {
        let mut out = Vec::new();
        let mut t = 0.0;
        while t < secs {
            out.push(flow.frame(size, style, state, FRAME));
            t += FRAME;
        }
        out
    }

    #[test]
    fn unfolding_mid_game_pauses_then_counts_back_in() {
        let mut flow = PauseFlow::default();
        let quiet = run(&mut flow, FOLDED, HudStyle::OneColumn, GameState::Playing, SETTLE + 0.05);
        assert!(quiet.iter().all(|a| *a == PauseAction::None));
        assert_eq!(flow.frame(OPEN, HudStyle::TwoColumns, GameState::Playing, FRAME), PauseAction::Pause);
        // Paused now; settles, then counts down.
        run(&mut flow, OPEN, HudStyle::TwoColumns, GameState::Paused, SETTLE + 0.05);
        assert!(flow.countdown().is_some());
        let actions = run(&mut flow, OPEN, HudStyle::TwoColumns, GameState::Paused, COUNTDOWN + 0.05);
        assert_eq!(actions.iter().filter(|a| **a == PauseAction::Resume).count(), 1);
        assert_eq!(flow.countdown(), None);
    }

    #[test]
    fn folding_mid_game_stays_paused() {
        let mut flow = PauseFlow::default();
        run(&mut flow, OPEN, HudStyle::TwoColumns, GameState::Playing, SETTLE + 0.05);
        assert_eq!(flow.frame(FOLDED, HudStyle::OneColumn, GameState::Playing, FRAME), PauseAction::Pause);
        let actions = run(&mut flow, FOLDED, HudStyle::OneColumn, GameState::Paused, 5.0);
        assert!(actions.iter().all(|a| *a == PauseAction::None));
        assert_eq!(flow.countdown(), None);
    }

    #[test]
    fn folding_during_the_countdown_cancels_it() {
        let mut flow = PauseFlow::default();
        run(&mut flow, FOLDED, HudStyle::OneColumn, GameState::Playing, SETTLE + 0.05);
        assert_eq!(flow.frame(OPEN, HudStyle::TwoColumns, GameState::Playing, FRAME), PauseAction::Pause);
        run(&mut flow, OPEN, HudStyle::TwoColumns, GameState::Paused, SETTLE + 0.5);
        assert!(flow.countdown().is_some());
        assert_eq!(flow.frame(FOLDED, HudStyle::OneColumn, GameState::Paused, FRAME), PauseAction::None);
        assert_eq!(flow.countdown(), None);
        // Open again: counts in again.
        flow.frame(OPEN, HudStyle::TwoColumns, GameState::Paused, FRAME);
        run(&mut flow, OPEN, HudStyle::TwoColumns, GameState::Paused, SETTLE + 0.05);
        assert!(flow.countdown().is_some());
    }

    #[test]
    fn a_pause_by_the_player_waits_for_resume() {
        let mut flow = PauseFlow::default();
        flow.frame(OPEN, HudStyle::TwoColumns, GameState::Playing, FRAME);
        // Paused with the button; nothing happens by itself, even across a fold and back.
        run(&mut flow, OPEN, HudStyle::TwoColumns, GameState::Paused, 1.0);
        flow.frame(FOLDED, HudStyle::OneColumn, GameState::Paused, FRAME);
        run(&mut flow, OPEN, HudStyle::TwoColumns, GameState::Paused, 2.0);
        assert_eq!(flow.countdown(), None);
        flow.resume_pressed();
        assert_eq!(flow.countdown(), Some(COUNTDOWN));
        let actions = run(&mut flow, OPEN, HudStyle::TwoColumns, GameState::Paused, COUNTDOWN + 0.05);
        assert!(actions.contains(&PauseAction::Resume));
    }

    #[test]
    fn sizes_settling_at_the_start_arent_a_fold() {
        // A window (or UIKit's launch-time view) taking a few frames to reach its size.
        let mut flow = PauseFlow::default();
        assert_eq!(flow.frame((900.0, 750.0), HudStyle::TwoColumns, GameState::Playing, FRAME), PauseAction::None);
        assert_eq!(flow.frame(OPEN, HudStyle::TwoColumns, GameState::Playing, FRAME), PauseAction::None);
        assert_eq!(flow.frame((1700.0, 1340.0), HudStyle::TwoColumns, GameState::Playing, FRAME), PauseAction::None);
        // Once it has held its size a moment, a change is a fold.
        run(&mut flow, OPEN, HudStyle::TwoColumns, GameState::Playing, SETTLE + 0.05);
        assert_eq!(flow.frame(FOLDED, HudStyle::OneColumn, GameState::Playing, FRAME), PauseAction::Pause);
    }

    #[test]
    fn a_size_seen_from_the_menu_isnt_a_fold() {
        let mut flow = PauseFlow::default();
        flow.idle(FOLDED);
        flow.idle(OPEN);
        assert_eq!(flow.frame(OPEN, HudStyle::TwoColumns, GameState::Playing, FRAME), PauseAction::None);
    }

    fn rec(kind: LineClearType, chain: u32, back_to_back: bool, perfect: bool) -> ClearRecord {
        ClearRecord { kind, chain, back_to_back, perfect }
    }

    #[test]
    fn callouts() {
        assert!(callout_for(&rec(LineClearType::Double, 0, false, false)).is_empty());
        assert_eq!(callout_for(&rec(LineClearType::TSpinDouble, 0, false, false)), ["T-SPIN DOUBLE"]);
        assert_eq!(callout_for(&rec(LineClearType::Tetris, 0, true, false)), ["BACK-TO-BACK TETRIS"]);
        assert_eq!(callout_for(&rec(LineClearType::Single, 4, false, false)), ["COMBO 4"]);
        assert_eq!(callout_for(&rec(LineClearType::Tetris, 1, false, true)), ["PERFECT CLEAR", "TETRIS", "COMBO 1"]);
    }

    #[test]
    fn a_callout_fades_and_ends() {
        let mut c = Callout::default();
        assert_eq!(c.alpha(), 0.0);
        c.show(&rec(LineClearType::Tetris, 0, false, false));
        c.tick(0.5);
        assert_eq!(c.alpha(), 1.0);
        c.show(&rec(LineClearType::Single, 0, false, false));
        assert!(!c.lines.is_empty(), "a plain clear leaves the last callout up");
        c.tick(CALLOUT_TIME);
        assert_eq!(c.alpha(), 0.0);
        assert!(c.lines.is_empty());
    }
}
