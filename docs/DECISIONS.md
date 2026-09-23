# Decision log

Why things are the way they are. The rules themselves live in `docs/design/`; this file holds the
reasons, the rejected alternatives and the hypotheses still to be tested. Newest at the bottom.
Add an entry when a rule changes, so the "why" survives the person who knew it.

## 2026-09-16 / 17 — planning phase

The proof of concept (rectangles, 12 waves, veterancy ranks) was played five rounds by the author:
interesting, too easy, top rank unreachable, boat discovered late. A planning phase re-derived the
game before any further code. Decisions, with reasons:

| Decision | Why |
|---|---|
| Fixed build spots instead of grid or free placement | fits "operating an emplacement", the radial menu, contextual spot display, and keeps balance tractable |
| River with deliberate straights | "position is the aim" was the best part of the PoC; a fully organic spline would kill torpedo lanes |
| One U-boat, manual torpedo fire | the boat is the identity; a second boat exceeds the attention budget; firing is the fun |
| Fire only surfaced | the surfaced/submerged tension is the core; historically wrong, deliberately |
| Heading = last travel direction, turning not reversing | reversing would make firing downstream at leakers impossible |
| Ships never stop for the boat | a stopping ship makes the boat a barracks and parking the dominant strategy |
| RM as the only currency | kill-stealing between turret and boat felt like cheating; three currencies overload a minimal UI |
| Economy target 60 % | the PoC paid >100 % and was solved by overbuilding |
| No friendly fire, no scatter | output randomness that takes from the player feels unfair; role differences come from stats |
| Debris damage random in place, fixed in amount | bonus-only randomness; plannable AoE stays deterministic |
| Reward at the killing hit | solves overkill targeting on sinking ships |
| Prologue with a gold transport | forces discovery of the boat structurally; a target that cannot hurt you is the best torpedo lesson |
| No forced spending after the prologue | a shopping list is not a decision; a 10 000 RM windfall breaks the currency's scale |
| Destroyer ranges in for 3 s, no scripted stop | keeps "ships are timetables", allows self-earned aha, teaches the real head-on dance |
| Tab as toggle | the overlay is a mode people work in, not a peek |
| Radial menu on click, not right-click-hold | hold-drag fails on trackpads and touch |
| Fictional names, no NS symbols, cartoon tone | §86a StGB and store policy for the German market; the real Bunker Valentin is a memorial |
| Everything English in the repo | portfolio use; German only for unit names |
| Comment convention written before stripping the PoC | without a rule nobody knows what to strip |
| Input module is `input`, messages are named by intent | the PoC's `orders` was fiction naming: three words (input, orders, intents) for one thing cost every reader a lookup |
| Monitor armour above any shell's damage | the mechanical guarantee for pillar 1; with ordinary armour four maxed 15 cm would kill it |
| Destroyer 380 HP, three VIIA torpedoes | 400 HP left 10 HP after three torpedoes, which feels like a wasted torpedo |
| Boat stops, turns on the spot, then moves (M1 feel test, 2026-09-17) | turning while moving read as a car, not a boat; reversing without stopping felt broken |
| Docked hull repair is free and automatic; the only money mechanic of damage is the 30 % bill after a loss (M5) | two repair mechanics would double the rule; the shipyard menu holds only the next model |
| Blast only on Break, not on Explode (M4 feel test) | the matrix as approved; chains come from torpedoes and the 15 cm, not from every kill |
| Polish (M9) before playtest 1, which moves to 2027 (2026-09-23, designer's call) | testers should meet the game with sound, aim and a clean screen; the cost is no outside feedback this year |
| Turret barrels turn to their target, but firing never waits for them | cosmetic only: the balance model and the M5 run stay valid |
| Audio is recorded or sourced by the designer to a written spec (`AUDIO.md`) | the spec fixes length, level and moment, so any source fits the mix |
| Hulls x1.8, emplacements x1.5 (2026-09-22) | the PoC sizes put an MTB at 8 x 13 px at 1280x800, below what any drawing can make readable; the first art in the game showed it. Convoys get longer, debris hits big hulls more often |

## Rejected ideas

| Idea | Why rejected |
|---|---|
| Mortar / AoE turret | flak covers the swarm role |
| XP, ranks, veterancy | kill stealing, currency clutter |
| Debris as a currency; click-to-collect salvage | clutter; collecting chores are disliked; AoE already rewards over-destruction |
| Right-click-hold radial menu; Tab hold-to-show | ergonomics; overlay is a mode |
| Hit a sinking ship for a bonus explosion | overkill/targeting conflict, gimmicky |
| Turret scatter / misses; friendly fire | randomness that takes |
| Auto-fire as a weaker optional mode | punishes exactly the players who need it |
| Boat reverses instead of turning | no downstream shots |
| Ships stop in front of a surfaced boat | barracks exploit |
| Second U-boat; stern tube | attention budget; overkill |
| Enemies damaging turrets | repair economy and a second thing to watch |
| Type-priority targeting | standard modes fit the radial menu |
| Per-wave hint texts; persistent upgrade arrows | too chatty; nagging |
| Bunker upgrade mechanic | a third sink with no interesting decision |
| Forced spending after the prologue | not a decision |
| Scripted stopping destroyer | breaks the timetable rule; single solution; wrong move |
| Extra "tiny" starter boat model | a fourth model and a forced purchase; the VIIA is the tiny boat |

## Runs on record

| Date | Who | Result | Numbers | Reading |
|---|---|---|---|---|
| 2026-09-17 | designer, first full greybox run | won | 11.8 min, integrity 20/20, 372 RM left, 82 torpedoes / 72 hits / 9 lost, turrets 2/2/2 + 8 upgrades + 2 sells, 180 kills incl. 17 chain and 6 ram, boat never sunk, VIIC bought mid-run | felt close because of the boat, not the bunker. No balance change before playtest 1; levers ready: economy target 50 %, +10 % enemy HP from wave 8 |

## Hypotheses to test

| Hypothesis | Where | Kill criterion |
|---|---|---|
| 8 build spots is the right scarcity | `map.md` | testers never face a "which spot" decision, or always run out |
| Testers do not forget the boat mid-wave | `uboat.md` | ≥2/5 leave it docked for a whole wave → add auto-fire baseline |
| Stop-turn-go at bends is not annoying | `uboat.md` | testers complain about standing still at bends → let small angles turn while moving |
| Losing the boat costs the wave plus a 30 % bill, and that feels fair | `uboat.md` | rage-quits after a loss |
| Ranging-in warning is enough to learn diving | `onboarding.md` | <3/5 dive in the first destroyer wave |
| Prologue bounty size gives one real decision | `economy.md` | everyone buys the same thing |
| Three boat models change behaviour enough to matter | `uboat.md` | testers cannot say what changed after buying one |
| 60 % economy target produces tension | `economy.md` | money left over at wave 12 |
