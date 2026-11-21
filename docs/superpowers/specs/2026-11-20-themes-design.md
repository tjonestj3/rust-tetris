# Themes and a minimal look (design)

## Why

The game has two looks, the glass "neon" one and Legacy (green terminal). The user wants more,
picked from things they like (Linux, Windows 98, arcade, Apple, Claude, chess), each a complete
look rather than a recolour, plus a minimal mode with a bigger, centred board. Played on iPhone.

## Approaches considered

1. **Colours only**: palette swaps. Cheap, many themes, but they'd all feel like neon in a
   different colour.
2. **Full look** (chosen by the user): each theme brings its own colours, block style, panel
   frames, background and font.
3. **Full look plus sound**: option 2 and each theme's own sound set. Left for later; the theme
   interface leaves room for it.

## Goals

- A theme picker in Settings replaces the Legacy switch. Existing players who had Legacy on start
  in Terminal; everyone else in Neon.
- Each theme lives in one file under `src/graphics/themes/` and is added to one list. Adding a
  theme touches no drawing code.
- A separate **Minimal** switch that works with every theme: no stats panel, small Hold and Next
  beside the board, board as large as the screen allows, centred.
- The themes are tributes, not copies: our own names, no logos, no copied fonts or artwork.
- **Animations follow the theme.** JONES'IN! (Slam and Back-to-back), the line-clear pop, the
  T-spin, combo and perfect-clear labels, the game-over card, the ghost-block comet and the level
  flip all take their colours, font and accents from the theme. The animations are built first
  with every colour coming from one `FxPalette` (`src/app/fx.rs`); a theme supplies its own
  palette, so no animation code changes when a theme is added. Terminal, for example, draws
  JONES'IN! in green block letters with a scanline flash instead of confetti.

## Non-goals

- Per-theme sounds (approach 3), and the animation upgrades from the Animation Lab page (separate
  PRs; they will draw through the theme's colours).
- Desktop. Desktop keeps its two looks, mapped from the theme (Terminal → Legacy board, anything
  else → the current one). Its drawing is a separate 3,800-line path in `src/app.rs`; theming it
  doubles the work for a screen nobody plays on day to day. *(Builder's call; easy to revisit.)*

## The themes

| Name | Inspired by | Board and blocks | Panels and background | Font |
|---|---|---|---|---|
| Neon | today's look | glossy rounded blocks | glass panels, indigo-to-magenta backdrop | ProggyClean |
| Terminal | today's Legacy | `[]` text blocks, green on black | ruled text frames | ProggyClean |
| Arcade | coin-op cabinets | flat pixel blocks with a 1 px highlight | black with scanlines and a slight CRT vignette, neon pixel frames | a chunky pixel face |
| Desktop 98 | Windows 98 | raised bevelled squares, 3D grey edges | grey bevelled "windows" with a blue title bar on a teal desktop | a plain bitmap sans |
| Glass | Apple's frosted look | soft rounded blocks with a light gradient | frosted translucent cards on a soft blurred gradient, light | a rounded sans |
| Warm Paper | Claude's palette | flat blocks in warm muted tones with a paper grain | cream background, thin warm-grey rules, coral accent | a book serif for labels |
| Chess | a wooden chessboard | blocks drawn as wooden tiles; the board is a checkered wood grid | dark walnut frame, brass labels | a classic serif |

Fonts are open-licence faces shipped in `assets/fonts/` (checked before adding each one).

## How it fits the code

- `src/app/touch_view.rs` already draws through a two-way `Look` (`Look::of(game)`,
  `look.pick(modern, legacy)`). That becomes `&Theme`, and each `pick` becomes a named field
  (`theme.label`, `theme.value`, `theme.panel(...)`, ...).
- `Theme` is a struct of colours plus a few function pointers for the parts that differ in
  shape: `block`, `ghost_block`, `panel`, `backdrop`, `board_frame`. Neon's are the current
  `graphics::style` functions; Terminal's are the `term_*` ones.
- `GameSettings.theme: ThemeId` (serde default from `legacy_mode`), `GameSettings.minimal: bool`.
  `Game.legacy_mode` stays for saves and desktop and is set from the theme.
- Minimal: a third `HudStyle` in `src/graphics/view.rs` with its own `PlayLayout` (board plus two
  small side boxes), with the existing layout tests extended to it.
- Settings: the LEGACY switch row becomes THEME (tap to cycle, name shown) and a MINIMAL switch.
  The pause screen's LEGACY button cycles themes too.

## Build order

1. Theme system with Neon and Terminal only, no visible change (the conversion). Screenshot every
   touch screen before and after; they must match.
2. Settings picker and Minimal mode.
3. Arcade and Desktop 98.
4. Glass, Warm Paper and Chess.

Each step is a PR and can go to TestFlight on its own.

## Left to the builder

Exact colours, bevel sizes and font choices within each theme's direction; how the menus pick up
the theme's accent; whether Minimal hides the ghost-block counter or moves it under Next.
