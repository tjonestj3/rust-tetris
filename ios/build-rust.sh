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

# Xcode's build environment (SDKROOT pointing at the iOS SDK and the like) breaks cargo's build
# scripts, which run on the Mac, so cargo gets a clean one.
cargo_path="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin"
env -i HOME="$HOME" PATH="$cargo_path" IPHONEOS_DEPLOYMENT_TARGET="$deployment" \
  bash -c "cd '$repo' && rustup target add '$target' >/dev/null && cargo build -p rust-tetris-ios --release --target '$target'"

lib="$repo/target/$target/release/librust_tetris_ios.a"
test -s "$lib"
echo "Built $lib"
