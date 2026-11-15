# Touch layout: use the whole screen

## Why

The first TestFlight build drew the fixed 900x750 desktop canvas scaled to fit the screen. On the
folded iPhone Duo that left big empty bands above and below and a small board; open, a small board
with desktop clutter beside it (logo, gesture list, debug stats). The menu came up small and
off-centre, and its emoji showed as empty boxes. Goal: a board as big as the screen allows, a
clean, polished HUD, no empty bands.

## Goals / non-goals

- Goals: the board fills the height (or width on tall phones) in every orientation and fold
  state; Next, Hold and the score in compact panels; the backdrop edge to edge; large, centred
  menus; sharp text.
- Non-goals: changing desktop play; changing the game itself (a next-piece queue, ghost-block
  controls by touch).
- Anti-goals: on-screen control buttons (gestures only, plus pause); desktop-only text on touch
  screens.

## Approaches

1. **Lay out from the real screen on touch devices; reuse the existing drawing through cameras**
   (picked). The board drawing maps the canvas board rect onto the new board rect; panels are new.
   Small, low-risk, desktop untouched.
2. Rewrite all drawing in screen coordinates. Cleaner in the end, but a large rewrite of
   3,500 lines of drawing for the same result.
3. Keep the fixed canvas but crop it. Still wastes space and keeps the clutter.

## Design

- `PlayLayout` (graphics/view.rs) picks one column, two columns, or a top strip, whichever gives
  the biggest whole-pixel cell; two columns when they cost almost nothing (balanced).
- `app/touch_view.rs` draws the touch game screen: backdrop, board via the board camera, panels,
  first-game gesture hint (shown once, remembered in settings), pause screen with the gesture list.
- Menus: each menu screen reports the canvas area it uses (`content_bounds`); only that is fitted
  to the screen. Touch-only tweaks: compact settings, a three-column leaderboard, name entry above
  the keyboard. Emoji replaced with plain text.
- `graphics/text.rs`: draw text rasterized at its on-screen size.
- miniquad (iOS): render at the native scale; keep the view the size of its window (the launch-time
  375x667 compatibility size caused the off-centre menu); safe area from the game's own view.

## Left to the builder

Panel proportions, colours and type sizes, tuned against desktop screenshots at Duo and phone sizes.
