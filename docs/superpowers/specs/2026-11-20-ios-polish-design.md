# iPhone polish: haptics, Duo extras, Game Center, TestFlight refresh

## Why

The iPhone build plays well but feels flat in the hand, does nothing special with the iPhone Duo's
open screen beyond the two-column layout, keeps high scores on the phone only, and quietly stops
working when its TestFlight build expires after 90 days. This round makes it feel like a native
iPhone game (idea 4 of the tetris-ideas review).

"Duo" is the iPhone Duo: **one folding screen**, not two displays. Folded it is upright (about
458x574 pt safe area, `HudStyle::OneColumn`); open it is wider than tall (about 568x449 pt,
`HudStyle::TwoColumns`). See `src/graphics/view.rs`.

## Goals / non-goals / anti-goals

- Goals
  - Haptics for hard drops, line clears (stronger for more lines, Tetris its own), T-spins,
    combos (rising with the chain), perfect clears, and the ghost-block fireball (throw and
    impact). On/off in Settings.
  - Folding, unfolding or rotating mid-game pauses; the open screen resumes with a 3-2-1 countdown.
  - Open Duo: a richer stats panel and a big callout for the last special clear.
  - Game Center: a global high-score leaderboard alongside the local one.
  - TestFlight builds never lapse: a cheap weekly check rebuilds when the latest is getting old.
- Non-goals
  - A multi-piece Next queue (changes gameplay; would touch `game/state.rs`, where the
    fix-failing-tests worktree is working). Possible later round.
  - Game Center achievements, multiplayer, access-point overlay.
  - Desktop changes (all of this is touch / iOS only; desktop builds and plays as today).
- Anti-goals
  - Buzzing on every move or lock: haptics only on events that matter, or it becomes noise.
  - Game Center sign-in nagging or anything that blocks play when offline / signed out.
  - Scheduled Mac builds that run when nothing needs rebuilding.

## Constraints

- Built and checked on Linux: CI runs `cargo check --target aarch64-apple-ios`; real feel and
  Game Center can only be tried on the phone via a TestFlight build (`gh workflow run ios.yml
  --ref <branch>`).
- Stay out of `src/rotation/` and `Game::drop_current_piece` (the fix-failing-tests worktree's
  5 test fixes, commit 40eea58). Base on `origin/main` (a66f826).
- Public repo: GitHub disables scheduled workflows after 60 days without commits; it emails, and
  one click re-enables.

## Approaches

1. **A small Objective-C file in the app (`ios/App/Native.m`) for haptics and Game Center, called
   from Rust through a few `extern "C"` functions** (picked). Core Haptics patterns, GameKit's
   completion blocks and view-controller presentation are a few lines of Obj-C each; from Rust
   they need hand-built NSDictionary patterns and Obj-C blocks. Rust declares the functions only
   for `target_os = "ios"`; desktop gets no-ops.
2. All in Rust through `objc` `msg_send!` (as `platform.rs` does for the safe area). Fine for a
   one-shot `UIImpactFeedbackGenerator`, painful for Core Haptics patterns and GameKit's
   block-based API (needs the `block` crate).
3. A Swift package. Adds a Swift toolchain step to the build for no gain over option 1.

TestFlight refresh: **weekly Linux job checks the latest build's age, dispatches `ios.yml` only
when it is 75+ days old** (picked) over a fixed monthly Mac build (12 Mac builds a year whether
needed or not, and still lapses if one fails unnoticed).

## Design

### 1. Haptics (PR 1)

- `ios/App/Native.m`: `void rt_haptic(int kind, float strength)`. Prepared
  `UIImpactFeedbackGenerator`s for single taps; a `CHHapticEngine` for multi-pulse patterns
  (Tetris double, T-spin triple tick, combo rising taps, fireball rumble). Falls back to impact
  generators when the device has no Core Haptics. `project.yml` adds `-framework CoreHaptics`.
- `src/haptics.rs`: `enum Haptic { HardDrop, Clear(u32), Tetris, TSpin, Combo(u32), PerfectClear,
  FireballThrow, FireballHit, GameOver }` and `play(Haptic)`; respects the setting; no-op off iOS.
- Events, detected in `app.rs` beside the existing sounds (`detect_and_play_audio_events` and the
  hard-drop sound calls), with no change to `game/`:
  - Hard drop: where `SoundType::HardDrop` plays.
  - Clears, T-spin, combo: when the line-clear animation starts (with the flash and the sound),
    from the rows being cleared, `game.lock_was_t_spin` and the current combo. One haptic per
    clear, the most special wins: T-spin > Tetris > combo (chain >= 1) > plain clear.
  - Perfect clear: when the clear is scored at the end of the animation (a `ClearRecord` with
    `perfect`), as a second, bigger buzz with the celebration.
  - Fireball: `ghost_throw_active` going false->true (throw), true->false (impact).
  - Game over: on the transition, like its sound.
- Setting: `haptics_enabled` (default on, missing in old settings files = on) in `GameSettings`;
  a Haptics row in touch Settings, shown on iOS (and with `--touch` on desktop for screenshots).

### 2. Duo extras (PR 2)

- **Auto-pause:** while playing by touch, any change of screen size (fold, unfold, rotate) pauses
  the game, using the normal pause screen. If the game was auto-paused and the new layout is
  `TwoColumns` (open Duo, or a phone turned sideways), a 3-2-1 countdown over the board starts
  and play resumes. Resume from the pause screen also counts down 3-2-1. The decision is a pure
  function of (previous size, new size, style, paused-by-fold) so it's unit-tested.
- **Run stats:** tallied in `TetrisScoring` (saved with the game, so they survive the app being
  killed and the game reloaded): T-spins, Tetrises, best combo, back-to-back streak (current /
  best), perfect clears, pieces placed; pieces per second from pieces and `game_time`.
- **Open layout:** in `TwoColumns` the right column is empty between Next and the pause button.
  `PlayLayout` names that space `feed` (only in `TwoColumns`), and `touch_view.rs` draws the run
  stats there, in both looks. The left stats panel, `OneColumn` and `TopStrip` are unchanged.
- **Clear callout:** on special clears (T-spin, Tetris, combo >= 1, back-to-back, perfect clear)
  a large label, e.g. `T-SPIN DOUBLE`, `COMBO 4`, `BACK-TO-BACK TETRIS`, fades in at the top of
  `feed` for about 1.5 s. Open layout only; folded there's no room.
- Desktop check: `cargo run -- --touch --window 568x449 --play` and `--window 458x574`, screenshots
  via `TETRIS_SCREENSHOT`; layout tests in `view.rs` extended for the taller stats panel.

### 3. TestFlight refresh (PR 3)

- `ios/fastlane/Fastfile`: lane `testflight_age` writes the latest TestFlight build's age in days
  (via the App Store Connect API key, same secrets) to `build/testflight_age`.
- `.github/workflows/testflight-refresh.yml`: weekly (Monday) and by hand (with a `force` input);
  `ubuntu-latest`, Ruby + fastlane, runs the lane; if age >= 75 days (or forced), `gh workflow run
  ios.yml --ref main` with `permissions: actions: write`. Logs the age either way.
- `ios/README.md` Maintenance section updated.

### 4. Game Center (PR 4)

- One-time Apple setup (by hand, written up in `ios/README.md`): Game Center on the App ID;
  App Store Connect → the app → Game Center → a classic leaderboard **High Score**, ID
  `rusttetris.highscore`, integer, high to low. The Fastfile already regenerates the provisioning
  profile on each build, so it picks up the capability.
- `ios/App/RustTetris.entitlements` with `com.apple.developer.game-center`; `project.yml` sets
  `CODE_SIGN_ENTITLEMENTS` and links `GameKit`.
- `Native.m`: `rt_gc_authenticate()` at launch (if GameKit hands back a sign-in view controller,
  present it once from the window's root view controller; signed out or offline means the game
  just carries on), `rt_gc_submit_score(int64)` at game over, `rt_gc_show_leaderboard()` opens
  `GKGameCenterViewController` on the High Score board, `rt_gc_available()`.
- The touch leaderboard gets a **GLOBAL** button at its bottom (iOS, signed in); a tap on it opens
  Game Center, a tap anywhere else still goes back.
- Rust: `src/game_center.rs` wrappers (no-ops off iOS). Score submitted at every game over
  (Game Center keeps the best). The leaderboard menu gets a **Global** option on iOS when signed
  in; the local leaderboard stays as it is.

## Scoring fixes this needs (found while building)

- Combos never ended: a lock without a clear wasn't scored, so `process_no_line_clear` never ran.
  `lock_current_piece` now scores it (`add_score_for_lines(0)`).
- T-spins were never detected: `is_t_spin()` looks at `current_piece`, which is gone by the time
  the clear is scored. It's now worked out at the lock (`lock_was_t_spin`); a hard drop that moves
  the piece also clears `last_action_was_rotation`. Tested in `src/game/t_spin_tests.rs`.
- Both change scores: combo bonuses stop piling up, T-spins start paying.

## Order and coordination

One commit series per part on `tjonestj3/ios-polish`, in the order above (parts 1 and 4 both
touch `Native.m` and `project.yml`); split into PRs when pushed. Device testing happens through a
TestFlight build of the branch, started only after asking.
Files touched: `src/app.rs`, `src/app/*`, `src/graphics/view.rs`, `src/scoring/mod.rs`,
`src/menu/mod.rs`, `src/game/state.rs` (`lock_current_piece`, `hard_drop`, `add_score_for_lines`,
one field), new `src/game/t_spin_tests.rs`, `src/haptics.rs`, `src/game_center.rs`, `ios/*`,
`.github/workflows/*`. The fix-failing-tests session has been told and agrees there's no overlap.

## Left to the builder

Haptic pattern timings and intensities (tuned on the phone), stats panel typography and row
order, callout styling and fade, countdown look, exact cron time.
