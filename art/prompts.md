# Prompts

Every prompt starts with the same style block and attaches `source/background_1672x941.png` as
reference. One batch per image; the pieces get cut apart afterwards (`README.md`). Batches are
small on purpose: models keep a style within one image and drift between images, and they miscount
above about six items.

## Style block

> Hand-drawn ink illustration in the exact style of the attached map: black fineliner and pencil
> hatching, one accent colour (a slightly desaturated red), no other colours, no gradients, no
> shading with grey fills. Drawn strictly from directly above (orthographic top-down, like a map
> symbol), not from an angle. Plain pure white background, no paper texture, no ground shadow, no
> text, no labels, no frame. Bold, simple silhouettes that stay readable when shrunk to 40 pixels.

## Batch A: enemy ships (6)

> [style block] A single row of six warships seen from directly above, bows pointing to the top of
> the image, generous white space between them, from smallest to largest:
> 1. a tiny fast motor torpedo boat, very narrow, open cockpit, two torpedo tubes along the sides,
>    length about 1.8 times its width;
> 2. a small corvette, one funnel, one small gun forward, length about 1.8 times its width;
> 3. a blunt boxy landing craft with a square bow ramp and an open cargo well, length about 1.8
>    times its width;
> 4. a long narrow destroyer, two funnels, gun turrets fore and aft, length about 1.8 times its
>    width, clearly bigger than the corvette;
> 5. a wide, low river monitor with one enormous twin-gun turret amidships, the widest hull of all;
> 6. a rich-looking transport yacht with a white hull, a red waterline, stacked crates and a red
>    flag, length about 1.9 times its width.

The lengths in world units are 16, 24, 30, 32, 40, 28 (`docs/ART.md`); what matters is that each
silhouette differs in *shape*, not only size: narrow / one funnel / blunt box / two funnels / one
huge turret / white hull with red.

## Batch B: the U-boats (3)

> [style block] A single row of three submarines of the same family seen from directly above,
> bows pointing to the top of the image, generous white space between them, surfaced, showing the
> narrow hull, the conning tower slightly aft of centre and a deck gun forward of it. They differ
> only in length: short, medium, long (roughly 30, 34 and 38 units at the same width of 12). The
> conning tower gets a small red marking.

## Batch C: emplacements (3 prompts, one per family)

Turrets are never rotated in the game, so the barrel direction is permanent: point it up.

> [style block] Three round gun emplacements seen from directly above, in a row, growing from
> left to right, each a circular sandbag ring with the weapon in the centre, barrel pointing to the
> top of the image:
> - Flak: 1. one light automatic cannon; 2. the same, larger ring; 3. two barrels side by side.
> - 8.8 cm gun: 1. one long-barrelled gun with a shield; 2. the same, larger ring; 3. a twin mount.
> - 15 cm heavy: 1. one heavy barrel in an armoured box; 2. two barrels; 3. three barrels, the
>   biggest emplacement of all.

Run it three times, one bullet each (replace the `- Family:` line by the family's three tiers).

## Batch D: small things (one image)

> [style block] A single row of small map symbols seen from directly above, generous white space
> between them: a torpedo (thin cylinder, pointing up); a shell (tiny dark dot with a red ring);
> six flying debris pieces: a broken gun turret, a funnel, a lifeboat, a crate, an anchor, a hatch
> cover; a bomb crater as a scorched pencil scribble; a water splash column seen from above as a
> white burst with pencil rings; a round bunker with a flat concrete roof and gun slits, red
> marked; an empty pencil circle drawn twice, like a "build here" mark.

Crater, splash and the circle are drawn on the paper, so no cutout edge; everything else is a
cutout.

## Map

Already done: `source/background_1672x941.png` was generated from the geometry guide
(`tools/art_fit.py map-template`) and fits the code within a third of a tile. To regenerate:

> [style block] A hand-drawn treasure-map style top-down map on crumpled white paper, 16:9. Keep the
> river exactly where the blue band is in the attached guide, the banks made of pencil rocks;
> leave the eight circles empty as bare ground; the square at the bottom right is where a bunker
> will be placed, mark it with a red dashed square. Scatter small pencil trees and grass tufts on
> the land, a few red accents.
