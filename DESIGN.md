# Design: Bunker Valentin

## 0. A note on the setting

Bunker Valentin in Bremen-Rekum is a real place and today a memorial
(*Denkort Bunker Valentin*). It was built with forced labour; over a thousand
people died there. That is worth knowing before shipping anything with the name
on it. Nothing in the code depends on the theme — `enemy`, `tower`, `river`,
`ally` are the actual mechanics, and swapping the fiction is a rename. If you
ever put this in front of other people, a one-screen historical note costs
nothing and makes the difference between "period setting" and "tone-deaf".

---

## 1. The core loop

> **Read the river → spend marks on batteries and u-boats → start the wave →
> reposition the u-boat → collect bounty → adapt.**

One sentence, verbs only. The test that matters: is this engaging after 30
seconds with no art, no sound and no story? Everything below is judged against
that sentence.

The loop has two nested time scales, on purpose:

| Scale | Decision | Reversible? |
|---|---|---|
| Between waves (seconds to a minute) | Where to build, what to build, whether to bank marks for a u-boat | No — towers are permanent |
| During a wave (continuous) | When to launch, where to station, when to pull back | Yes — u-boats can be repositioned |

A tower defense with only the first scale becomes a screensaver: place the
known-best build, watch. That is the single most common failure mode of the
genre, and the second scale exists specifically to prevent it.

---

## 2. Design alternatives considered

Six shapes fit "boats on a river, you defend the bunker". They are genuinely
different games, not skins.

| # | Shape | What it plays like | Why not (or yes) |
|---|---|---|---|
| **A** | **Fixed path, grid build** (Kingdom Rush) | River is authored, you build on the banks | **Chosen.** Balancing is tractable, the map is readable at a glance, and the river *is* the path — the fiction and the mechanic are the same thing. Pathfinding disappears entirely: the level data is a `Vec` of waypoints. |
| B | Maze-building (Desktop TD) | You dredge and dam channels to lengthen the route | Water routing is a lovely fantasy but the genre's known trap: one optimal maze exists, players find it, the game is solved. Also needs real pathfinding plus "you may not fully block" validation. Two systems and a balance nightmare for one idea. |
| C | Lanes (Plants vs Zombies) | 4–5 parallel straight rivers | Very readable, very easy to build. But lanes make position trivial — the interesting spatial question ("does this hill cover both bends?") vanishes. Great onboarding, less depth. |
| D | Free placement, no grid | Towers anywhere, radius overlap is the puzzle | Removes the grid, and with it the clean "is this tile buildable" rule. Placement becomes fiddly with a mouse and much harder to reason about. |
| E | Reverse TD | You *are* the u-boats, attacking British defences | A better fit for the fiction, honestly. But it is an entirely different genre (RTS-lite), and it discards the tower-defense learning goals. Log it as game #2. |
| F | Fixed-position shooter (Storm the House) | You man the bunker's guns directly | Excellent short loop and the shortest path to fun. But then it is an action game where towers are upgrades, not a tower defense. |

**A + one mechanic from E.** Fixed path for the skeleton, plus a
player-controlled u-boat as the active layer. That is the "one new mechanic on
top of a proven loop" heuristic: everything except the u-boat is a known-good
tower defense, so if the game is not fun you know exactly which part to blame.

---

## 3. The u-boat mechanic, and why it was changed

The brief: *the bunker creates u-boats that spawn and help me defend*.

Taken literally — free boats appearing on a timer and fighting on their own —
that is an **anti-mechanic**. It adds spectacle while *removing* agency: more
happens on screen, the player does less. It also breaks the economy, since free
combat power that scales with time makes the build decision matter less every
minute.

Same fiction, agency restored:

| Property | Effect on the loop |
|---|---|
| Costs 110 marks | Competes with towers for the same budget — a real trade |
| 9 s build time | Ordering one is a bet on the *next* wave, not this one |
| Player picks the station tile | A positioning decision, mid-wave, every wave |
| 4 torpedoes, then sails home | A resource clock: the sortie has a beginning and an end |
| Destroyers sink it | The decision has downside; timing matters |
| Only source of torpedo damage | It is the *answer* to a threat, not a general power-up |

The last row is the important one. Tower variety in a good tower defense does
not come from having many towers — it comes from **enemies that demand
different answers**. One armour stat plus one damage type that ignores it
generates the entire counterplay structure of this game:

```
Destroyer armour 7  ·  Flak shell 9  →  2 damage per hit (guns are useless)
Destroyer armour 7  ·  Torpedo 60    →  60 damage        (u-boats are the answer)
```

That is one number in `balance.rs` doing the work a whole tower tree usually
does.

---

## 4. Wave pacing

The intent is written down in `balance.rs` next to the data, so the numbers can
be checked against it:

1. **Aufklärung** — six MTBs. Teaches: build, watch, earn. No pressure.
2. **Schnellbootrudel** — twelve MTBs, tighter spacing. Same threat, more volume: teaches that one battery is not enough.
3. **Geleitzug** — corvettes with an MTB screen behind them. Teaches target priority.
4. **Zerstörer voraus** — one destroyer, alone, telegraphed. This is the teaching moment: guns visibly stop working. The player has ~40 s to work out why, and one u-boat is already docked.
5. **Großangriff** — three destroyers mixed with escorts. Punishes a mono-build in both directions: guns-only cannot kill destroyers, u-boat-only cannot cover the swarm.

The rhythm is pressure → breathing room → new information → pressure. Waves
never auto-start; the player presses Space. That guaranteed decision window is
where the loop actually lives.

---

## 5. Juice, in priority order

Readability first, spectacle later — in a tower defense the player must be able
to see *why* a lane fell.

Already in the code (cheap, high value):
- Range circle and build-legality colour on hover — without this the player cannot plan.
- Health pips under damaged boats — without this "why did that survive" is unanswerable.
- Boats rotate to their heading; splash sprites fade on impact.

Next, in order of value per hour:
1. Enemy flashes white for 2 frames on hit (`Sprite.color`, 0.1 s timer).
2. Money counter tweens instead of snapping — 0.3 s lerp on the displayed number.
3. Firing sound with ±10% pitch variation; a placement "thunk".
4. Coin sprite flying from a kill toward the marks counter.
5. Wave-start banner; brief slow-mo (`Time<Virtual>::set_relative_speed(0.3)`) on wave clear.
6. Screen shake and a red vignette pulse on a breach.

Items 1, 2 and the existing range indicator are feedback. The rest is decoration.
Do them in that order.

---

## 6. Deliberately cut from v1

Logged, not built. Over-scoping is the reason hobby games do not ship, and it
hits experienced developers hardest.

- Tower upgrades and branching paths
- Selling and refunds
- A second map, or any map selection
- Air attack (Beaufighters) as a threat towers cannot reach
- Minefields as a one-shot consumable
- Searchlight support tower (aura: +range for neighbours)
- Save/load, settings, audio
- Sprites instead of coloured rectangles

Each is a good idea. Each is also a reason not to finish. The go/no-go rule:
**nothing on this list gets built until five waves are genuinely fun to play
twice.**
