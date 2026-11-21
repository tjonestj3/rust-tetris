# Rust Tetris for iPhone

The game's Rust code builds as a static library (`ios/rust`, which calls `rust_tetris::app::run`),
a small Xcode project wraps it (`ios/project.yml`, generated with XcodeGen; `App/main.m` just calls
into Rust), and fastlane signs it and uploads it to TestFlight. Built by
`.github/workflows/ios.yml` on GitHub's `xcode-27` runner (Xcode 27.1 and its iOS 27.1 SDK, so the
game resizes on iPhone Duo instead of letterboxing; the repository variable `IOS_RUNNER` overrides
the label). It runs only by hand: Actions → iOS app → Run workflow, or
`gh workflow run ios.yml --ref <branch>` (either needs the workflow on `main`).

`.github/workflows/ci.yml` checks on Linux, on every PR, that the iOS code still compiles for the
iPhone (`cargo check -p rust-tetris-ios --target aarch64-apple-ios`), so most mistakes show up
without waiting for a Mac.

## Playing by touch

All gestures (`src/input/touch.rs`); the only button is pause:

| Gesture | Does |
|---|---|
| Tap | Rotate clockwise |
| Two-finger tap | Rotate counter-clockwise |
| Drag sideways | Move, a cell per step, following the finger |
| Drag down (and keep it there) | Soft drop |
| Flick down | Hard drop |
| Flick up | Hold |
| **II** (beside the board) | Pause: the gestures, Resume, Menu |

**Game Center** (`src/game_center.rs`, `App/GameCenter.m`): the game signs in at launch (GameKit
offers its sheet once a launch at most), sends the score at the end of every game to the
`rusttetris.highscore` leaderboard, and, when signed in, the touch leaderboard has a **GAME
CENTER** button that opens the global board. The phone's own high-score table is unchanged.

Menus: tap an option. Settings: tap Sound, Volume, Legacy or Haptics to change it, anywhere else
to go back. A high score brings up the keyboard; Return saves it.

### Haptics

`src/haptics.rs` picks one haptic a frame (the strongest asked for) and `App/Haptics.m` plays it
with Core Haptics (UIKit impact feedback on phones without it):

| Event | Feel |
|---|---|
| Hard drop | a sharp tap |
| 1-3 lines | a tap per line, rising |
| Combo | a quick run of rising taps, longer with the combo |
| Tetris | a heavy double with a rumble between |
| T-spin | three sharp ticks |
| Perfect clear | a rumble and three rising taps |
| Ghost-block fireball | a soft whoosh on the throw, a thud on impact |
| Game over | a low rumble |

Clears are felt when the lines flash; a perfect clear when they're gone. Off in Settings.

The first game shows a short gesture hint over the board for ten seconds.

### Layout

Laid out from the real screen every frame (`src/graphics/view.rs`), so the phone can rotate and the
Duo can fold and unfold mid-game:

- **The board** gets the full height of the safe area, or the full width on a tall phone, at whole
  pixels per cell. Next, Hold and the score go in compact panels in **one side column** (Duo
  folded), **two side columns** (Duo open, landscape), or **a strip along the top** (tall phones),
  whichever gives the bigger board. The panels and pause screen are `src/app/touch_view.rs`; the
  board and its effects are the desktop drawing, through a camera that maps the canvas's board onto
  the new board rect.
- **Open Duo extras**: the right column's space under Next (`PlayLayout::feed`) shows this game's
  tallies (pieces, pieces per second, Tetrises, T-spins, best combo, best back-to-back run) and,
  for a moment after a special clear, a callout (`T-SPIN DOUBLE`, `BACK-TO-BACK TETRIS`,
  `COMBO 3`). The tallies are saved with the game.
- **Folding mid-game pauses** (so does unfolding or turning the phone: the board jumps). Unfolded
  or sideways (two columns), it counts 3-2-1 back into play by itself; folded, it waits for
  Resume, which counts 3-2-1 too (`src/app/duo.rs`).
- **Menus** fit just the part of the canvas they use (`MenuSystem::content_bounds`) to the screen,
  so they come out large; the backdrop covers the whole screen. The leaderboard shows rank, name and
  score, and name entry sits in the top half, above the keyboard.
- **Text** is rasterized at its on-screen size (`src/graphics/text.rs`), so it's sharp at any scale.

Desktop play is unchanged. Desktop flags for checking the touch screens:

```sh
cargo run -- --touch --window 458x667          # mouse = finger; Duo-folded-ish size
TETRIS_INSETS=59,0,34,0 cargo run -- --touch   # fake safe-area insets (top,left,bottom,right)
cargo run -- --touch --play [--paused]         # straight into a game (paused)
cargo run -- --touch --play --paused --countdown --callout   # the 3-2-1 and a clear callout
cargo run -- --touch --menu settings           # or leaderboard, name
cargo run -- --touch --menu leaderboard --game-center   # as if signed in to Game Center
TETRIS_SCREENSHOT=out.png cargo run -- ...     # save frame 30 and quit
```

## Apple setup (one time) ✅ done

1. **App ID**: developer.apple.com → Identifiers → explicit bundle ID `com.tjonestj3.rusttetris`,
   no capabilities.
2. **App record**: App Store Connect → Apps → + → New App with that bundle ID.
3. **TestFlight group**: the app → TestFlight → Internal Testing → +, add yourself, tick
   **Enable automatic distribution**.

### Game Center (one time, by hand) ⏳ to do

4. **Capability**: Identifiers → `com.tjonestj3.rusttetris` → make sure **Game Center** is ticked
   (it usually is by default) and save. The next build's provisioning profile picks it up
   (`get_provisioning_profile(force: true)` in the Fastfile).
5. **Leaderboard**: App Store Connect → the app → **Game Center** (under Features or Services)
   → Leaderboards → + → **Classic**: reference name `High Score`, ID **`rusttetris.highscore`**,
   score format Integer, sorted **High to Low**, submission type Best Score; add an English
   localization (name `High Score`).
6. TestFlight builds use Game Center's sandbox: sign in on the phone when the game offers it.

Until then the game plays as before: scores just don't reach Game Center.

## Repository secrets (`gh secret set -R tjonestj3/rust-tetris …`)

The same six as Spell Duel; the distribution certificate and the App Store Connect API key are the
team's, shared by every app.

| Secret | Value |
|---|---|
| `ASC_KEY_ID` | the API key's ID |
| `ASC_ISSUER_ID` | its issuer ID |
| `ASC_API_KEY_P8` | base64 of the `.p8` |
| `APPLE_TEAM_ID` | the team ID |
| `IOS_DIST_CERT_P12` | base64 of the Apple Distribution `.p12` |
| `IOS_DIST_CERT_PASSWORD` | the `.p12`'s password |

Build the `.p12` with legacy encryption, or `security import` on the runner rejects it:
`openssl pkcs12 -export -legacy -inkey dist.key -in distribution.pem -out dist.p12`

Variables (optional): `IOS_BUNDLE_IDENTIFIER` (if the App ID isn't `com.tjonestj3.rusttetris`),
`IOS_RUNNER`.

## On a Mac (optional)

```sh
ios/build-rust.sh                 # the Rust library (Xcode also runs this itself)
cd ios && xcodegen && open RustTetris.xcodeproj
```

Pick a team under Signing & Capabilities and run on a plugged-in iPhone. The simulator works too
(`rustup target add aarch64-apple-ios-sim`; the build picks it).

## How it fits together

- **miniquad patch** (`vendor/miniquad`, used through `[patch.crates-io]` in `Cargo.toml`): fixes
  to miniquad 0.4.8's iOS code, each marked `rust-tetris patch`:
  - Touches report only the fingers that changed, with stable ids (upstream lifted every finger
    when any one lifted).
  - The window is made by a UIScene delegate (`QuadSceneDelegate`, named in `App/Info.plist`),
    which apps built with the iOS 27 SDK need in order to launch.
  - The game is sized from its view, and the view is kept the size of its window: it's created at
    `UIScreen.mainScreen`'s size, which on the Duo is a 375x667 compatibility size at launch (the
    first build's menu came up small and off-centre until a fold).
  - It renders at the screen's native scale (3x) instead of a fixed 2x, so it's pixel-sharp.
  - The window is kept the size of its scene, and each frame is drawn at the GL drawable's real
    size, so a stale size can't push the picture into a corner.

Touch settings show the screen's measurements in small print at the bottom (game size, UIKit's
screen / window / scene / view sizes, the drawable, the scale and the safe area): a screenshot of
it is enough to sort out a layout problem on a device.

  Desktop code is untouched.
- **Save files** go in the app's Documents folder (`src/platform.rs`); desktop keeps them in the
  working directory.
- **Music**: the build swaps the 48 kHz background music for a 24 kHz copy inside the app
  (`shrink-music.sh`), halving its size and memory. The repo's file is unchanged.
- **No Quit** on iOS (apps don't quit themselves).
- **Objective-C** (`App/*.m`, ARC): the bits that are much simpler in Objective-C than through
  Rust's `objc` crate (Core Haptics patterns, GameKit's blocks and view controllers). Rust declares their C functions for
  `target_os = "ios"` only; desktop builds get no-ops. Linux CI can't compile them: the first
  check is the Mac build in `ios.yml`.

## Maintenance

- TestFlight builds expire 90 days after upload. `.github/workflows/testflight-refresh.yml` checks
  every Monday on Linux (fastlane's `testflight_age` lane asks App Store Connect) and starts
  `ios.yml` on `main` once the newest build is 75 days old; run it by hand with **force** to
  rebuild now. GitHub turns scheduled workflows off after 60 days without a commit and emails
  about it: turn it back on under Actions → TestFlight refresh.
