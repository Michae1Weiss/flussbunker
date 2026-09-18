# Art sources

`art/source/` holds what the image models produced, untouched. `assets/` holds what the game loads,
made from here with `tools/art_fit.py`. The contract (files, world sizes, orientation, what is
tinted at runtime) is `docs/ART.md`; this folder is only the way in.

## Pipeline

1. Take a prompt from `prompts.md`. Attach `source/background_1672x941.png` as the style reference.
2. Save the result as `source/<batch>.png` (e.g. `ships_a.png`), never edit it.
3. Cut it into one PNG per drawing:
   `python3 tools/art_fit.py split art/source/ships_a.png art/source/ships_a/`
4. Fit each piece to its key; the script strips the white background, trims, pads to the exact
   aspect and writes the file the game looks for:
   `python3 tools/art_fit.py fit art/source/ships_a/piece_03.png sprites/ships/destroyer.png`
5. `cargo run`. Whatever is fitted replaces its rectangle; `python3 tools/art_fit.py check` lists
   the rest.

The map is the one exception: `fit ... map/background.png --keep-background`, then
`python3 tools/art_fit.py overlay assets/map/background.png /tmp/check.png` and look whether the
water fills the band and the circles sit on land. If the model needs a guide to draw over,
`python3 tools/art_fit.py map-template /tmp/guide.png` makes one.

## What the script cannot fix

- **Wrong view.** A three-quarter view stays a three-quarter view. Ask again.
- **Wrong orientation.** Bow must point up. Rotate in Preview before fitting, or ask again.
- **A drawing far from its aspect.** `fit` pads with transparency and warns; the ship then
  renders smaller than its hitbox. Redraw or accept.
- **Style drift between batches.** Always attach the same reference; regenerate the odd one out
  rather than the rest.
