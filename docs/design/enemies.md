# Enemies

Five types for the MVP; variety comes from mixing, not from more types. Each enemy carries a
**size class** that drives ramming, death style, debris count and turret roles: one value, many
rules.

## Threat → answer

| Enemy | Class | Threat | Best answer | Second best | Useless |
|---|---|---|---|---|---|
| Motor torpedo boat | light | numbers, speed | naval flak | deck gun, ram, explosion AoE | 15 cm (too slow) |
| Corvette | medium | baseline | 8.8 cm | flak (slow), torpedo (wasteful) | – |
| Landing craft | medium, tough | a breach hurts | 8.8 cm AP, 15 cm | torpedo | flak |
| Destroyer | heavy | shells the U-boat | **torpedo** | 15 cm (slow) | flak, 8.8 |
| Monitor | capital | bunker breaker | **torpedo only** | – | all guns (belt armour above any shell's damage: shells bounce) |

A run without turrets is possible; a run without the boat is not. That is pillar 1.

## Rules

- **Armour** subtracts a flat amount from shell damage (10 % floor). Torpedoes ignore armour. One
  number produces the whole counterplay structure.
- Ships are timetables. They never evade, speed up, or shoot at turrets. The game is the answer
  to the timetable.
- **Convoy rule:** a ship cannot overtake a slower ship ahead; it keeps a minimum distance.
  Wave design puts fast groups before slow ones unless a clump is intended (a clump is a gift to
  15 cm AoE). Two-lane overtaking is a post-2026 experiment.
- **Ships never stop** for the U-boat; they ram through (`uboat.md`).
- Only destroyers (and possibly the monitor) attack the boat, by shelling it while surfaced.
  A destroyer ranges in for ~3 s (visible splashes, no damage) before hitting. This warning window
  is what makes losing the boat feel fair, and it is what teaches diving without a script.
- **Breach:** a ship reaching the bunker is removed and deducts integrity. The breach is shown:
  impact, explosion, one more pencil crack on the bunker drawing. Ships shelling the bunker as a
  last stand is a backlog option.
- **Damage readability:** three visible stages on the drawing (smoke, list, fire). Exact HP as a
  pencil note in the Tab overlay. Greybox uses health pips.
- Reward is paid at the killing hit; the ship leaves the targeting system immediately.

## The gold transport (prologue)

A slow enemy gold transport cruises up and down the river before wave 1. It cannot reach the bunker,
cannot shoot back, ignores turrets (there are none yet) and dies only to the boat. The prologue
ends when it sinks. Its bounty is in normal scale, big enough for one real first decision
(`economy.md`); the cha-ching presentation may be as big as it likes.

## Waves

Prologue plus 12 waves. Composition and numbers are round-3 balance work; the pacing rules are
fixed:

- pressure → breathing room → new information → pressure
- every third wave introduces a threat the previous build cannot answer; the next two drill it
- fast groups before slow groups (convoy rule)
- the first destroyer comes alone and telegraphed
- wave 12 is winnable on a second or third run by a player who has understood the boat; a
  first-attempt clear by a newcomer means the curve is too flat
