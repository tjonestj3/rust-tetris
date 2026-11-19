# Legacy look for the touch panels and pause screen

## Why

In the legacy look, the touch screen draws the board as a green terminal, but the panels beside it
(Next, Hold, score, the pause button) and the pause screen keep the modern glass style. The two
looks clash. In legacy mode the whole game screen should read as one terminal.

## What

Only touch screens, and only in the legacy look. The modern look and the desktop are unchanged.
Layout and tap targets stay as they are.

- **Panels:** black, framed by the board's own borders: `<== NEXT ====>` along the top and
  `<=======>` along the bottom, in the board's green. Labels are dim green, values bright green.
  The score panel is framed as `STATS`.
- **Pieces in Next / Hold:** drawn with the board's block glyph, all green; a held piece that
  can't be swapped yet is dimmed.
- **Level progress:** a text bar, `[#####.......]`.
- **Ghost blocks:** in place of the violet glow, the stats frame and the ghost lines blink
  bright green.
- **Banners** (aiming a ghost block, the first-game gesture hint): black boxes with a thin green
  edge and green text.
- **Buttons** (pause icon; Resume / Legacy / Menu): black with a thin green edge, green label.
- **Pause screen:** a near-black overlay, `PAUSED` in green with a blinking cursor, the gesture
  list in a black card framed like the panels (`<== CONTROLS ==>`).
- **Game over:** the same treatment as the pause screen (it's the same kind of overlay).

## Approaches

1. **Switch the look inside the existing touch drawing** (chosen). The same layout code draws
   either look: frames, colours and pieces are picked by a legacy flag. One layout to keep right,
   and the two looks can't drift apart in where things go.
2. Reuse the desktop's legacy panel functions through a camera. They're laid out for the fixed
   900x750 canvas and don't fit the touch columns or strip.
3. Separate legacy drawing functions for touch. Duplicates all the layout arithmetic.

## Decided

- The pause screen goes green too (asked for), and game over with it (same overlay family).
- Taps, layout and the menus are unchanged.
