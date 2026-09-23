# Roadmap

Budget: ~130–150 h until end of 2026. Milestones are ordered by risk: whatever could kill the game
is tested first. Each has a hypothesis and a kill criterion; if the kill criterion fires, the
feature is cut or redesigned before the next milestone starts. Everything is greybox (shapes, no
art) until M8.

| # | Milestone | Deliverable | Hypothesis | Kill criterion | ~h |
|---|---|---|---|---|---|
| 0 | Repo hygiene | crate renamed, `lib.rs` + thin `main.rs`, `input` module, toolchain + lints, project board, PoC comments stripped per `COMMENTS.md` | – | – | 6 |
| 1 | Boat on the river | river with straights and bends, centreline movement, heading rule, dive, torpedo lanes, deck gun, ramming vs dummy ships | "position is the aim" is fun with rectangles | you do not reposition the boat voluntarily | 25 |
| 2 | Emplacements | 8 spots, radial menu, 3 turrets, upgrades, hold-to-sell, targeting modes, build time | the radial menu is usable without explanation | you need a tooltip to build | 20 |
| 3 | Enemies | 5 types, size class, armour rule, convoy rule, destroyer ranging-in and shelling, breach | the destroyer forces diving | you survive a destroyer surfaced | 15 |
| 4 | Destruction | death matrix, AoE, chains with 0.2 s stagger, debris arcs and craters | chains are readable and motivate placement | you cannot tell what killed the second ship | 15 |
| 5 | The run | prologue, 12 waves, economy from the balance model, boat models, boat loss and repair, onboarding notes, telemetry CSV | a perfect run pays ~60 % | money left over, or wave 4 unwinnable | 20 |
| 6 | Readability | Tab overlay, minimal paper UI, wave preview, feedback notes, juice basics (hit flash, RM tween, shake) | a new player can say why they lost | "I don't know what happened" in playtest | 15 |
| 8 | Art + web | hand-drawn sprites and map in the paper style (`ART.md`), WASM build on itch.io | the style keeps readability at 1280×800 | ships are not distinguishable at a glance | 25 |
| 9 | Feel and skin | turrets that aim, recoil and flash; explosions; sound and music (`AUDIO.md`); decluttered paper UI with 9-slice notes, icons, pencil strokes; phase banners | every action answers with sight and sound, and the screen says only what the player needs now | a tester asks what a turret is shooting at, or reads a key hint after the first wave | 30 |
| 7 | **Playtest 1** (2027) | 5 friends, video, CSV, `PLAYTEST.md` thresholds | thresholds in `PLAYTEST.md` | any threshold missed → fix before publishing | 10 |

Total ~160 h. Order changed on 2026-09-23 by the designer: art (8) and feel (9) come before the
first playtest, which moves to 2027. The cost is no outside feedback in 2026. Anything not in the
table is a GitHub issue labelled `idea` and is not started in 2026.

## Sequencing rules

- Do not start a milestone before the previous kill criterion is checked.
- Numbers come from the balance model (`tools/balance_model.py` → `docs/balance/`), never typed straight into `balance.rs`.
- Feel is tested in `examples/` harnesses around the crate's plugins (`architecture.md`), one question per harness.
- A **style test** runs in parallel to M1–M4 at near-zero cost: one hand-drawn frame at game size,
  to learn whether pencil on paper stays readable before 100 h of greybox assume it does.
- Bevy stays pinned; a migration is a separate, user-initiated decision.

## After 2026

Hand-drawn art, more waves and models, branching boat tree, second map, touch mapping (iPad,
phones), CI, buying build spots, two-lane overtaking, integrity repair, ships shelling the bunker.
