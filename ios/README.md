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

All gestures, no on-screen buttons (`src/input/touch.rs`):

| Gesture | Does |
|---|---|
| Tap | Rotate clockwise |
| Two-finger tap | Rotate counter-clockwise |
| Drag sideways | Move, a cell per step, following the finger |
| Drag down (and keep it there) | Soft drop |
| Flick down | Hard drop |
| Flick up | Hold |
| **II** (top right) | Pause (then Resume / Menu) |

Menus: tap an option. Settings: tap Sound or Volume to change it, anywhere else to go back. A high
score brings up the keyboard; Return saves it.

The 900x750 game canvas is scaled to fit the screen inside the safe area (`src/graphics/view.rs`),
re-fitted every frame, so any orientation works and the Duo can fold and unfold mid-game. Landscape
gives the biggest board on a regular iPhone.

Try it on desktop: `cargo run -- --touch` (the mouse is a finger), with `--window 390x844` for a
phone-sized window and `--play` to skip the menu.

## Apple setup (one time) ✅ done

1. **App ID**: developer.apple.com → Identifiers → explicit bundle ID `com.tjonestj3.rusttetris`,
   no capabilities.
2. **App record**: App Store Connect → Apps → + → New App with that bundle ID.
3. **TestFlight group**: the app → TestFlight → Internal Testing → +, add yourself, tick
   **Enable automatic distribution**.

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

- **miniquad patch** (`vendor/miniquad`, used through `[patch.crates-io]` in `Cargo.toml`): three
  fixes to miniquad 0.4.8's iOS code, each marked `rust-tetris patch`. Touches report only the
  fingers that changed, with stable ids (upstream lifted every finger when any one lifted). The
  window is made by a UIScene delegate (`QuadSceneDelegate`, named in `App/Info.plist`), which apps
  built with the iOS 27 SDK need in order to launch. And the game is sized from its view rather than
  the whole screen, so rotation and the Duo's fold resize it. Desktop code is untouched.
- **Save files** go in the app's Documents folder (`src/platform.rs`); desktop keeps them in the
  working directory.
- **Music**: the build swaps the 48 kHz background music for a 24 kHz copy inside the app
  (`shrink-music.sh`), halving its size and memory. The repo's file is unchanged.
- **No Quit** on iOS (apps don't quit themselves).

## Maintenance

- TestFlight builds expire 90 days after upload; run the workflow again before then.
