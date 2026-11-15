# SysVane logo guidelines

**Mark:** the Crate, an isometric cube of three faces and a lit lid. The gaps between the faces form a Y whose upper arms echo a V.
**Wordmark:** `SysVane`, written exactly that way. Inter SemiBold, converted to outlines, tracking -1.2%, HarfBuzz kerning. Do not retype it.

## Colour

| Name | HEX | RGB | Use |
|---|---|---|---|
| Ink | `#171717` | 23 23 23 | Side faces, wordmark, app-icon tile |
| Emerald | `#3ecf8e` | 62 207 142 | The lid only. The one colour event, as in `DESIGN.md` |
| White | `#ffffff` | 255 255 255 | Side faces and wordmark on dark |

Screen use only. No CMYK or Pantone values are set.
The lid on white is about 2.0:1, so the ink faces carry the shape; never rely on the lid alone.

## Files

| Need | File |
|---|---|
| Light background | `sysvane-symbol.svg`, `sysvane-horizontal.svg`, `sysvane-stacked.svg` |
| Dark background | `*-reversed.svg` |
| One colour | `variants/*-black.svg`, `*-white.svg`, `*-mono-3ecf8e.svg` |
| 48 px and below | `sysvane-symbol-small.svg` (wider gaps), `sysvane-symbol-small-reversed.svg` |
| App icon | `sysvane-app-icon.svg`, `sysvane-app-icon-small.svg` (16 to 48 px) |
| Windows / Tauri | `desktop/src-tauri/icons/` (`icon.ico`, `icon.png`, `32x32.png`, `128x128*.png`, `Square*Logo.png`, `StoreLogo.png`) |
| Tray | `core/assets/tray-32.rgba` (raw 32 px RGBA of the app icon) |
| Web | `icons/favicon.svg`, `favicon.ico`, `apple-touch-icon.png`, `icon-192.png` |
| Wordmark only | `sysvane-wordmark.svg` |

Tray icon: use the app icon (ink tile), because the ink faces disappear on a dark taskbar.
The SVG viewBoxes are tight to the artwork; they are not the clear space.

## Clear space and minimum size

- Clear space on every side: one quarter of the symbol height.
- Symbol: 16 px minimum. Use the small cut up to 48 px.
- Horizontal lockup: 24 px high minimum. Stacked lockup: 48 px high minimum. Below that, use the symbol alone.
- The side gap between symbol and wordmark in the horizontal lockup is fixed. Do not rescale the parts separately.

## Backgrounds

- Light or white: full colour. Dark or near-black: reversed.
- Photos, mid-tone or coloured backgrounds: one-colour black or white.
- The lid is the only part that may be emerald. No other element takes the accent.

## Do not

- Recolour the side faces, or colour more than the lid.
- Close the gaps, add outlines, shadows, glows or gradients.
- Rotate, skew or change the cube's angle.
- Put the full-colour mark on a dark background (the faces vanish).
- Add the wordmark in another font, or set it in lowercase or all caps.
- Use the mark as the V of a longer word.

## Handover notes

- Wordmark font is Inter (SIL OFL 1.1), already a project dependency. Outlines are allowed under that licence.
- Not done: trademark search, CMYK/Pantone, a new `.icns` for macOS (`desktop/src-tauri/icons/icon.icns` is still the old template icon), custom letterforms, motion.
- Sources are generated from geometry (three faces on a 256 grid, 60 degree edges, face gap 10, reversed 12, small cut 20, small reversed 22). Rebuild instead of hand-editing:
  - `uv run --no-project --with fonttools --with brotli --with uharfbuzz python branding/logo/scripts/build_kit.py` (needs `npm install` in `desktop/` for the Inter file)
  - `uv run --no-project python branding/logo/scripts/build_icons.py <logo-design skill>/scripts` (renders through that skill's `render_png.py`; it needs Chrome, rsvg, Inkscape or cairosvg)
- `variants/`, `test/` and `board/` come from the skill's `export_variants.py`, `preview_sheet.py` and `presentation_board.py` (spec: `board-spec.json`); they are not rebuilt by the scripts above.
- The sidebar logo in `desktop/src/components/layout.tsx` (`Logo`) holds a copy of `sysvane-symbol-small.svg`'s paths. Update it when the kit changes.

