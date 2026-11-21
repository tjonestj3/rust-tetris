# iPhone polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Haptics, iPhone Duo fold extras, a TestFlight expiry guard and Game Center for the iOS build.

**Architecture:** Game events are read in `app.rs` after each frame's update (as the sounds are);
scoring records each clear so the app can tell what kind it was. iOS-only native calls go through
`ios/App/Haptics.m` and `ios/App/GameCenter.m`, declared in Rust behind `cfg(target_os = "ios")` with desktop no-ops.

**Tech Stack:** Rust + macroquad 0.4 (vendored miniquad), Objective-C (UIKit, Core Haptics,
GameKit), XcodeGen, fastlane, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-11-20-ios-polish-design.md`

## Global Constraints

- Desktop play unchanged; everything new is touch / iOS only (desktop `--touch` may show it for screenshots).
- `src/rotation/` and `Game::drop_current_piece` untouched (fix-failing-tests); `src/game/state.rs` only for the combo / T-spin fixes.
- Old save and settings files still load (`#[serde(default)]` on every new field).
- `cargo test --lib` passes except the 5 known failures fixed on `tjonestj3/fix-failing-tests`;
  `cargo check -p rust-tetris-ios --target aarch64-apple-ios` passes.
- One commit series per part on `tjonestj3/ios-polish`; no push; no TestFlight build without asking.
- Leaderboard ID `rusttetris.highscore`. Rebuild threshold 75 days.

## Review Focus

- Loading a save from before this change: scoring's new fields default; no false "clear" event on load.
- Combo after a lock without a clear: chain resets (needs the `lock_current_piece` call).
- Fold mid line-clear animation or mid fireball: pause, no lost or doubled events.
- Rapid fold/unfold while counting down: countdown cancels and the game stays paused.
- Game Center signed out / offline / not set up in App Store Connect: game plays, no Global button, no crash.

---

### Task 1: Clear records in scoring

**Files:** Modify `src/scoring/mod.rs`, `src/game/state.rs` (`lock_current_piece`, `hard_drop`, `add_score_for_lines`, `lock_was_t_spin`); create `src/game/t_spin_tests.rs`.

**Interfaces — Produces:**
- `pub struct ClearRecord { pub kind: LineClearType, pub chain: u32, pub back_to_back: bool, pub perfect: bool }`
  (`chain` = consecutive clears before this one: 0 first clear, 1 = "COMBO 1").
- `TetrisScoring::{clears() -> u32, last_clear() -> Option<ClearRecord>, pieces() -> u32}`;
  `clears` +1 per `process_line_clear`; `pieces` +1 per `process_line_clear` and `process_no_line_clear`.

- [x] Tests: `records_each_clear` (two clears → clears 2, chain 0 then 1); `no_clear_breaks_chain`;
  `old_save_deserializes` (JSON without the new fields → zeros/None); `perfect_and_b2b_recorded`.
- [x] Implement; call `process_no_line_clear()` in `lock_current_piece` when no lines complete, before the game-over check.
- [x] `cargo test --lib scoring` passes; commit.

### Task 2: Haptics

**Files:** Create `src/haptics.rs`, `ios/App/Haptics.m`; modify `src/lib.rs`, `src/app.rs`,
`src/menu/mod.rs`, `ios/project.yml` (CoreHaptics, GameKit frameworks), `ios/README.md`.

**Interfaces — Produces:** `pub enum Haptic { HardDrop, Clear(u32), Tetris, TSpin, Combo(u32), PerfectClear, FireballThrow, FireballHit, GameOver }`;
`haptics::for_clear(&ClearRecord) -> Haptic` (perfect > T-spin > Tetris > combo (chain ≥ 1) > Clear(lines));
`haptics::play(Haptic)`; `haptics::set_enabled(bool)`; C: `void rt_haptic(int kind, int amount)`.
`GameSettings::haptics_enabled: bool` (serde default true).

- [x] Tests: `for_clear_priority` (one case per rung), `settings_default_haptics_on` (old JSON → true).
- [x] Implement `Haptics.m` (prepared impact generators; CHHapticEngine patterns with impact fallback).
- [x] Hook in `app.rs`: hard drop (both input paths), clear-counter change, `ghost_throw_active` edges, game over.
- [x] Touch settings: HAPTICS switch row under LEGACY; tap bands and `content_bounds` moved down.
- [x] `cargo test --lib`, iOS `cargo check`, desktop screenshot of settings; commit.

### Task 3: Duo extras

**Files:** Create `src/app/duo.rs` (fold watch + countdown, run stats); modify `src/app.rs`, `src/app/touch_view.rs`,
`src/graphics/view.rs` (`PlayLayout::feed: Option<Rect>`, right column between Next and pause in `TwoColumns`).

**Interfaces — Produces:** `FoldWatch::frame(&mut self, size: (f32, f32), style: HudStyle, state: GameState, dt: f32) -> FoldEvent`
with `FoldEvent { None, Pause, StartCountdown, CancelCountdown }`; `RunStats` (T-spins, Tetrises, best chain,
B2B streak/best, pieces, perfect clears, pps) fed by `ClearRecord`s and `pieces()`; `Callout { text, age }`.

- [x] Tests: size change while playing → Pause; stable 0.4 s in TwoColumns after auto-pause → StartCountdown;
  size change during countdown → CancelCountdown; OneColumn stays paused; desktop not touch → None;
  `feed` present and inside safe area for open Duo, absent for folded/strip; run stats tally.
- [x] Implement; touch Resume (P while Paused) starts the 3-2-1 instead of unpausing; countdown drawn over the board.
- [x] Draw run stats + callout in `feed` (both looks); screenshots at 568x449 and 458x574; commit.

### Task 4: TestFlight refresh

**Files:** Modify `ios/fastlane/Fastfile` (lane `testflight_age`), create `.github/workflows/testflight-refresh.yml`, modify `ios/README.md`.

- [x] Lane writes whole days since the newest TestFlight build's upload to `build/testflight_age`.
- [x] Workflow: Mondays 14:00 UTC + manual (`force` boolean); ubuntu; `actions: write`; dispatches `ios.yml` on `main` when ≥ 75 days or forced.
- [x] `actionlint` if available, YAML parse check; commit.

### Task 5: Game Center

**Files:** Create `src/game_center.rs`, `ios/App/RustTetris.entitlements`; create `ios/App/GameCenter.m`; modify `ios/project.yml`,
`src/app.rs`, `src/menu/mod.rs`, `ios/README.md`.

**Interfaces — Produces:** C `rt_gc_authenticate()`, `rt_gc_available() -> bool`, `rt_gc_submit_score(int64_t)`, `rt_gc_show_leaderboard()`;
Rust wrappers of the same names without `rt_gc_`.

- [x] Authenticate after the first frame; submit at game over; leaderboard screen shows a GLOBAL button on iOS when available.
- [x] iOS `cargo check`, desktop leaderboard screenshot unchanged; README setup steps; commit.
