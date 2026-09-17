# Turrets

Three roles, not a calibre ladder. Each upgrade must change behaviour visibly.

| Turret | Role | Tier 1 | Upgrade 1 | Upgrade 2 |
|---|---|---|---|---|
| Naval flak | anti-swarm, light boats | one fast gun | higher rate + larger magazine | two longer barrels + autoloader |
| 8.8 cm coastal gun | precise all-rounder | single gun | fire control + AP (halves armour) | twin gun / burst |
| 15 cm battery | heavy, slow | one barrel | two barrels | three barrels |

## Rules

- Built only on the fixed build spots (`map.md`). Build takes 2–3 s, visibly under construction,
  does not fire meanwhile. **Upgrades are instant** — a turret that goes offline mid-wave because
  you upgraded it would punish the player's hand.
- Two linear upgrades each. Upgrade affordances exist **only inside the radial menu**, never as a
  persistent arrow on the turret.
- Sell refunds 60 %, allowed during a wave, spot immediately buildable again. Selling is
  hold-to-confirm: hold 0.5 s, a ring fills, release early cancels. No dialog.
- Targeting modes per turret: **First** (default), Strongest, Weakest, Nearest. Cycled by clicking
  the entry in the radial menu; current mode shown in the menu and in the overlay.
- **Shells always hit** their target (tracked). Turrets differ by turn rate, reload, shell speed and
  damage, never by dice. Turning is presentation plus a short delay before the first shot.
- Armour subtracts a flat amount from shell damage with a 10 % floor; torpedoes ignore armour
  (`enemies.md`). The 8.8 AP upgrade halves armour, never removes it.
- Turrets are invulnerable. Enemies never damage them.
- Range is shown on hover, during placement and in the overlay.

## Radial menu

| Context | Entries |
|---|---|
| Empty spot | three blueprints with cost; the ghost shows range and turns toward the river; unaffordable ones greyed |
| Built turret | Upgrade (ghost shows the change, e.g. the second barrel appears) · Sell (amount shown, hold) · Targeting mode |

The menu opens on left-click, closes on Esc or a click elsewhere. The game keeps running; there is
no hidden pause.

## Edge cases

| Case | Rule |
|---|---|
| Sold while its shell is in flight | the shell keeps flying and hits; it is its own entity |
| Menu open, Enter pressed | the wave starts, the menu stays open |
| Menu open, Space pressed | torpedo fires; menus never swallow game intents |
| Menu open, click on the river | the menu closes only; a second click moves the boat |
| Debris lands on a free spot | cosmetic; the spot stays buildable, the crater is drawn around the turret |
| Not enough RM for an option | blueprint stays greyed, short "too expensive" note, menu stays open |
| Two targets at equal distance | deterministic: the one spawned first |
| Upgrade bought while the turret is turning | instant; the turret keeps its current target |
