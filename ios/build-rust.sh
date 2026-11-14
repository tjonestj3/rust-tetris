#!/bin/bash
# Builds the game's Rust code (ios/rust, which pulls in the whole game) as the static library the
# Xcode project links: target/<rust target>/release/librust_tetris_ios.a.
#
# Xcode runs this as the first build phase; the workflow also runs it on its own first, so a Rust
# error shows up as its own step. Run it by hand on a Mac the same way: ios/build-rust.sh
set -euo pipefail

repo="$(cd "$(dirname "$0")/.." && pwd)"

# Device unless Xcode says it's building for the simulator.
case "${PLATFORM_NAME:-iphoneos}" in
  iphonesimulator) target=aarch64-apple-ios-sim ;;
  *) target=aarch64-apple-ios ;;
esac
deployment="${IPHONEOS_DEPLOYMENT_TARGET:-16.0}"

# Find rustup and cargo wherever they are installed: ~/.cargo/bin, Homebrew's keg-only rustup (no
# cargo on PATH, only rustup), or a runner's own location. Xcode's build phases don't see the
# login PATH, so look in the usual places too.
export PATH="$PATH:$HOME/.cargo/bin:/opt/homebrew/opt/rustup/bin:/opt/homebrew/bin:/usr/local/bin"
rustup target add "$target" >/dev/null
cargo="$(command -v cargo || rustup which cargo)"
toolchain_bin="$(dirname "$(rustup which rustc 2>/dev/null || command -v rustc)")"

# Xcode's build environment (SDKROOT pointing at the iOS SDK and the like) breaks cargo's build
# scripts, which run on the Mac, so cargo gets a clean one.
clean_path="$(dirname "$cargo"):$toolchain_bin:$(dirname "$(command -v rustup)"):/usr/bin:/bin"
env -i HOME="$HOME" PATH="$clean_path" IPHONEOS_DEPLOYMENT_TARGET="$deployment" \
  ${RUSTUP_HOME:+RUSTUP_HOME="$RUSTUP_HOME"} ${CARGO_HOME:+CARGO_HOME="$CARGO_HOME"} \
  "$cargo" build --manifest-path "$repo/Cargo.toml" -p rust-tetris-ios --release --target "$target"

lib="$repo/target/$target/release/librust_tetris_ios.a"
test -s "$lib"
echo "Built $lib"
