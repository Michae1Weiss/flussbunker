# Flussbunker

A 2D tower defense in Bevy 0.19. Enemy ships come up a river toward your bunker; you hold the banks
with shore batteries and personally command one U-boat against everything the guns cannot kill.

Working title. Fictional setting, Kriegsmarine-era optics, cartoon tone. See `docs/VISION.md`.

**Play in the browser: https://michae1weiss.github.io/flussbunker/** (mouse and keyboard; click
the page once so it gets focus and sound)

## Status

**Greybox, roadmap M5 done.** A full run is playable with coloured rectangles: prologue, twelve
waves, three turret roles with upgrades, three boat models, destruction, onboarding notes and a
telemetry CSV per run (`telemetry/`, not committed). Milestones and kill criteria: `docs/ROADMAP.md`.

```bash
cargo run                    # play
cargo run --features dev     # + FPS log and schedule ambiguity warnings
cargo test
```

Art goes into `assets/` as listed in `docs/ART.md`; missing files stay rectangles.

Controls: left-click a build spot or turret for the radial menu, left-click the river to move the
boat, the docked boat for the shipyard. Right-click dives. Space fires (hold: salvo). Enter starts
the next wave, F toggles 2x pace, Esc closes a menu. Feel harnesses: `cargo run --example boat`,
`cargo run --example turrets`.

## Build for web

**Pre-requirement**: [bevy_cli](https://github.com/TheBevyFlock/bevy_cli); the `wasm32` target
comes with the toolchain file.
```bash
bevy build --yes --release web --bundle
# Test locally
python3 -m http.server 8000 --directory target/bevy_web/web-release/flussbunker
```

The page is `web/index.html` (paper loading screen, audio unlock on first input). Textures must be
8 bits per channel (WebGL2); `tools/art_fit.py` writes them so and a test guards it.

## Deploy to GitHub Pages

The site is served from the `gh-pages` branch (root, with a `.nojekyll` file).
```bash
# once: check out gh-pages next to this repo
git worktree add ../flussbunker-gh-pages gh-pages
# every deploy
SRC=target/bevy_web/web-release/flussbunker
DEST=../flussbunker-gh-pages
( cd "$DEST" && find . -mindepth 1 -maxdepth 1 ! -name .git ! -name .nojekyll -exec rm -rf {} + )
cp -r "$SRC"/. "$DEST"/
( cd "$DEST" && git add -A && git commit -m "ops: deploy web build" && git push )
```

## Read in this order

1. `docs/VISION.md` — what the game is, four pillars, non-goals
2. `docs/design/` — the rules of each system, with edge cases
3. `docs/DECISIONS.md` — why, rejected ideas, hypotheses
4. `docs/ROADMAP.md` — milestones with kill criteria
5. `CONTRIBUTING.md`, `docs/COMMENTS.md` — conventions
6. `src/` — start at `game_plugin.rs`
