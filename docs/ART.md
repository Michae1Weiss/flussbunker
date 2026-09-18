# Art: what to draw, at what size, and how it gets into the game

The art layer (`src/game/art.rs`) swaps a greybox rectangle for a drawing the moment the file
exists under `assets/`. Nothing else changes: the game scales every image to the greybox size,
so **proportions matter, pixel counts do not**. Export PNG with transparency at the exact aspect
below, drop it in, run. Missing files stay rectangles. Generated images go through
`tools/art_fit.py`, which strips the background, trims and pads to the aspect (`art/README.md`);
the web build loads exactly the files listed in `assets/manifest.txt`, which that script keeps.

Style, from `docs/VISION.md`: pencil, liner and one red on paper. Each sprite is a paper cutout
laid on the map: bake the tiny edge shadow into the PNG. Craters, wrecks and splashes are drawn
*on* the map, not cut out. Readability before detail: the five ship classes must be tellable
apart at 1280x800 in a glance.

## Orientation

Ships, the boat, torpedoes and shells are drawn **bow up** (facing +y); the game rotates them.
Turrets, the bunker, spots, craters, splashes and debris have no orientation.

## Files

Sizes are world units (1 tile = 48). "Export" is what `art_fit.py` writes: 4x, crisp on a 4K screen.

| File | World size (w x h) | Export (px) | Notes |
|---|---|---|---|
| `sprites/ships/mtb.png` | 9 x 16 | 36 x 64 | fast, small, light |
| `sprites/ships/corvette.png` | 13 x 24 | 52 x 96 | medium |
| `sprites/ships/landing.png` | 17 x 30 | 68 x 120 | medium, blunt, tough |
| `sprites/ships/destroyer.png` | 18 x 32 | 72 x 128 | heavy; the one with guns that shoot back |
| `sprites/ships/monitor.png` | 22 x 40 | 88 x 160 | capital; one huge turret |
| `sprites/ships/gold.png` | 15 x 28 | 60 x 112 | prologue transport; must look rich, not civilian |
| `sprites/boats/viia.png` | 12 x 30 | 48 x 120 | the small boat |
| `sprites/boats/viic.png` | 12 x 34 | 48 x 136 | |
| `sprites/boats/ixc.png` | 12 x 38 | 48 x 152 | |
| `sprites/turrets/flak_0..2.png` | 24 / 29 / 34 square | 96 / 116 / 136 | tier 2 has two barrels |
| `sprites/turrets/gun88_0..2.png` | 24 / 29 / 34 square | 96 / 116 / 136 | tier 2 is a twin |
| `sprites/turrets/heavy15_0..2.png` | 24 / 29 / 34 square | 96 / 116 / 136 | one, two, three barrels |
| `sprites/torpedo.png` | 5 x 14 | 20 x 56 | |
| `sprites/shell.png` | 4 x 4 | 16 x 16 | |
| `sprites/debris_0.png` | 14 square | 56 | the turret; flies first and hurts most |
| `sprites/debris_1..5.png` | 6 to 10 square | 24 to 40 | funnel, lifeboat, crate, anchor, hatch |
| `sprites/crater.png` | ~10 to 22 square | 88 | pencil on the canvas, no cutout shadow |
| `sprites/splash.png` | 8 to 48 square | 192 | water column / impact; scaled per use, keep it centred |
| `sprites/bunker.png` | 77 square | 308 | |
| `sprites/spot.png` | 48 square | 192 | a pencil circle: "build here" |
| `map/background.png` | 1536 x 864 | as generated, 16:9 | the whole map: paper, banks, river. Replaces the greybox water |
| `fonts/pencil.ttf` | – | – | a handwriting font for notes and paper HUD |

Ship and boat sprites are tinted at runtime: white means "as drawn", the submerged boat is
tinted blue-grey, hits flash red-ish, wrecks are tinted dark. Draw in full contrast.

## The map

The river centreline is fixed in `src/game/map.rs` (`CENTRELINE_TILES`, tile coordinates from
the top-left, y down) and the water is `RIVER_HALF_WIDTH` (0.9 tiles) either side. The
background must put water exactly there, banks may bulge outward as they like. Build spots
(`BUILD_SPOT_TILES`) are drawn by their own sprite; leave the paper blank underneath.

Coordinates for tracing: a 3072 x 1728 canvas is 32 x 18 tiles, one tile = 96 px.

## Style test first

Before drawing the whole set: one frame at game size with three ships, one turret and the boat,
photographed and scaled to 1280x800. If the classes are not distinguishable at a glance, change
the drawing, not the game. That is the M8 kill criterion in `docs/ROADMAP.md`.
