# Design decisions — Flussbunker (working title)

Status: **draft, planning phase** (started 2026-09-16). Nothing here is implemented yet.
Where this file conflicts with `DESIGN.md`, `ARCHITECTURE.md` or `docs/IDEAS.md`, this file wins.
Those three will be pruned into the final doc structure once planning is done.

Legend: **[locked]** decided · **[hyp]** hypothesis, needs a playtest · **[open]** still to decide

---

## 1. Vision

- **What it is:** a 2D tower defense on a river. You hold the banks with shore batteries and personally command one U-boat. **[locked]**
- **Purpose:** learning project that gets "finished enough", and a portfolio piece. Not commercial. **[locked]**
- **Budget:** ~8–10 h/week until end of 2026 → roughly 130–150 h. Hand-drawn final art is out of scope for 2026. **[locked]**
- **Player feeling after 30 min (MDA):** spectacle & chaos, challenge, discovery (new waves, enemies, upgrades). **[locked]**
- **References, one thing each:** Silent Hunter (firing a torpedo, the ticking wait, the water column); Toy Defense 2 (upgrade feel); Storm the House 3 (passive defense plus one proactive weapon). **[locked]**
- **Not this:** no RTS, no idle TD, no micromanagement stress, no realism, no multi-session runs. **[locked]**

### Design pillars **[locked 2026-09-17]**

1. **The U-boat is the game.** Heavy ships can only be sunk by torpedoes; a run without the boat is impossible by design. Turrets support, the boat decides.
2. **Destruction is the reward.** Every kill is a readable little show; how a ship dies depends on what killed it.
3. **Never punish the player's hand.** One manual action (torpedoes), interaction is never blocked, randomness only ever gives.
4. **Paper and pencil.** Diegetic, minimal UI; hand-drawn paper-cutout look (after greyboxing).

Every future idea is checked against these four. If it serves none, it goes to the backlog.

---

## 2. Run structure

- MVP: one map, 12 fixed waves, 10–15 min per run. Later: more waves, up to ~30 min. **[locked]**
- No meta-progression; every run is a fresh start. **[locked]**
- Lose: bunker integrity reaches 0. Win: survive all waves. **[locked]**
- Pause and 2× speed available. Building during a wave is allowed. **[locked]**
- Waves never auto-start; Enter starts the next wave. Unlimited prep time. **[locked]**
- Breach: a ship reaching the bunker is removed and deducts integrity (PoC behaviour). The breach must be *shown* (impact, crack drawn on the bunker). Ships shelling the bunker as a "last stand" is a later option. **[locked for MVP]**

---

## 3. Economy

- **Reichsmark (RM) is the only currency.** XP, ranks, veterancy and a debris currency are dropped. **[locked]**
  - Why: removes kill-stealing (turret takes the kill your torpedoes paid for), removes currency clutter in a minimal UI, enables an all-in-on-the-boat playstyle.
- U-boat upgrades are bought with RM; new models unlock by wave reached ("shipyard tier"), not by level. **[locked]**
- Starting money: **[open]** — candidate idea: start nearly broke so the first kill must come from the boat (forces discovery of the boat; risk: death spiral if it fails).

---

## 4. Map & river

- Fixed map, static camera, 16:9 with `ScalingMode::AutoMin`. **[locked]**
- River is organic-looking (curved banks, soft corners) but built from **deliberate straights** joined by bends. Straights are torpedo lanes, bends are dead water. **[locked]**
- Rule of thumb: bends are turret territory, straights are U-boat territory.
- The map needs several straights of different lengths; torpedo run length is shorter than the longest straight so the position *on* a straight matters. **[locked]**
- Turrets are built on **fixed build spots** (Kingdom Rush style), shown contextually. **[locked]**
- MVP map: **8 build spots**, ~5 at bends and ~3 at straights, deliberately too few to cover everything. **[locked, count is hyp]**

---

## 4a. Turrets

- Three turret types, each a role, not a calibre ladder: naval flak (anti-swarm), 8.8 cm (precise all-rounder), 15 cm (heavy). Two linear upgrades each; every upgrade changes behaviour, not only numbers. **[locked]**
- Build takes 2–3 s; the turret is visibly under construction and does not fire. **Upgrades are instant** (an upgrade that takes the turret offline mid-wave would punish the player's hand). **[locked]**
- Sell refunds 60 %, allowed during a wave; the spot is immediately buildable again. **[locked]**
- Targeting modes per turret: First (default), Strongest, Weakest, Nearest. Type priority may come later. **[locked]**
- Shells always hit their target (tracked, as in the PoC); only torpedoes fly physically. Turrets differ by turn rate, reload, shell speed and damage. Turning is presentation plus a short delay before the first shot. **[locked]**
- Turrets are invulnerable; enemies never damage them. **[locked]**
- Radial menu opens on click of a spot or turret; the game keeps running (no hidden pause); Esc or clicking elsewhere closes it. **[locked]**

Edge cases **[locked]**:

| Case | Rule |
|---|---|
| Turret sold while its shell is in flight | Shell keeps flying and hits; it is its own entity |
| Radial menu open, Enter pressed | Wave starts, menu stays open |
| Radial menu open, Space pressed | Torpedo fires; menus never swallow game intents |
| Radial menu open, click on the river | Menu closes only; a second click moves the boat |
| Debris lands on a free build spot | Cosmetic; spot stays buildable, crater is drawn around the turret |
| Not enough RM for the chosen option | Blueprint stays greyed, short "too expensive" feedback, menu stays open |
| Two targets at equal distance | Deterministic: the one spawned first |

---

## 5. Threat → answer

Armour rule kept from PoC: armour subtracts a flat amount from shell damage; torpedoes ignore armour. **[locked]**

| Enemy | Class | Threat | Best answer | Second best | Useless |
|---|---|---|---|---|---|
| Motor torpedo boat | light | numbers, speed | naval flak | deck gun, ram, explosion AoE | 15 cm (too slow) |
| Corvette | medium | baseline | 8.8 cm | flak (slow), torpedo (wasteful) | – |
| Landing craft | medium, tough | breach hurts | 8.8 cm AP, 15 cm | torpedo | flak |
| Destroyer | heavy | hunts the U-boat | **torpedo** | 15 cm (slow) | flak, 8.8 |
| Monitor | capital | bunker breaker | **torpedo only** | – | all guns |

- 8.8 cm AP upgrade *halves* armour, never removes it. Destroyers stay torpedo territory. **[locked]**
- Five enemy types are enough for 12 waves; variety comes from mixing. **[locked]**
- Only destroyers (and possibly the monitor) attack the U-boat. They **shell** a surfaced boat; a submerged boat is safe. (Replaces the PoC's depth charges on a surfaced boat, which was fiction-wrong.) **[locked]**
- Mortar/AoE turret from the PoC is dropped; flak is the swarm answer. **[locked]**
- A **size class** (light / medium / heavy / capital) on each enemy drives ramming, death style, debris amount and turret roles. One value, many rules. **[locked]**
- **Convoy rule:** a ship cannot overtake a slower ship ahead; it keeps a minimum distance. Consequence for wave design: fast groups go *before* slow ones unless a clump is intended. Two-lane overtaking is a post-2026 experiment. **[locked]**
- Ships never react to being shot (no evasion, no speed-up, no return fire). Ships are timetables; the game is the answer to them. **[locked]**
- Damage readability: three visible damage stages on the drawing (smoke, list, fire); exact HP as a pencil note in the Tab overlay. Greybox uses health pips. **[locked]**

---

## 6. U-boat

- Exactly one U-boat. **[locked]**
- Present from wave 1 (no purchase step). **[locked]**
- **Fire only when surfaced.** Surfaced = can shoot, can be shot. Submerged = safe, blind. (Exception: a late "underwater hunter" model may fire submerged but cannot ram and has no deck gun.) **[locked]**
- **Manual torpedo fire:** Space = one torpedo; hold Space = salvo, ~0.3 s between tubes. No auto-fire mode in the MVP. **[locked]** — **[hyp]** "Do testers forget the boat mid-wave?" If yes, add auto-fire as the *baseline* with Space as "fire now", never as a worse optional mode.
- Torpedoes run straight along the current straight; at the next bend they hit the bank (water column, no damage). **[locked]**
- Tubes reload fast (salvo possible); torpedo *resupply* only at the bunker. Sortie → shoot → return home is the rhythm. **[locked]**
- Deck gun fires automatically at anything in range while surfaced. The boat has one manual weapon and one automatic weapon. **[locked]**
- Movement: left-click on the river; the boat goes to the nearest point on the centreline. Orders during dive/surface/movement are always accepted (never block interaction). **[locked]**
- **Heading:** bow points in the last direction of travel; no explicit turn command — to turn, move. Turning is animated (~0.8 s for 180°). Space is accepted only when the bow is within ±10° of the river axis, otherwise short feedback. **[locked]** — **[hyp]** if this annoys, auto-swing bow upstream after 3 s at rest.
- Dive cost: 1.4 s transition each way **and** 70 % speed while submerged. **[locked]**
- Docked at the bunker the boat is invulnerable. **[locked]**
- Models for MVP: 3, linear (VIIA → VIIC → IXC). Each changes *behaviour*, not just numbers. Branching later. **[locked]**

### Ramming (ships never stop; they ram through) **[locked, hyp on numbers]**

| Situation | Light boat | Medium / heavy ship |
|---|---|---|
| Passive collision (boat surfaced in the lane) | boat capsizes, small dent on the U-boat | U-boat takes heavy damage, is shoved downstream, ship slows briefly and continues |
| Active ram ("Rammstoß": surfacing under the ship) | guaranteed capsize, little U-boat damage | **blocked**: surfacing refused, feedback "contact above", order not queued |

- Why ships never stop: a stopping ship turns the surfaced boat into a barracks (park it, jam the convoy, let turrets farm). That would be the dominant strategy.
- The active ram is strictly better than the passive collision, so parking is never the plan.
- Ram costs: hull damage per ram, the dive/surface cycle, and being surfaced inside the convoy afterwards.
- "No torpedoes, ship at the bunker" is what the deck gun is for; the ram is the panic button.

### Losing the boat **[hyp]**

- Sunk → gone for the rest of the wave. That is the real cost.
- Prep phase: the boat returns automatically with model and upgrades. The shipyard bills ~30 % of the boat's value. If unaffordable, it returns at 50 % HP and can be repaired for RM later.
- No wait timer (prep time is unlimited, so waiting costs nothing). No torpedo/damage penalties (keep it to one visible number: HP).

---

## 7. Destruction, randomness, debris

### Randomness rules **[locked]**

1. Presentation randomness (animations, pitch, debris direction): unlimited.
2. Output randomness may only *give* to the player, never take; never decides a wave; always visible.
3. The plannable part is deterministic: explosion AoE has a fixed radius. Debris hits are the bonus on top.

Reading: Costikyan *Uncertainty in Games*; GMTK "The Two Types of Random in Game Design"; Sid Meier GDC 2010.

- No friendly fire of any kind (shells, AoE, debris never hurt the boat, turrets or bunker). **[locked]**
- No turret scatter / misses. Role differences come from stats and rules (turn rate, reload, shell speed, damage), never from dice. Impact point may be visually offset within the hull. **[locked]**
- Dead ships: reward is paid at the killing hit; the ship leaves the targeting system immediately. Burning ships drift to the bank as pure presentation; wrecks are cosmetic; underwater silhouettes remain. Old bank wrecks fade to pencil silhouettes after N. **[locked]**
- "Hit a sinking ship for a bigger explosion" idea: dropped. **[locked]**
- Debris collecting: **dropped for the MVP.** Debris is presentation only; over-destruction is already rewarded by AoE and chains. Salvage may return if a later mechanic gives it a purpose. **[locked]**

### Death-style matrix (class × killing weapon) **[locked]**

Burn = drifts to the bank, no debris, no AoE. Explode = sinks in place, one debris piece (the turret). Break = two halves sink, several debris pieces, damage radius.

| | Flak | 8.8 cm | 15 cm | Torpedo | Ram | Deck gun |
|---|---|---|---|---|---|---|
| **light** | burn | explode | **break + AoE** | explode | capsize | burn |
| **medium** | burn | explode | break + AoE | break + AoE | – | burn |
| **heavy** | cannot kill | cannot kill | explode | break + AoE | – | – |
| **capital** | – | – | – | **huge break + large AoE + many debris** | – | – |

- AoE radius is fixed per class; it damages enemies only. Chain reactions are allowed without cap; each link fires 0.2 s after the previous one so the chain is readable. **[locked]**
- Debris types: turret (most damage), funnel, lifeboat, crates, anchor, hatch. Damage per type is fixed; only the landing spot is random. Lands on: ship (damage), water (sinks), land (crater). Never on the boat, a turret or the bunker. **[locked]**
- Debris count by class: light 1–2, medium 3–4, heavy 5–6, capital 8+. Cap ~20 airborne at once. **[locked]**
- Screen shake on torpedo hits and heavy explosions, scaled by class, can be disabled in settings. **[locked]**

---

## 8. Theme & tone

- Fictional setting, fictional names, as little story as possible. No real place names (not "Bunker Valentin"). **[locked]**
- Kriegsmarine-era optics yes; **no NS symbols**, no references to the regime. Real weapon and U-boat class names (8.8 cm, Type VIIC) and Reichsmark are fine. Avoid "Ritterkreuz" naming. **[locked]**
- Tone: playful / cartoonish. **[locked]**
- Art direction (post-greybox): hand-drawn pencil + liner + one red, vintage poster palette, paper-cutout assets with a tiny edge shadow; craters drawn directly on the "canvas". AI-generated assets for the MVP. **[locked]**
- Working title: **Flussbunker**. Crate/repo rename is a TODO for when implementation resumes. **[locked]**

---

## 9. Platforms, input, UI

- Targets: desktop first, then web (WASM, for playtests), then iPad, then phones. **[locked]**
- Input is built around *intents* (Fire, StartWave, ToggleOverlay, …), not keys, so touch is a mapping later. **[locked]**
- Space = fire torpedoes, Enter = start wave, Tab = **toggle** tactical overlay (setting persisted). **[locked]**
  - Why toggle: hold = "peek" (scoreboards), toggle = "mode you work in" (Factorio alt-mode). This overlay is a mode.
  - Web note: check `Window::prevent_default_event_handling` so Tab does not leave the canvas.
- Radial build/upgrade menu opens on **click** of a build spot (not right-click-hold; hold-drag fails on trackpads and touch). **[locked]**
- Minimal UI by default; hover still reveals context (turret range etc.) so the minimal-UI player is never blind. **[locked]**
- Onboarding: on by default, skippable at any time, choice remembered. Wave 1 teaches one thing. **[locked]**

---

## 10. Engineering conventions

- Language: everything in the repo in English (code, comments, commits, README, design docs). German only for in-game unit names / flavour. **[locked]**
- Commits: Conventional Commits with scopes (`feat(uboat): …`, `docs: …`). **[locked]**
- Git: branch `main`, commit straight to main, remote on GitHub. CI (`fmt`, `clippy`, `test`) later — TODO. **[locked]**
- Tracking: design lives in `docs/` (durable); bugs/tasks/ideas live in GitHub Issues with labels; a Project board with milestones = greybox milestones. `IDEAS.md`/`BUGS.md` dissolve into those. **[locked]**
- Bevy version stays pinned. **The user decides migrations; the AI never proposes one unprompted.** **[locked]**
- Comments: a written convention comes *first*; only then are the PoC's essay comments stripped. **[locked]**
- Existing docs: keep what is useful, drop the rest (git remembers). **[locked]**
- PoC code is a usable base, not sacred. Architecture follows the feature-plugin layout; design evolves (Ousterhout). **[locked]**
- Small mechanics may be prototyped as isolated `examples/` "Lego bricks". **[locked]**

---

## 11. Rejected ideas (so they are not re-proposed)

| Idea | Why rejected |
|---|---|
| Mortar / AoE turret | Flak covers the swarm role |
| XP, ranks, veterancy | Kill stealing, currency clutter |
| Debris as its own currency | Currency clutter |
| Right-click-hold radial menu | Fails on trackpad/touch |
| Tab hold-to-show | Overlay is a mode, not a peek |
| Hit a sinking ship for a bonus explosion | Overkill/targeting conflict, felt gimmicky |
| Turret scatter / misses | Output randomness that takes from the player |
| Friendly fire | Frustration without decision value |
| Auto-fire as a worse optional mode | Punishes exactly the players who need it |
| U-boat reverses instead of turning | Would make firing downstream at leakers impossible |
| Ships stop in front of a surfaced U-boat | Turns the boat into a barracks; dominant strategy |
| More than one U-boat | Attention budget |
| Stern tube mechanic | Overkill |
| Debris salvage / click-to-collect | Collecting chores (Fallout Shelter style) are disliked; AoE already rewards over-destruction |
| Enemies damaging turrets | Adds repair economy and a second thing to watch |
| Turret type-priority targeting | Standard modes are enough and fit a radial menu |

---

## 12. Open questions (next planning rounds)

- Economy: starting money, income curve, model prices, wave-clear bonus.
- Input & UI: full intent list, radial menu contents, overlay contents, touch mapping.
- Onboarding script for wave 1–3.
- Playtest protocol and telemetry.
