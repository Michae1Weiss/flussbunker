# Art: what to draw, at what size, and how it gets into the game

The art layer (`src/game/art.rs`) swaps a greybox rectangle for a drawing the moment the file
exists under `assets/`. Nothing else changes: the game scales every image to the greybox size,
so **proportions matter, pixel counts do not**. Draw at roughly 2x the world size given below,
export PNG with transparency, drop it in, run. Missing files stay rectangles.

Style, from `docs/VISION.md`: pencil, liner and one red on paper. Each sprite is a paper cutout
laid on the map: bake the tiny edge shadow into the PNG. Craters, wrecks and splashes are drawn
*on* the map, not cut out. Readability before detail: the five ship classes must be tellable
apart at 1280x800 in a glance.

## Orientation

Ships, the boat, torpedoes and shells are drawn **bow up** (facing +y); the game rotates them.
Turrets, the bunker, spots, craters, splashes and debris have no orientation.

## Files

Sizes are world units (1 tile = 48). "Draw at" is a suggested export size at 2x.

| File | World size (w x h) | Draw at (px) | Notes |
|---|---|---|---|
| `sprites/ships/mtb.png` | 9 x 16 | 18 x 32 | fast, small, light |
| `sprites/ships/corvette.png` | 13 x 24 | 26 x 48 | medium |
| `sprites/ships/landing.png` | 17 x 30 | 34 x 60 | medium, blunt, tough |
| `sprites/ships/destroyer.png` | 18 x 32 | 36 x 64 | heavy; the one with guns that shoot back |
| `sprites/ships/monitor.png` | 22 x 40 | 44 x 80 | capital; one huge turret |
| `sprites/ships/gold.png` | 15 x 28 | 30 x 56 | prologue transport; must look rich, not civilian |
| `sprites/boats/viia.png` | 12 x 30 | 24 x 60 | the small boat |
| `sprites/boats/viic.png` | 12 x 34 | 24 x 68 | |
| `sprites/boats/ixc.png` | 12 x 38 | 24 x 76 | |
| `sprites/turrets/flak_0..2.png` | 24 / 29 / 34 square | 48 / 58 / 68 | tier 2 has two barrels |
| `sprites/turrets/gun88_0..2.png` | 24 / 29 / 34 square | 48 / 58 / 68 | tier 2 is a twin |
| `sprites/turrets/heavy15_0..2.png` | 24 / 29 / 34 square | 48 / 58 / 68 | one, two, three barrels |
| `sprites/torpedo.png` | 5 x 14 | 10 x 28 | |
| `sprites/shell.png` | 4 x 4 | 8 x 8 | |
| `sprites/debris_0.png` | 14 square | 28 | the turret; flies first and hurts most |
| `sprites/debris_1..5.png` | 6 to 10 square | 12 to 20 | funnel, lifeboat, crate, anchor, hatch |
| `sprites/crater.png` | ~10 to 22 square | 44 | pencil on the canvas, no cutout shadow |
| `sprites/splash.png` | 8 to 48 square | 96 | water column / impact; scaled per use, keep it centred |
| `sprites/bunker.png` | 77 square | 154 | |
| `sprites/spot.png` | 48 square | 96 | a pencil circle: "build here" |
| `map/background.png` | 1536 x 864 | 3072 x 1728 | the whole map: paper, banks, river. Replaces the greybox water |
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
