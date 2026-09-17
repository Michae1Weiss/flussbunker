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

## Hypotheses to test

| Hypothesis | Where | Kill criterion |
|---|---|---|
| 8 build spots is the right scarcity | `map.md` | testers never face a "which spot" decision, or always run out |
| Testers do not forget the boat mid-wave | `uboat.md` | ≥2/5 leave it docked for a whole wave → add auto-fire baseline |
| Heading-by-movement is not annoying | `uboat.md` | testers complain about turning → auto-swing upstream after 3 s |
| Losing the boat costs the wave plus a 30 % bill, and that feels fair | `uboat.md` | rage-quits after a loss |
| Ranging-in warning is enough to learn diving | `onboarding.md` | <3/5 dive in the first destroyer wave |
| Prologue bounty size gives one real decision | `economy.md` | everyone buys the same thing |
| Three boat models change behaviour enough to matter | `uboat.md` | testers cannot say what changed after buying one |
| 60 % economy target produces tension | `economy.md` | money left over at wave 12 |
