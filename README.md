# Flussbunker

A 2D tower defense in Bevy 0.19. Enemy ships come up a river toward your bunker; you hold the banks
with shore batteries and personally command one U-boat against everything the guns cannot kill.

Working title. Fictional setting, Kriegsmarine-era optics, cartoon tone. See `docs/VISION.md`.

## Status

**Greybox, roadmap M5 done.** A full run is playable with coloured rectangles: prologue, twelve
waves, three turret roles with upgrades, three boat models, destruction, onboarding notes and a
telemetry CSV per run (`telemetry/`, not committed). Milestones and kill criteria: `docs/ROADMAP.md`.

```bash
cargo run                    # play
cargo run --features dev     # + FPS log and schedule ambiguity warnings
cargo test
```

Web build (playtests by link): `cargo install trunk`, then `trunk serve` and open
http://127.0.0.1:8080; `trunk build` writes a static `dist/` for itch.io. The `wasm32` target
comes with the toolchain file. Art goes into `assets/` as listed in `docs/ART.md`; missing files
stay rectangles.

Controls: left-click a build spot or turret for the radial menu, left-click the river to move the
boat, the docked boat for the shipyard. Right-click dives. Space fires (hold: salvo). Enter starts
the next wave, F toggles 2x pace, Esc closes a menu. Feel harnesses: `cargo run --example boat`,
`cargo run --example turrets`.

## Read in this order

1. `docs/VISION.md` — what the game is, four pillars, non-goals
2. `docs/design/` — the rules of each system, with edge cases
3. `docs/DECISIONS.md` — why, rejected ideas, hypotheses
4. `docs/ROADMAP.md` — milestones with kill criteria
5. `CONTRIBUTING.md`, `docs/COMMENTS.md` — conventions
6. `src/` — start at `game_plugin.rs`
