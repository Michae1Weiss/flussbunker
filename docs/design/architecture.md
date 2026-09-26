# Architecture

Bevy 0.19, pinned. The layout follows the `bevy_new_2d` template and the bevy-gamedev skill's
conventions; Ousterhout's *deep module* is the plugin.

## Layout

```
src/
├── main.rs             window + GamePlugin, nothing else
├── game_plugin.rs      composition root; the ONE place ordering is declared
├── balance.rs          every tuning number, engine-free (RON asset once numbers churn)
├── settings.rs         what the player sets: volumes, hints
├── dev_tools.rs        #[cfg(feature = "dev")]
├── screens/            `Screen` and `Menu` states, DespawnOnExit: title (main menu) · pause ·
│                       settings · game_over · widget (paper card, button)
└── game/               one plugin per feature: map, uboat, tower, enemy, projectile,
                        combat, destruction, wave, economy, bunker, input, hud, pace,
                        notes, cursor, audio, art, telemetry, ...
examples/               feel harnesses: a minimal App around ONE crate plugin, runnable on its own
```

Feature-based, never layer-based. A feature branch touches one or two files.

## Module boundary

> A module boundary in ECS is **who may write a component**. Reading is public, writing is private.

If two plugins write the same component, they are one plugin. Plugins never call each other. The
shared vocabularies:

| Vocabulary | Use when | Example |
|---|---|---|
| Component | per-entity data; anyone reads, one feature writes | `Health` written only by `combat` |
| Message | buffered, ordered, decoupled, possibly many writers | `Damage`, `BuildTower`, `FireTorpedo` |
| Event + observer | immediate, entity-targeted, must run before a despawn | `Sunk` |
| Resource | durable global state | `Treasury`, `Integrity` |

The producer owns the message type; consumers import it. A signal that two features both produce
and consume moves to a shared `contract.rs` — not before.

## Ordering

One `SystemSet` chain declared once in `game_plugin.rs` (`Spawn → Movement → Targeting → Damage →
Death`), gated once with `run_if(in_state(Screen::Gameplay))`; input sampling additionally only
under `Menu::None`. No system anywhere says `.after()`.
The schedule rationale is commented there and nowhere else (`docs/COMMENTS.md` §5).

- `FixedUpdate`: movement, targeting, damage, chains. Deterministic, frame-rate independent.
- `Update`: input sampling, HUD, gizmos, animation. Input is sampled in `Update`, buffered as a
  message, consumed in `FixedUpdate`.

## Errors defined out of existence

`let Ok(..) = q.get_mut(..) else { continue }` for despawned targets · `Single` skips instead of
erroring · `saturating_sub` on integrity · 10 % armour floor · `expect` at startup for a malformed
map · pen slot freed by an observer on `Sunk`.

## Input

`input.rs` is the only module that reads devices. It maps keys and clicks to **intents**
(`docs/design/ui-input.md`) and writes one message per intent (`FireTorpedo`, `MoveBoat`, …).
Touch later is a second mapping in the same file. The PoC calls this module `orders.rs`; it is
renamed at M0 so that module, vocabulary and Bevy's own `ButtonInput` all say the same word.

## Feel harnesses in `examples/`

Some questions can only be answered by hand: does the radial menu open smoothly, does turning
feel right, is the chain explosion readable. For those, `examples/<feature>.rs` builds a minimal
`App` with dummy world data around the **crate's own plugin** — never a copy of it. This needs a
`lib.rs` plus a thin `main.rs` (the `bevy_new_2d` layout). Each harness answers one question and
is listed in its milestone; CI builds them (`cargo build --examples`) so they cannot rot.
Integration into the running game is a separate step with its own check. Numbers are never tuned
in a harness; they come from the balance model and the headless tests.

MVP harnesses: boat movement + turning + dive, torpedo lane + bank hit, ramming (M1); radial menu
(M2); chain explosion + debris arcs (M4).

## Telemetry

A `telemetry` plugin observes messages and events and writes one CSV per run to a local directory.
It never writes game state and never uploads (`docs/PLAYTEST.md`).

## Non-decisions, with the trigger that changes them

| Not used | Would use it when |
|---|---|
| physics engine (avian / rapier) | debris and wreck halves need real bodies; until then, hand-rolled arcs |
| spatial grid | a profiler shows the O(turrets × ships) loop mattering |
| `bevy_transform_interpolation` | 64 Hz stepping is visibly choppy |
| input manager crate | rebindable controls are needed |
| `bevy_hanabi` | sprite particles are not enough |
| tilemap crate | never, for one fixed map |
| `bevy_asset_loader` | there are assets |

Each is a dependency added before its trigger, which is complexity bought with nothing.

## Contested, not settled

`Message` vs `Event` naming (bevyengine/bevy #21492); observers vs systems+messages for gameplay
logic (observers cannot be ordered); whether a game this size needs screen states at all (kept for
`DespawnOnExit`).
