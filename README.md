# Bunker Valentin

A 2D tower defense in Bevy 0.19.1. British boats run up the Weser; you hold the
banks with shore batteries and send u-boats out to meet what the guns cannot
kill.

Read in this order: **`DESIGN.md`** (why this game), **`ARCHITECTURE.md`**
(why this code shape), then `src/`.

```bash
cargo run                    # play
cargo run --features dev     # + FPS log and schedule ambiguity warnings
cargo test                   # damage maths and map parsing, headless
```

Controls: `1`/`2` pick a battery · left-click land to build · `U` orders a
u-boat · left-click a river tile to send a docked u-boat there · `Space` starts
the next wave.

---

## ⚠️ Not compiled

There is no Rust toolchain where this was written, so **nothing here has been
run through `cargo check`.** Expect a handful of fixes on first build. The
architecture and the design are the deliverable; the compiler will handle the
rest.

APIs I could not verify against docs.rs, in rough order of risk — check these
first if the build fails:

| API | Where | Note |
|---|---|---|
| `gizmos.circle_2d(Isometry2d::from_translation(p), r, color)` | `orders.rs` | Signature took `Isometry2d` as of 0.15; confirm for 0.19 |
| `gizmos.rect_2d(..)`, `gizmos.line_2d(..)` | `orders.rs`, `hud.rs` | Same family |
| `Color::with_alpha` / `set_alpha` | `orders.rs`, `projectile.rs` | Needs the `Alpha` trait in scope — `use bevy::color::Alpha;` if it is not in the prelude |
| `Single<&mut Text, With<..>>` then `status.0 = ..` | `hud.rs` | Field access through two `Deref`s; `(**status).0` if it complains |
| `ScheduleBuildSettings` / `LogLevel` path | `dev_tools.rs` | Module path moves between releases |
| `#[derive(EntityEvent)]` + `#[event_target]` | `combat.rs` | 0.17+ form |
| `add_message` / `MessageReader` / `MessageWriter` | everywhere | 0.17 rename from `add_event`/`EventReader`/`EventWriter` |
| `DespawnOnExit(..)` | everywhere | 0.19 name for `StateScoped` |
| `viewport_to_world_2d` returning `Result` | `orders.rs` | 0.19 change |

Everything from `EventReader` onwards is exactly where stale tutorials will
lead you astray — if you copy a snippet from a blog post and it uses
`EventWriter` or `SpriteBundle`, it is pre-0.17 and wrong.

---

## File map

```
src/
├── main.rs             window + GamePlugin, nothing else
├── game_plugin.rs      composition root; the ONE place ordering is declared
├── balance.rs          every tuning number, engine-free
├── dev_tools.rs        #[cfg(feature = "dev")]
├── screens/            title · game_over          (state flow, DespawnOnExit)
└── game/
    ├── map.rs          ASCII level, grid↔world maths, river walk   + tests
    ├── combat.rs       Health · Armor · Damage · Sunk              + tests
    ├── enemy.rs        boats: movement, breaching, depth charges
    ├── tower.rs        batteries: construction, targeting, firing
    ├── projectile.rs   shells and torpedoes in flight
    ├── uboat.rs        the pen, sorties, torpedoes, rearming
    ├── wave.rs         pacing, spawning, victory
    ├── economy.rs      marks and bounties
    ├── bunker.rs       integrity, defeat
    ├── orders.rs       the only module that reads input
    └── hud.rs          status line, health pips
```

Feature-based, never layer-based. `git log --stat` on a feature branch should
touch one or two files, not eleven.

---

## Where to go next, and when

Do not do these in parallel. Each stage has a threshold, and the threshold is
the point — it is what stops you polishing a loop that is not fun yet.

**Stage 1 — play it.**
Twenty minutes, cold. Are the five waves interesting *twice*? If not, change
the numbers in `balance.rs`, not the code. Suspects: MTB speed, Flak reload,
starting marks, destroyer armour.

*Threshold: two minutes of graybox is genuinely engaging.* Nothing below this
line matters until that is true.

**Stage 2 — feel.**
Juice list in `DESIGN.md` §5, top three only: hit flash, money tween, firing
sound. Then add `bevy_transform_interpolation` if 64 Hz movement looks choppy.

*Threshold: a new player can tell why they lost a wave without asking you.*

**Stage 3 — data.**
Move `balance.rs` to `assets/balance.ron`. The systems should not change at
all; if they do, they were reading literals somewhere.

*Threshold: you can rebalance an entire wave without recompiling.*

**Stage 4 — variety, then stop.**
One more enemy that demands a third answer (air attack is the classic: towers
cannot reach it, u-boats cannot either — so a third tool must exist). Branching
upgrades. Then **finish it**. Everything else goes in the backlog for game #2.

*Threshold: it is shippable. Ship it.*

---

## Six exercises

Small enough to finish in a sitting, each teaching one thing the code above
already sets up:

1. **Add a `Searchlight` support tower** that gives neighbouring batteries +25%
   range. Where does the aura live — a component on the tower, or a system that
   recomputes `Range`? (Try both; the second is much easier to get wrong.)
2. **Make targeting a per-tower choice** — First / Closest / Strongest — as a
   component, cycled by right-clicking the battery. This is one `max_by_key`
   becoming a match, and it is the single best gameplay-per-line change available.
3. **Sell a tower** for 60% of cost. Watch what it does to the projectiles that
   are already in the air targeting through it.
4. **Write a test for wave pacing**: assert that wave 3 puts exactly twelve
   boats in the water and that the corvettes arrive before the MTBs.
5. **Replace the money counter with a tween.** A `DisplayedMarks(f32)` resource
   lerping toward `Treasury.marks` — three lines, and the game immediately feels
   more expensive.
6. **Break the map on purpose** — put a `~` in the middle of the land — and read
   the panic. That message is the map parser's real interface.
