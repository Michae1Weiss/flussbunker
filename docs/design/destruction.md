# Destruction

Pillar 2: destruction is the reward. Half of it is mechanics and must be greyboxed; the other half
is presentation and waits for art.

## Randomness

1. **Presentation randomness** (which animation, pitch, debris direction): unlimited.
2. **Output randomness** may only *give* to the player, never take; never decides a wave; is
   always visible.
3. The plannable part is deterministic: explosion AoE has a fixed radius per class. Debris hits
   are the bonus on top.

No friendly fire of any kind: shells, AoE and debris never hurt the boat, turrets or the bunker.
No turret scatter or misses.

Reading: Costikyan, *Uncertainty in Games*; GMTK, "The Two Types of Random in Game Design";
Sid Meier, GDC 2010.

## Death-style matrix (class × killing weapon)

Burn = drifts to the bank, no debris, no AoE. Explode = sinks in place, one debris piece (the
turret). Break = two halves sink, several debris pieces, damage radius.

| | Flak | 8.8 cm | 15 cm | Torpedo | Ram | Deck gun |
|---|---|---|---|---|---|---|
| **light** | burn | explode | **break + AoE** | explode | capsize | burn |
| **medium** | burn | explode | break + AoE | break + AoE | – | burn |
| **heavy** | cannot kill | cannot kill | explode | break + AoE | – | – |
| **capital** | – | – | – | **huge break, large AoE, many debris** | – | – |

## AoE and chains

- Radius fixed per class, damages enemies only.
- Chain reactions are allowed without cap. Each link fires **0.2 s after the previous** so the chain
  is readable rather than one flash.

## Debris

- Types: turret (most damage), funnel, lifeboat, crates, anchor, hatch. Damage per type is fixed;
  only the landing spot is random.
- Lands on a ship → damage. Water → sinks. Land → a crater drawn on the canvas. Never on the boat,
  a turret or the bunker. A crater on a build spot is cosmetic.
- Count per class: light 1–2, medium 3–4, heavy 5–6, capital 8+. Cap ~20 airborne at once.
- Debris is presentation plus bonus damage. There is no salvage or collecting.

## Presentation (post-greybox)

- Several animations for the same action (reload, muzzle, hit), chosen at random.
- Screen shake on torpedo hits and heavy explosions, scaled by class, can be disabled in settings.
- Torpedo hit: the Silent Hunter moment — water column, brief hitstop.
- Underwater silhouettes of sunk ships; stranded wrecks on the banks (`map.md`).
