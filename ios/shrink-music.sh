#!/bin/bash
# Xcode build phase, after the resources are copied: swaps the background music in the built app
# for a 24 kHz copy. The original (48 kHz, 6 minutes) is ~71 MB on disk and ~140 MB in memory once
# decoded; this halves both and sounds the same through a phone speaker. Desktop keeps the original.
set -euo pipefail

music="$TARGET_BUILD_DIR/$UNLOCALIZED_RESOURCES_FOLDER_PATH/assets/sounds/tetris-background-music.wav"
if [ ! -f "$music" ]; then
  echo "warning: no background music at $music"
  exit 0
fi
tmp="$TARGET_TEMP_DIR/music-24k.wav"
afconvert -f WAVE -d LEI16@24000 "$music" "$tmp"
mv "$tmp" "$music"
ls -la "$music"
