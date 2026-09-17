# Roadmap

Budget: ~130–150 h until end of 2026. Milestones are ordered by risk: whatever could kill the game
is tested first. Each has a hypothesis and a kill criterion; if the kill criterion fires, the
feature is cut or redesigned before the next milestone starts. Everything is greybox (shapes, no
art) until M8.

| # | Milestone | Deliverable | Hypothesis | Kill criterion | ~h |
|---|---|---|---|---|---|
| 0 | Repo hygiene | crate renamed, `main`, GitHub remote, issues from `BACKLOG.md`, project board, PoC comments stripped per `COMMENTS.md` | – | – | 6 |
| 1 | Boat on the river | river with straights and bends, centreline movement, heading rule, dive, torpedo lanes, deck gun, ramming vs dummy ships | "position is the aim" is fun with rectangles | you do not reposition the boat voluntarily | 25 |
| 2 | Emplacements | 8 spots, radial menu, 3 turrets, upgrades, hold-to-sell, targeting modes, build time | the radial menu is usable without explanation | you need a tooltip to build | 20 |
| 3 | Enemies | 5 types, size class, armour rule, convoy rule, destroyer ranging-in and shelling, breach | the destroyer forces diving | you survive a destroyer surfaced | 15 |
| 4 | Destruction | death matrix, AoE, chains with 0.2 s stagger, debris arcs and craters | chains are readable and motivate placement | you cannot tell what killed the second ship | 15 |
| 5 | The run | prologue, 12 waves, economy from the balance model, boat models, boat loss and repair, onboarding notes, telemetry CSV | a perfect run pays ~60 % | money left over, or wave 4 unwinnable | 20 |
| 6 | Readability | Tab overlay, minimal paper UI, wave preview, feedback notes, juice basics (hit flash, RM tween, shake) | a new player can say why they lost | "I don't know what happened" in playtest | 15 |
| 7 | **Playtest 1** | 5 friends, video, CSV, `PLAYTEST.md` thresholds | thresholds in `PLAYTEST.md` | any threshold missed → fix before M8 | 10 |
| 8 | Placeholder art + web | AI assets in the paper style, WASM build on itch.io, playtest 2 | the style keeps readability at 1280×800 | ships are not distinguishable at a glance | 25 |

Total ~150 h. Anything not in the table is backlog (`BACKLOG.md`) and is not started in 2026.

## Sequencing rules

- Do not start a milestone before the previous kill criterion is checked.
- Numbers come from the balance model (round 3), never typed straight into `balance.rs`.
- A **style test** runs in parallel to M1–M4 at near-zero cost: one hand-drawn frame at game size,
  to learn whether pencil on paper stays readable before 100 h of greybox assume it does.
- Bevy stays pinned; a migration is a separate, user-initiated decision.

## After 2026

Hand-drawn art, more waves and models, branching boat tree, second map, touch mapping (iPad,
phones), CI, buying build spots, two-lane overtaking, integrity repair, ships shelling the bunker.
