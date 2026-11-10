# SimpleGrid app icon (PLACEHOLDER)

**This icon is temporary.** Every Crafting App's icon is an engraved creature portrait that the
product owner makes; SimpleGrid's hasn't been made yet.
Until it is, this placeholder stands in: a simple ledger page with a header row, a row-number
column, a selected cell with its fill handle and a double rule under a total, on a full-bleed
SimpleGrid-green tile. It is original geometry written by hand for this repository, deliberately
not modelled on any vendor's spreadsheet logo (no letters, no "X", no overlapping panels).

When the owner's art arrives, replace `simplegrid.svg`, run `packaging/icons.sh`, update this
README (provenance, creature) and the rows in `ATTRIBUTION.md`, and drop the word placeholder.

## Palette

Exactly three colours:

| Colour | Hex | Used for |
|---|---|---|
| Ink | `#0b0b0c` | Line work and contour |
| Paper | `#efe9dc` | The page |
| SimpleGrid green (app colour, **proposed**) | `#1f9d55` | The full-bleed field and the header cells |

The app colour is a proposal pending the owner's approval: SimpleGrid green `#1f9d55`, with ink
(darker) variant `#147a40` for text and accents on light backgrounds. It differs from
DesignCraft's `#7bb51c`.

## Geometry

A 512 × 512 viewBox, clipped to a rounded square with `rx=112`. Windows and Linux use the full
tile. The macOS icons (`.icns`, `simplegrid-macos-512.png`) sit on Apple's grid: an 824/1024
body with a transparent margin.

## Files

| File | Use |
|---|---|
| `simplegrid.svg` | Canonical artwork; also `hicolor/scalable` |
| `simplegrid-1024.png` | 1024 px render of the full tile |
| `simplegrid-macos-512.png` | Runtime Dock icon on macOS (for `apps/simplegrid` to embed) |
| `simplegrid.icns` | macOS bundle icon (`CFBundleIconFile`) |
| `simplegrid.ico` | Windows exe icon, 16–256 px (embedded by `apps/simplegrid/build.rs`) |
| `hicolor/<size>/apps/simplegrid.png` | Linux icon theme, 16–512 px; the 256 px one is also the runtime window icon on Windows and Linux |
| `hicolor/scalable/apps/simplegrid.svg` | Linux scalable icon |
| `LICENSE.txt` | MIT OR Apache-2.0 |

The app ID is `simplegrid`. It's used for the hicolor icon names, the Wayland app
ID and `packaging/linux/simplegrid.desktop`.

## Regenerate

Edit `simplegrid.svg`, then run `packaging/icons.sh`. It needs `resvg`, and `iconutil` for the
`.icns` (macOS only). It packs the `.ico` with `cargo xtask ico`.
