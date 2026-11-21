# Theme fonts

Compiled into the game (`src/graphics/text.rs`, `include_bytes!`), one per theme
(`src/graphics/theme.rs`, `Theme::face`). Kept out of `assets/` so the iOS app doesn't also
copy them in as loose files. All from the Google Fonts repository (github.com/google/fonts):

| File | Font | Licence | Theme |
|---|---|---|---|
| PixelifySans.ttf | Pixelify Sans | SIL OFL 1.1 | Windows 98 |
| NotoSans.ttf | Noto Sans | SIL OFL 1.1 | Mac OS X |
| Ubuntu-Medium.ttf | Ubuntu Medium | Ubuntu Font Licence 1.0 | Linux |
| PressStart2P.ttf | Press Start 2P | SIL OFL 1.1 | Arcade |
| Cinzel.ttf | Cinzel | SIL OFL 1.1 | Chess |
| Lora.ttf | Lora | SIL OFL 1.1 | Warm Paper |
| Inter.ttf | Inter | SIL OFL 1.1 | Glass |

The licence texts are in `licenses/`.
