# U-boat

Exactly one boat, present from the first second. It is the identity of the game
(`docs/VISION.md`, pillar 1).

## Movement

- Left-click on the river: the boat travels along the centreline to the nearest point.
  Clicking the bunker sends it home. The last order wins; orders are never queued or refused.
- **Heading:** the bow points in the last direction of travel. There is no turn command; to turn,
  move. The boat **stops, turns on the spot (~0.8 s for 180°), then moves**; at a bend it does the
  same for the smaller angle. Turning while moving felt wrong in the first feel test.
  - Hypothesis: if standing still at bends annoys, let small angles turn while moving.
- Surface speed 100 %, submerged 70 %.

## Dive

- Right-click toggles dive/surface (`T` secondary). Transition 1.4 s each way.
- Orders during a transition or while moving are always accepted: a dive can be cancelled by
  surfacing, a move can be issued mid-dive.
- Surfaced: can fire, can be shelled by destroyers, collides with ships.
- Submerged: safe, blind (cannot fire), ships pass over it.
- The transition time and the slower submerged speed are the whole cost of diving.

## Torpedoes

- **Manual fire.** Space = one torpedo. Hold Space = salvo, ~0.3 s between tubes.
- Fire is accepted only when surfaced and the bow is within ±10° of the river axis. Otherwise a
  short feedback note ("bow not aligned"), no torpedo.
- Empty tubes: dry click plus a one-time note per press. No repeated feedback while holding.
- Torpedoes run straight along the current straight and hit the bank at the next bend: water
  column, no damage. Bends are dead water.
- A torpedo hits the first hull on its line. It ignores armour.
- Tubes reload fast; torpedo **resupply only at the bunker**. Sortie → shoot → return is the rhythm.
- Hypothesis: "do testers forget the boat mid-wave?" If yes, add auto-fire as the *baseline* with
  Space as "fire now" — never as an optional, weaker mode.

## Deck gun

Fires automatically at anything in range while surfaced. Weak, always available; the answer to
"no torpedoes left and a boat is heading for the bunker". One manual weapon, one automatic.

## Ramming — ships never stop, they ram through

| Situation | Light boat | Medium / heavy ship |
|---|---|---|
| Passive collision: surfaced boat in the lane | boat capsizes, small dent on the U-boat | U-boat takes heavy damage and is shoved downstream; ship slows briefly, continues |
| Active ram: surfacing under the ship | guaranteed capsize, little U-boat damage | **blocked**: surfacing refused, note "contact above", order not queued |

- A ship that stopped in front of a surfaced boat would make the boat a barracks (park it, jam
  the convoy, let turrets farm). Ships therefore never stop.
- The active ram is strictly better than the passive collision, so parking is never the plan.
- Costs of ramming: hull damage, the dive/surface cycle, being surfaced inside the convoy afterwards.
- Ramming is never taught; it is discovery. The overlay may hint at it.

## Damage and loss

- HP is the only damage number. Docked at the bunker the boat is invulnerable and repairs.
- Only destroyers (and possibly the monitor) attack the boat: they **shell** a surfaced boat.
  A destroyer ranges in for ~3 s first (splashes next to the boat, no damage), then hits.
- Sunk → the boat is gone for the rest of the wave. That is the real cost.
- Prep phase: the boat returns automatically with its model. The shipyard bills ~30 % of the
  boat's value; if unaffordable it returns at 50 % HP and can be repaired for RM later.
  No wait timer (prep time is unlimited, so waiting would cost nothing). Hypothesis.

## Models

Three, linear, bought with RM at the docked boat's radial menu, unlocked by wave reached. Each
changes behaviour, not only numbers. No separate equipment upgrades in the MVP.

| Model | Role | Behavioural change (hypothesis) |
|---|---|---|
| VIIA | starter, the small boat | 4 torpedoes, baseline |
| VIIC | all-rounder | more torpedoes, faster dive |
| IXC | heavy | two salvos, tougher, slower dive |

Post-MVP: an "underwater hunter" (Type XXI) that may fire submerged but cannot ram and has no deck
gun.

## Edge cases

| Case | Rule |
|---|---|
| Surface order while a light boat is above | boat capsizes (active ram) |
| Surface order while a heavy ship is above | refused, feedback, not queued; the player retries |
| Space while turning | refused with feedback until the bow is aligned |
| Move order to the sea end of the river | clamped to the first centreline point |
| Two orders in quick succession | last order wins |
| Ships reach the bunker while the boat is docked | the boat is untouched |
| Boat sunk while a torpedo is in the water | the torpedo keeps running |
| 2× speed active when a wave starts | speed resets to 1× |
