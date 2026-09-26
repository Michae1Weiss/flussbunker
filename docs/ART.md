# Art

Everything is drawn by hand in Procreate on two canvases: one **sheet** with every sprite, one
**map**. `tools/art_fit.py` makes the guides, cuts the drawings and writes `assets/`. The art layer
(`src/game/art.rs`) swaps a greybox rectangle for a drawing the moment its file exists; missing
files stay rectangles, so the set can grow piece by piece.

Style, from `docs/VISION.md`: pencil, liner and one red on paper. Sprites are paper cutouts laid
on the map; craters, splashes and the build-spot circle are drawn on the paper.

## Hard rules

Each rule exists because breaking it is the usual way hand-drawn game art ends up looking amateur.

1. **One canvas, one scale.** Every sprite is drawn on the sheet at 16 px per world unit, with the
   same brushes. Sprites drawn one by one at different sizes end up with different line weights
   after scaling, and the set stops looking like one hand made it.
2. **Line weights are fixed**, shown as the legend on the sheet: silhouette outline 1.2 units
   (19 px on the sheet), detail 0.6 units (10 px), nothing thinner. At 1280x800 one unit is 0.83
   screen pixels; anything finer turns into grey noise. Hatching: few, wide strokes.
3. **Draw inside the dark frame, fill it.** The light outer frame is the exported image; the band
   between the frames is where the script adds the cut edge and shadow. A hull that fills only 70 %
   of the dark frame is 30 % smaller than its hitbox in the game.
4. **Straight top-down, bow up, centre is the pivot.** No three-quarter view, no ground shadow, no
   perspective. The game rotates ships, boats and torpedoes; the cell's crosshair is the rotation
   point. Turrets never rotate: barrel up.
5. **Silhouette before detail.** Classes differ in *shape*, not only size: MTB narrow and pointed,
   corvette one funnel, landing craft a blunt box, destroyer two funnels and guns fore and aft,
   monitor one huge turret, gold transport white hull with crates. Size depends on screen and
   distance; shape does not.
6. **Red belongs to the player.** Boat, emplacements, torpedoes and the bunker carry the one red;
   enemies stay black and grey. The cheapest readability there is.
7. **Value hierarchy: sprite ink is the darkest thing on screen, water the darkest area of the
   map, land the lightest.** Sprites are white paper cutouts with black ink, so they need a toned
   ground to stand on: the water is a flat, calm wash, clearly darker than paper and clearly
   lighter than ink. Banks are thin grey pencil lines, never solid black. If the map competes with
   the ships, the map changes. Exact colours: "The map", below.
8. **Draw in full contrast, never pre-tinted.** The game tints at runtime: submerged boat
   blue-grey, hits flash red, sinking wrecks grey.
9. **Leave the inside empty or draw it.** The script fills everything inside the outline with paper,
   so a closed outline is enough; draw only what should show on top.
10. **Squint test for every sprite:** zoom Procreate out until the cell is the size of the game
    (the table below, at 1920x1080). If you cannot name the class, redraw before drawing the next.

## Files and sizes

World units, 1 tile = 48. "On screen" is at 1920x1080 (1.25 px per unit); at 1280x800 multiply by
two thirds. The script exports at 3 px per unit: a Retina MacBook shows about 2.3, and linear
filtering stays clean up to about 2x reduction.

| File | World (w x h) | On screen | Notes |
|---|---|---|---|
| `sprites/ships/mtb.png` | 16 x 29 | 20 x 36 | fast, small, light |
| `sprites/ships/corvette.png` | 24 x 43 | 30 x 54 | one funnel |
| `sprites/ships/landing.png` | 30 x 54 | 37 x 68 | blunt, tough |
| `sprites/ships/destroyer.png` | 32 x 58 | 40 x 72 | the one that shoots back |
| `sprites/ships/monitor.png` | 40 x 72 | 50 x 90 | capital, one huge turret |
| `sprites/ships/gold.png` | 28 x 50 | 34 x 62 | prologue transport: rich, not civilian |
| `sprites/boats/viia.png` | 22 x 54 | 28 x 68 | |
| `sprites/boats/viic.png` | 22 x 61 | 28 x 76 | |
| `sprites/boats/ixc.png` | 22 x 68 | 28 x 85 | |
| `sprites/turrets/{flak,gun88,heavy15}_{0,1,2}.png` | 36 / 43 / 50 square | 45 / 54 / 63 | flak tier 2 two barrels, 88 tier 2 twin, 15 cm one, two, three barrels |
| `sprites/bunker.png` | 77 square | 96 | |
| `sprites/spot.png` | 48 square | 60 | flat: a pencil circle, "build here" |
| `sprites/splash.png` | 48 square | scaled per use | flat, centred, seen from above: a ring burst, not a fountain from the side |
| `sprites/crater.png` | 22 square | scaled per use | flat |
| `sprites/debris_0..5.png` | 14, 6, 7, 8, 9, 10 square | ~1.6x | turret, funnel, lifeboat, crate, anchor, hatch |
| `sprites/torpedo.png` | 5 x 14 | 6 x 18 | flat: runs under water |
| `sprites/shell.png` | 4 x 4 | 5 | flat |
| `map/background.png` | 1632 x 960 with bleed | the window | drawn 6528 x 3840, stored 3264 x 1920 |
| `fonts/pencil.ttf` | | | an OFL handwriting font, e.g. Patrick Hand or Kalam |

The ship sizes live in `src/balance.rs` (`EnemyKind::stats`, `UBOAT_*`, `TOWER_SIZE`,
`HULL_BEAM_RATIO`) and are mirrored in `tools/art_fit.py`; change both, then make a new sheet.

## Sprites: the sheet

```bash
python3 tools/art_fit.py sheet art/guides/sheet.png
```

1. AirDrop or iCloud the PNG to the iPad. Procreate gallery, **Import**: it opens as a canvas of
   exactly its size (4096 x 5686). Never Insert it into another canvas; that rescales it.
2. The imported layer is the guide: opacity ~40 %, lock it. Above it: `ink`, `red`, as needed.
   Colours: ink near-black, red `#c8372d`.
3. Brushes: one pencil (Sketching, 6B) and one liner (Inking, Studio Pen). Match their sizes to
   the legend at the bottom by drawing a test stroke next to it; never change them afterwards.
4. Symmetric hulls: Actions, Canvas, Drawing Guide, Symmetry, vertical; drag the axis onto the
   cell's centre line and turn on Drawing Assist for the ink layer.
5. Export: hide the guide layer and the background colour, Share, PNG. Save it as
   `art/drawn/sheet.png` (commit it: re-slicing after a script change needs no redrawing).
6. Cut it:

```bash
python3 tools/art_fit.py slice art/drawn/sheet.png
cargo run
```

Empty cells are skipped, so start with the style test: MTB, destroyer, monitor, VIIC, 8.8 tier 1.
If the classes are not distinguishable at 1280x800, change the drawing, not the game
(`docs/ROADMAP.md`, M8).

## The second sheet: barrels, effects, paper UI

```bash
python3 tools/art_fit.py sheet --set fx art/guides/sheet_fx.png
python3 tools/art_fit.py slice --set fx art/drawn/sheet_fx.png
```

Its own layout, so the first sheet stays valid. Same brushes, same legend, same export steps.

- **Barrels** `sprites/barrels/{flak,gun88,heavy15}_{0,1,2}.png`: the cell is the emplacement's
  square and its centre is the pivot the barrel turns around. Draw the barrel from the centre
  upward; the drawn base shows faintly as a guide. The bases on the first sheet are redrawn
  **without** barrels, in the same cells: flak 2 and 8.8 tier 2 have two barrels on one mount,
  the 15 cm one, two, three.
- **Effects** `fx/flash_0..1` (muzzle flash, small then big), `fx/explosion_0..3` (flash,
  fireball, breaking up, smoke; played in that order), `fx/smoke`: seen from above, flat, no
  cutout edge. The game scales them per weapon and ship size.
- **Paper UI**: `ui/note` (the HUD notes), `ui/tag` (small labels on the map), `ui/banner` (the
  big phase pop-ups) are 9-slice: the thin guide lines cut the cell into corners, edges and middle.
  Corners are kept as drawn, edges are stretched along one axis, the middle in both. So: torn or
  folded detail only in the corners, even edges, a plain middle. `ui/disc` is a round paper
  plate for radial-menu entries.
- **Icons** `ui/icons/*`: coin, bunker, torpedo, hull, dive, surface, the four targeting modes
  (first, strongest, weakest, nearest), upgrade, sell. Ink on paper, flat, readable at 24 px.
- **Stroke** `ui/stroke`: one pencil line along the middle of a long strip, its left and right
  ends joining seamlessly. The game lays it around every range circle and along every line, at
  the same width for any radius, and exactly as thick as drawn: draw the line as heavy as it
  should look.

Paper UI and icons are exported at 8 px per world unit instead of 3: the UI grows with the window.

## The third sheet: hints and cursors

```bash
python3 tools/art_fit.py sheet --set hints art/guides/sheet_hints.png
python3 tools/art_fit.py slice --set hints art/drawn/sheet_hints.png
```

The pictures that teach without words (`docs/design/onboarding.md`) and the mouse cursors.

| File | World units | Exported | Style | What |
|---|---|---|---|---|
| `ui/hints/mouse_0` | 32 x 40 | 256 x 320 | sticker | a mouse, left button plain |
| `ui/hints/mouse_1` | 32 x 40 | 256 x 320 | sticker | the same mouse, left button filled red, two or three short click strokes |
| `ui/hints/space_0` | 72 x 24 | 576 x 192 | sticker | the Space key up, its side visible; no lettering |
| `ui/hints/space_1` | 72 x 24 | 576 x 192 | sticker | the same key pressed: the cap lower, the side thinner |
| `ui/cursors/pencil` | 12 x 12 | 48 x 48 | flat | the default cursor; tip at the top left |
| `ui/cursors/steer` | 12 x 12 | 48 x 48 | flat | over the water; a steering mark, its centre is the click point |
| `ui/cursors/build` | 12 x 12 | 48 x 48 | flat | over an empty build spot; a hammer, head at the top left |
| `ui/cursors/hand` | 12 x 12 | 48 x 48 | flat | over anything that opens or presses; fingertip at the top left |

- Two frames of one drawing: draw frame 0, trace it for frame 1 and change only the pressed
  part, or the picture jumps instead of clicking.
- Cursors sit on paper and on water: ink with a thin white outline, no paper. They are screen
  pixels, not world units, exported at 4 px per unit. The click points are set in
  `src/game/cursor.rs`; move one there if the drawing puts it elsewhere.

## The map

```bash
python3 tools/art_fit.py map-template art/guides/map_template.png
python3 tools/art_fit.py map-mask art/guides/map_mask.png
```

The canvas is 6528 x 3840: 4 px per world unit, the 32 x 18 tile playfield plus one tile of
bleed on every side (the blue frame is the edge of a 16:9 screen; a 16:10 MacBook sees the bleed
top and bottom, so draw paper and land there too). It is 4 px per unit, the sheet is 16: the
legend in the bottom-left corner shows the line weights at this scale. Use it, not the brush
sizes from the sheet.

Colours (hex, sRGB). Land matches the game's paper around the map, so the edge is invisible:

| What | Hex | Why this value |
|---|---|---|
| Land, paper | `#F5F2EB` | same as the game's `PAPER`; stickers (`#F7F5EF`) sit on it by their shadow |
| Water, flat fill | `#AEBDC6` | 1.8:1 against the white stickers, 8:1 against their ink |
| Flow marks on the water | `#8FA2AE` | a step darker than the water, never ink |
| Banks, trees, rocks: lines | `#8A857E` | grey pencil; the darkest thing on the map, still far from ink |
| Grass, small detail | `#B3AEA6` | lighter than the banks |
| Red, rare accents | `#C8372D` | the one red, same as the sprites |

Never pure black (`#000000`) on the map: black belongs to the sprites. In Procreate, type the hex
under Colours, Value. Check Actions, Canvas, Canvas information: the colour profile should be
sRGB; Display P3 shifts every colour when the game loads the PNG.

1. Import the template as a canvas. The blue band is the water exactly as the code has it, the
   circles are build spots, the square is the bunker. Lock it.
2. Actions, Add, Insert a file: the mask. It lands on the band; lock it.
3. A `water` layer above the mask, set to Clipping Mask: fill it flat with `#AEBDC6`, a few flow
   marks in `#8FA2AE`. Nothing busy on the water: ships live there.
4. Banks on a layer above, along the band's edge, grey pencil at most as heavy as the legend's
   bank line. They may bulge outward, never inward.
5. Land: near-white paper, sparse detail. Nothing bigger than the true-size corvette outline in
   the legend corner; a tree is one tile at most. Leave the circles and the square bare: spot
   and bunker are sprites.
6. Hide template and mask, export PNG as `art/drawn/map.png`, then:

```bash
python3 tools/art_fit.py map art/drawn/map.png
python3 tools/art_fit.py overlay assets/map/background.png /tmp/overlay.png
cargo run --example lineup -- full
```

`map` also takes an old 16:9 map without bleed and pads it with paper.

## Checking a change

`cargo run --example lineup -- full` (or `-- 1280 800`) lays every ship, boat and emplacement
tier on the real map with the notes filled and the radial menu open, then saves
`target/lineup/<size>.png` and `-overlay.png` and quits. Look at the 1280 x 800 one first: that is
the M8 kill criterion.

## What the script does to a sprite

Cut the cell, fill the inside of the outline with paper (`#f7f5ef`), add a cut edge of 1 unit
(less on small debris) and a soft shadow half a unit down-right, scale to 3 px per unit, write
`assets/<file>`, update `assets/manifest.txt`. The web build cannot look at the disk and loads
exactly the files in that manifest. Flat sprites (spot, splash, crater, torpedo, shell) are only
cut and scaled.
