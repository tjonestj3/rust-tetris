//! Touch play: gestures, no on-screen buttons.
//!
//! In a game, anywhere on the screen:
//! - **tap**: rotate clockwise; **two-finger tap**: rotate counter-clockwise
//! - **drag sideways**: move the piece, one cell per step, following the finger
//! - **drag down** (and keep the finger there): soft drop
//! - **flick down**: hard drop; **flick up**, or **tap the HOLD panel**: hold
//!
//! The one thing drawn is a small pause icon in the corner (and Resume / Menu while paused); the
//! HOLD panel (drawn by app::touch_view) is a button too.
//! Menus are tapped directly ([`canvas_tap`]).
//!
//! Once a frame, before any input is read, the app calls [`begin_frame`]; the game reads
//! [`gestures`], the menus read [`canvas_tap`], and the pause icon / pause buttons come through
//! [`key_pressed`] as the P / Escape keys so the existing key handling does the rest. After drawing
//! the game the app calls [`draw`].

use std::cell::RefCell;
use std::collections::VecDeque;

use macroquad::prelude::*;
use crate::graphics::text::{draw_text, measure_text};

use crate::graphics::view::Layout;

/// What the screen is showing, which decides how touches are read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    None,
    /// Menus: taps on the canvas.
    Menu,
    /// Playing: gestures, plus the pause icon.
    Game,
    /// Paused: Resume / Menu buttons.
    Paused,
    /// Typing a high score name (the iOS keyboard is up).
    NameEntry,
}

/// What the player's fingers did this frame, for the game to apply.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Gestures {
    /// Cells to move: negative left, positive right.
    pub moves: i32,
    pub rotate_cw: bool,
    pub rotate_ccw: bool,
    pub hard_drop: bool,
    pub hold: bool,
    /// Soft drop while this is true.
    pub soft_drop: bool,
}

/// One finger this frame.
#[derive(Debug, Clone, Copy)]
struct Finger {
    id: u64,
    pos: Vec2,
    /// First frame we've seen it.
    new: bool,
    /// Lifted this frame (it's still reported once).
    ended: bool,
}

/// A gesture in progress: from the first finger down until it lifts.
#[derive(Debug, Clone)]
struct Gesture {
    id: u64,
    start: Vec2,
    start_time: f64,
    /// Where the next sideways step / soft drop is measured from.
    anchor: Vec2,
    /// Furthest the finger has been from `start`.
    travel: f32,
    moved: bool,
    max_fingers: usize,
    /// Recent positions, for the flick speed.
    samples: VecDeque<(f64, Vec2)>,
}

/// A button drawn on screen (the pause icon, Resume / Menu).
/// How far outside a button a tap still counts, in points.
const BUTTON_SLACK: f32 = 10.0;

/// The label of the HOLD panel's button: not drawn here (the panel is the button).
const HOLD_PANEL: &str = "";

#[derive(Debug, Clone, Copy)]
struct Button {
    rect: Rect,
    key: KeyCode,
    label: &'static str,
}

#[derive(Default)]
struct State {
    gesture: Option<Gesture>,
    gestures: Gestures,
    pressed: Vec<KeyCode>,
    canvas_tap: Option<Vec2>,
    buttons: Vec<Button>,
    /// Fingers seen last frame, to tell a new one from one that's been down a while.
    seen_ids: Vec<u64>,
    keyboard_shown: bool,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

/// `is_key_pressed`, or a touch standing in for that key this frame (pause icon, pause buttons).
pub fn key_pressed(key: KeyCode) -> bool {
    is_key_pressed(key) || STATE.with(|s| s.borrow().pressed.contains(&key))
}

/// `is_key_down`. (Touch holds come through [`gestures`], not keys.)
pub fn key_down(key: KeyCode) -> bool {
    is_key_down(key)
}

/// This frame's gestures (all zero when not playing by touch).
pub fn gestures() -> Gestures {
    STATE.with(|s| s.borrow().gestures)
}

/// A tap on the game canvas this frame, in canvas units (for tapping menu items).
pub fn canvas_tap() -> Option<Vec2> {
    STATE.with(|s| s.borrow().canvas_tap)
}

/// Show or hide the on-screen keyboard (for typing a high score name). No-op on desktop.
pub fn set_keyboard(show: bool) {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if s.keyboard_shown != show {
            s.keyboard_shown = show;
            if cfg!(target_os = "ios") {
                macroquad::miniquad::window::show_keyboard(show);
            }
        }
    });
}

/// Read this frame's touches. Call once per frame, before anything reads input.
pub fn begin_frame(layout: &Layout, mode: Mode) {
    let fingers = read_fingers();
    let now = get_time();
    let buttons = buttons_for(layout, mode);

    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.gestures = Gestures::default();
        s.pressed.clear();
        s.canvas_tap = None;
        s.buttons = buttons;
        s.seen_ids = fingers.iter().filter(|f| !f.ended).map(|f| f.id).collect();

        // A new finger on a button presses its key, and is otherwise ignored.
        let mut fresh: Vec<Finger> = Vec::new();
        for f in fingers.iter().filter(|f| f.new) {
            let slack = BUTTON_SLACK * layout.unit;
            match s.buttons.iter().find(|b| grow(b.rect, slack).contains(f.pos)) {
                Some(b) => {
                    let key = b.key;
                    s.pressed.push(key);
                }
                None => fresh.push(*f),
            }
        }

        match mode {
            Mode::Menu | Mode::NameEntry => {
                s.gesture = None;
                s.canvas_tap = fresh.first().and_then(|f| layout.screen_to_canvas(f.pos));
            }
            Mode::Game => track_gesture(&mut s, layout, &fingers, &fresh, now),
            Mode::Paused | Mode::None => s.gesture = None,
        }
    });
}

/// Draw the pause icon / pause buttons, in screen pixels, over everything.
pub fn draw(layout: &Layout) {
    crate::graphics::view::screen_camera();
    STATE.with(|s| {
        for b in &s.borrow().buttons {
            draw_button(b, layout.unit);
        }
    });
}

// ---------------------------------------------------------------------------------------------

/// Every finger on the screen this frame. On desktop (`--touch`) the mouse is one finger.
fn read_fingers() -> Vec<Finger> {
    let seen = STATE.with(|s| s.borrow().seen_ids.clone());
    let touches = touches();
    if !touches.is_empty() {
        return touches
            .iter()
            .map(|t| Finger {
                id: t.id,
                pos: t.position,
                new: !seen.contains(&t.id),
                ended: matches!(t.phase, TouchPhase::Ended | TouchPhase::Cancelled),
            })
            // One we already saw that has ended is just gone; a new one that's ended was a
            // tap too quick to span two frames, and still counts.
            .filter(|f| !(f.ended && !f.new))
            .collect();
    }
    let (down, pressed, released) = (
        is_mouse_button_down(MouseButton::Left),
        is_mouse_button_pressed(MouseButton::Left),
        is_mouse_button_released(MouseButton::Left),
    );
    if down || pressed || released {
        let (x, y) = mouse_position();
        return vec![Finger { id: u64::MAX, pos: vec2(x, y), new: pressed, ended: released && !down }];
    }
    Vec::new()
}

fn track_gesture(s: &mut State, layout: &Layout, fingers: &[Finger], fresh: &[Finger], now: f64) {
    let u = layout.unit;
    // One sideways step: a board cell as drawn, kept to a size a thumb can manage.
    let step = layout.play.cell.clamp(14.0 * u, 30.0 * u);
    let tap_slop = 12.0 * u;

    // A new gesture starts with a new finger when none is going.
    if s.gesture.is_none() {
        if let Some(f) = fresh.first() {
            let mut samples = VecDeque::new();
            samples.push_back((now, f.pos));
            s.gesture = Some(Gesture {
                id: f.id,
                start: f.pos,
                start_time: now,
                anchor: f.pos,
                travel: 0.0,
                moved: false,
                max_fingers: 1,
                samples,
            });
        }
    }

    let Some(g) = s.gesture.as_mut() else { return };
    let down_now = fingers.iter().filter(|f| !f.ended).count();
    g.max_fingers = g.max_fingers.max(down_now.max(fingers.len().min(2)));

    let finger = fingers.iter().find(|f| f.id == g.id).copied();
    let pos = finger.map(|f| f.pos).unwrap_or_else(|| g.samples.back().map(|s| s.1).unwrap_or(g.start));

    g.samples.push_back((now, pos));
    while g.samples.len() > 2 && now - g.samples[0].0 > 0.1 {
        g.samples.pop_front();
    }
    g.travel = g.travel.max(pos.distance(g.start));

    // Sideways: a move per step the finger has gone past the anchor. A second finger (going for
    // a two-finger tap) doesn't move anything.
    if g.max_fingers < 2 {
        let mut moves = 0;
        while pos.x - g.anchor.x >= step {
            moves += 1;
            g.anchor.x += step;
        }
        while g.anchor.x - pos.x >= step {
            moves -= 1;
            g.anchor.x -= step;
        }
        if moves != 0 {
            g.moved = true;
            g.anchor.y = pos.y; // moving sideways isn't a drag down
            s.gestures.moves = moves;
        }
    }

    // Down: soft drop while the finger is a good way below where it was highest (since the last
    // sideways move), and it's mostly a downward drag.
    g.anchor.y = g.anchor.y.min(pos.y);
    let below = pos.y - g.anchor.y;
    let soft = g.max_fingers < 2 && below > 1.5 * step && below > (pos.x - g.anchor.x).abs();
    if soft {
        g.moved = true;
    }
    s.gestures.soft_drop = soft && !finger.map(|f| f.ended).unwrap_or(true);

    // Lifted (or gone): what was it?
    if finger.map(|f| f.ended).unwrap_or(true) {
        let (t0, p0) = g.samples[0];
        let dt = ((now - t0) as f32).max(1.0 / 120.0);
        let v = (pos - p0) / dt / u; // points per second over the last ~0.1 s
        let vertical = v.y.abs() > 1.5 * v.x.abs();
        let quick = now - g.start_time < 0.35;

        if g.travel < tap_slop && quick && !g.moved {
            if g.max_fingers >= 2 {
                s.gestures.rotate_ccw = true;
            } else {
                s.gestures.rotate_cw = true;
            }
        } else if g.max_fingers < 2 && vertical && v.y > 900.0 && pos.y - g.start.y > step {
            s.gestures.hard_drop = true;
            s.gestures.soft_drop = false;
        } else if g.max_fingers < 2 && is_flick_up(g, pos, v, now, step) {
            s.gestures.hold = true;
        }
        s.gesture = None;
    }
}

/// A flick up: well up from where it started, mostly straight, and either still going up fast
/// when lifted or done quickly (a flick that slows before the finger lifts still counts).
fn is_flick_up(g: &Gesture, pos: Vec2, v: Vec2, now: f64, step: f32) -> bool {
    let up = g.start.y - pos.y;
    let straight = up > 1.5 * (pos.x - g.start.x).abs();
    let fast_at_lift = v.y < -400.0 && v.y.abs() > 1.5 * v.x.abs();
    let quick = now - g.start_time < 0.5;
    up > 1.5 * step && straight && (fast_at_lift || quick)
}

fn buttons_for(layout: &Layout, mode: Mode) -> Vec<Button> {
    let u = layout.unit;
    match mode {
        Mode::Game => {
            // Their places in the HUD (graphics::view::PlayLayout). The HOLD panel is drawn by
            // app::touch_view; shrunk by the slack taps get, so a tap just off it still isn't one.
            let slack = BUTTON_SLACK * u;
            let h = layout.play.hold;
            vec![
                Button { rect: layout.play.pause, key: KeyCode::P, label: "II" },
                Button {
                    rect: Rect::new(h.x + slack, h.y + slack, (h.w - 2.0 * slack).max(0.0), (h.h - 2.0 * slack).max(0.0)),
                    key: KeyCode::C,
                    label: HOLD_PANEL,
                },
            ]
        }
        Mode::Paused => {
            // Below the pause screen's text (app::touch_view::draw_pause).
            let (w, h, gap) = ((220.0 * u).min(layout.safe.w * 0.7), 52.0 * u, 14.0 * u);
            let cx = layout.safe.center().x;
            let cy = layout.safe.y + layout.safe.h * 0.8;
            vec![
                Button { rect: Rect::new(cx - w / 2.0, cy - h - gap / 2.0, w, h), key: KeyCode::P, label: "RESUME" },
                Button { rect: Rect::new(cx - w / 2.0, cy + gap / 2.0, w, h), key: KeyCode::Escape, label: "MENU" },
            ]
        }
        _ => Vec::new(),
    }
}

fn grow(r: Rect, by: f32) -> Rect {
    Rect::new(r.x - by, r.y - by, r.w + 2.0 * by, r.h + 2.0 * by)
}

fn draw_button(b: &Button, u: f32) {
    if b.label == HOLD_PANEL {
        return;
    }
    use crate::graphics::style::{self, CYAN, GOLD, INK, PINK};
    let r = b.rect;
    let ink = INK;
    let accent = match b.label {
        "II" => PINK,
        "MENU" => CYAN,
        _ => GOLD,
    };
    style::panel(r, accent, u);
    let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    if b.label == "II" {
        // Two bars, not text: crisper.
        let (bw, bh) = (r.w * 0.14, r.h * 0.45);
        draw_rectangle(cx - bw * 1.6, cy - bh / 2.0, bw, bh, ink);
        draw_rectangle(cx + bw * 0.6, cy - bh / 2.0, bw, bh, ink);
        return;
    }
    let size = r.h * 0.45;
    let dims = measure_text(b.label, None, size as u16, 1.0);
    draw_text(b.label, cx - dims.width / 2.0, cy + dims.offset_y / 2.0, size, ink);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 900x750 screen: a 36 px cell, so a sideways step is 30 px (the cap).
    fn layout() -> Layout {
        let r = Rect::new(0.0, 0.0, 900.0, 750.0);
        Layout::for_screen(r, r, 1.0, crate::graphics::view::FULL_CANVAS)
    }

    /// Feed frames of (time, fingers as (id, x, y, new, ended)); return each frame's gestures.
    fn run(frames: &[(f64, Vec<(u64, f32, f32, bool, bool)>)]) -> Vec<Gestures> {
        let mut s = State::default();
        let l = layout();
        frames
            .iter()
            .map(|(t, fs)| {
                let fingers: Vec<Finger> = fs
                    .iter()
                    .map(|&(id, x, y, new, ended)| Finger { id, pos: vec2(x, y), new, ended })
                    .collect();
                let fresh: Vec<Finger> = fingers.iter().copied().filter(|f| f.new).collect();
                s.gestures = Gestures::default();
                track_gesture(&mut s, &l, &fingers, &fresh, *t);
                s.gestures
            })
            .collect()
    }

    fn total(gs: &[Gestures]) -> Gestures {
        gs.iter().fold(Gestures::default(), |a, g| Gestures {
            moves: a.moves + g.moves,
            rotate_cw: a.rotate_cw || g.rotate_cw,
            rotate_ccw: a.rotate_ccw || g.rotate_ccw,
            hard_drop: a.hard_drop || g.hard_drop,
            hold: a.hold || g.hold,
            soft_drop: a.soft_drop || g.soft_drop,
        })
    }

    #[test]
    fn tap_rotates_clockwise() {
        let g = total(&run(&[(0.0, vec![(1, 100.0, 100.0, true, false)]), (0.1, vec![(1, 102.0, 101.0, false, true)])]));
        assert_eq!(g, Gestures { rotate_cw: true, ..Default::default() });
    }

    #[test]
    fn tap_within_one_frame_still_counts() {
        let g = total(&run(&[(0.0, vec![(1, 100.0, 100.0, true, true)])]));
        assert!(g.rotate_cw);
    }

    #[test]
    fn two_finger_tap_rotates_counter_clockwise() {
        let g = total(&run(&[
            (0.0, vec![(1, 100.0, 100.0, true, false)]),
            (0.03, vec![(1, 100.0, 100.0, false, false), (2, 200.0, 100.0, true, false)]),
            (0.12, vec![(1, 101.0, 100.0, false, true), (2, 200.0, 100.0, false, false)]),
            (0.15, vec![(2, 200.0, 100.0, false, true)]),
        ]));
        assert_eq!(g, Gestures { rotate_ccw: true, ..Default::default() });
    }

    #[test]
    fn drag_sideways_moves_a_cell_per_step() {
        let mut frames = vec![(0.0, vec![(1, 100.0, 300.0, true, false)])];
        for i in 1..=10 {
            frames.push((i as f64 * 0.05, vec![(1, 100.0 + i as f32 * 10.0, 300.0, false, false)]));
        }
        frames.push((0.6, vec![(1, 200.0, 300.0, false, true)]));
        let g = total(&run(&frames));
        assert_eq!(g.moves, 3); // 100 px right = 3 whole 30 px steps
        assert!(!g.rotate_cw && !g.hard_drop && !g.soft_drop);

        let mut frames = vec![(0.0, vec![(1, 300.0, 300.0, true, false)])];
        frames.push((0.2, vec![(1, 235.0, 300.0, false, false)]));
        frames.push((0.6, vec![(1, 235.0, 300.0, false, true)]));
        assert_eq!(total(&run(&frames)).moves, -2);
    }

    #[test]
    fn flick_down_hard_drops() {
        let g = total(&run(&[
            (0.0, vec![(1, 100.0, 100.0, true, false)]),
            (0.03, vec![(1, 100.0, 140.0, false, false)]),
            (0.06, vec![(1, 101.0, 200.0, false, false)]),
            (0.08, vec![(1, 101.0, 240.0, false, true)]),
        ]));
        assert!(g.hard_drop);
        assert!(!g.hold && !g.rotate_cw && g.moves == 0);
    }

    #[test]
    fn slow_ending_flick_up_still_holds() {
        // Up 140 px in a quarter second, nearly stopped when lifted.
        let g = total(&run(&[
            (0.0, vec![(1, 100.0, 400.0, true, false)]),
            (0.08, vec![(1, 102.0, 320.0, false, false)]),
            (0.16, vec![(1, 103.0, 270.0, false, false)]),
            (0.24, vec![(1, 103.0, 262.0, false, false)]),
            (0.26, vec![(1, 103.0, 260.0, false, true)]),
        ]));
        assert!(g.hold);
        assert!(!g.hard_drop && !g.rotate_cw);
    }

    #[test]
    fn slow_drag_up_is_not_a_hold() {
        let g = total(&run(&[
            (0.0, vec![(1, 100.0, 400.0, true, false)]),
            (0.4, vec![(1, 100.0, 330.0, false, false)]),
            (0.8, vec![(1, 100.0, 262.0, false, false)]),
            (0.9, vec![(1, 100.0, 260.0, false, true)]),
        ]));
        assert!(!g.hold);
    }

    #[test]
    fn flick_up_holds() {
        let g = total(&run(&[
            (0.0, vec![(1, 100.0, 400.0, true, false)]),
            (0.03, vec![(1, 100.0, 360.0, false, false)]),
            (0.06, vec![(1, 100.0, 300.0, false, false)]),
            (0.08, vec![(1, 100.0, 260.0, false, true)]),
        ]));
        assert!(g.hold);
        assert!(!g.hard_drop && !g.rotate_cw);
    }

    #[test]
    fn slow_drag_down_soft_drops_and_doesnt_hard_drop() {
        let mut frames = vec![(0.0, vec![(1, 100.0, 100.0, true, false)])];
        for i in 1..=20 {
            frames.push((i as f64 * 0.05, vec![(1, 100.0, 100.0 + i as f32 * 5.0, false, false)]));
        }
        // ...then rests there for a while before lifting.
        frames.push((1.5, vec![(1, 100.0, 200.0, false, false)]));
        frames.push((1.6, vec![(1, 100.0, 200.0, false, true)]));
        let gs = run(&frames);
        assert!(gs[12].soft_drop, "soft drop once well below the start");
        assert!(gs[21].soft_drop, "still soft dropping while resting low");
        let g = total(&gs);
        assert!(!g.hard_drop && !g.rotate_cw);
        assert!(!gs.last().unwrap().soft_drop, "stops on lift");
    }
}
