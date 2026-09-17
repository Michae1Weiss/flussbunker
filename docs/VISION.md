# Flussbunker — vision

Working title. A 2D tower defense on a river: you hold the banks with shore batteries and personally
command one U-boat against waves of ships heading for your bunker.

## Pitch

> Read the river, build emplacements at the bends, take the boat out onto the straights, put
> torpedoes into whatever the guns cannot kill, and watch it come apart.

## Pillars

Every idea is checked against these four. If it serves none, it goes to the backlog.

1. **The U-boat is the game.** Heavy ships can only be sunk by torpedoes; a run without the boat is
   impossible by design. Turrets support, the boat decides.
2. **Destruction is the reward.** Every kill is a readable little show, and how a ship dies depends
   on what killed it.
3. **Never punish the player's hand.** One manual weapon, interaction is never blocked, randomness
   only ever gives.
4. **Paper and pencil.** Diegetic, minimal UI; hand-drawn paper-cutout look (after greyboxing).

## The feeling after 30 minutes

Spectacle and chaos, challenge, discovery (new waves, enemies, upgrades). MDA framework vocabulary.

Reference points, one thing each:

| Game | The one thing |
|---|---|
| Silent Hunter | firing a torpedo, the ticking wait, the water column |
| Toy Defense 2 | the feel of upgrades |
| Storm the House 3 | passive defence plus one proactive weapon you fire yourself |
| Kingdom Rush | fixed build spots, readable lanes |
| Bodycam | presentation variety: many animations for the same action |

## Not this

No RTS. No idle TD. No micromanagement stress. No realism. No multi-session runs: one run is one
sitting, ~15 minutes in the MVP, up to ~30 later.

## Run structure

One map, a prologue plus 12 fixed waves, no meta-progression, every run a fresh start. Lose when
bunker integrity reaches 0, win by surviving all waves. Waves never auto-start. Pause and 2× speed
exist; building during a wave is allowed.

## Theme and tone

Fictional setting, fictional names, as little story as possible. Kriegsmarine-era optics, real
weapon and U-boat class names, Reichsmark as currency. **No NS symbols, no references to the
regime, no real place names.** Tone: playful, cartoonish, never satire.

Art direction, post-greybox: pencil, liner and one red on paper, vintage poster palette. Assets look
cut out and laid on the paper with a tiny edge shadow; craters and wrecks are drawn directly on the
canvas. AI-generated assets for the MVP, hand-drawn later.

## Scope and budget

Solo developer with AI assistance, ~8–10 h/week until end of 2026, roughly 130–150 h. That buys a
greybox of every mechanic, a first playtest round, AI placeholder art and a web build. It does not
buy hand-drawn art. The roadmap respects that.

## Where things are

| Document | Holds |
|---|---|
| `docs/design/*.md` | the rules of each system, edge cases, hypotheses |
| `docs/DECISIONS.md` | why, rejected ideas, hypotheses to test |
| `docs/ROADMAP.md` | greybox milestones with kill criteria |
| `docs/PLAYTEST.md` | protocol, telemetry, success thresholds |
| `docs/BACKLOG.md` | ideas and known bugs, until they become GitHub issues |
| `CONTRIBUTING.md`, `docs/COMMENTS.md` | conventions |
