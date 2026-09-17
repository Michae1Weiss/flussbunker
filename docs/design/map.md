# Map and river

## Rules

- One fixed map, static camera, 16:9, `ScalingMode::AutoMin`. Other aspect ratios show more paper
  around the map, never less map.
- The river looks organic (curved banks, soft corners) but is built from **deliberate straights
  joined by bends**. Straights are torpedo lanes; bends are dead water for torpedoes.
- Bends are turret territory, straights are U-boat territory. The two tools share the map instead
  of competing for it.
- Several straights of different lengths and positions. Torpedo run length is shorter than the
  longest straight, so *where* on a straight the boat sits is a decision.
- Enemies follow the river centreline from the sea to the bunker. The U-boat moves on the same
  centreline; a click on the water snaps to its nearest point.
- The bunker is at the downstream end. Docked there, the boat is invulnerable and resupplies.

## Build spots

- **8 fixed build spots** in the MVP: ~5 at bends, ~3 at straights, deliberately too few to cover
  everything. Scarcity forces the choice. (Count is a hypothesis.)
- Spots are hidden during the prologue and appear after it, as pencil circles in the prep phase.
  During combat they show only on hover or in the overlay.

## Route display

- Prep phase: the enemy route is always faintly visible as pencil dashes.
- Combat: route only in the Tab overlay.

## Wrecks

- Burning ships drift to the nearest bank and strand there. Sunk ships leave an underwater
  silhouette. Both are cosmetic: no collision, no blocking, no effect on torpedoes.
- After N stranded wrecks the oldest fade to pencil silhouettes so the banks stay readable.

## Backlog

- Buying/unlocking build spots (turns "which spots" into a money decision).
- Second map.
