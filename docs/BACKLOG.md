# Backlog

Temporary home until the GitHub remote exists (roadmap M0); then every line becomes an issue with a
label and this file is deleted.

## Bugs in the PoC (label: bug)

- Torpedoes pass through obstacles when a ship is in line with the boat. Absorbed by the new lane
  rule (torpedo hits the bank at the next bend), verify after M1.
- Boat keeps the previous tile's rotation in corners. Replaced by the heading rule (`uboat.md`).
- Ships can pass through each other. Replaced by the convoy rule (`enemies.md`).
- Top rank unreachable in 12 waves. Ranks are removed.

## Post-2026 ideas (label: idea)

- Buying/unlocking build spots
- Two-lane river with overtaking
- Ships shelling the bunker as a last stand
- Repair integrity for RM
- Debris salvage, if a mechanic gives it a purpose
- Boat equipment upgrades; branching boat tree; Type XXI underwater hunter
- Auto-fire baseline (only if the playtest hypothesis fires)
- Second map; endless mode
- Touch mapping for iPad and phones
- CI: `cargo fmt --check`, `clippy -D warnings`, `cargo test`, dead-anchor check for `docs/` links

## Chores (label: chore)

- Rename crate `valentin` → `flussbunker`
- Move `balance.rs` to a RON asset once numbers churn
- Strip PoC comments per `docs/COMMENTS.md`
