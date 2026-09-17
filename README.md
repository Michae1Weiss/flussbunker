# Flussbunker

A 2D tower defense in Bevy 0.19. Enemy ships come up a river toward your bunker; you hold the banks
with shore batteries and personally command one U-boat against everything the guns cannot kill.

Working title. Fictional setting, Kriegsmarine-era optics, cartoon tone. See `docs/VISION.md`.

## Status

**Planning done, greybox next.** A playable proof of concept exists in `src/` (coloured
rectangles, 12 waves). It predates the design in `docs/` and will be reshaped milestone by
milestone (`docs/ROADMAP.md`). Expect the code and the docs to disagree until then; the docs win.

```bash
cargo run                    # play the proof of concept
cargo run --features dev     # + FPS log and schedule ambiguity warnings
cargo test
```

PoC controls: `1`/`2`/`3` pick a gun · left-click land to build · left-click water to send the
boat · `T` dive · `Space` next wave. These change in the greybox (`docs/design/ui-input.md`).

## Read in this order

1. `docs/VISION.md` — what the game is, four pillars, non-goals
2. `docs/design/` — the rules of each system, with edge cases
3. `docs/DECISIONS.md` — why, rejected ideas, hypotheses
4. `docs/ROADMAP.md` — milestones with kill criteria
5. `CONTRIBUTING.md`, `docs/COMMENTS.md` — conventions
6. `src/` — start at `game_plugin.rs`
