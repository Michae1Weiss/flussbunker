# Architecture

## 1. Five architectures, honestly compared

Same game, five ways to organise it. The first two are the ones people
actually reach for after coming from OOP.

### A — Layer-based: `components.rs`, `systems.rs`, `resources.rs`

The most common beginner layout, and the worst. Adding "u-boats take damage
from destroyers" touches three files, none of which is about u-boats. Every
feature is smeared across every file, so nothing can be read, moved or deleted
as a unit. **Rejected.** The name of a file should say what it is *for*, not
what kind of Rust item is inside it.

### B — Manager objects: `TowerManager`, `WaveManager`, `EnemyManager`

The OOP instinct: an object that owns a `Vec<Tower>` and a tick method. It
works, and in Bevy it fights the engine the entire way — you lose parallel
scheduling, change detection, the inspector, and the ability to query across
types. A "manager" in ECS is a resource holding `HashMap<Entity, T>`, which is
the canonical smell: **that data belongs on the entity.** **Rejected.**

### C — Plugin per feature ← **chosen**

One file per feature, each exporting `pub(super) fn plugin(app: &mut App)`.
Eleven plugins, each owning its components, its resources, its messages and its
systems. Interface: one line. Implementation: everything else.

This is Ousterhout's *deep module* translated into Bevy: `uboat::plugin` is
~200 lines behind a one-line interface. It is also the layout the official
`bevy_new_2d` template uses, so it will look familiar to other Bevy developers
— which is itself a real benefit.

### D — Plugin per *thing*

`TowerRangePlugin`, `ProjectileMovementPlugin`, `HealthBarPlugin`. The Bevy
form of classitis. Twenty plugins whose interfaces cost as much as their
implementations, and a composition root nobody can read. **Rejected** — but
note the boundary: `screens/game_over.rs` handles both victory *and* defeat in
one file precisely because splitting them would land here.

### E — Fully data-driven from day one

Towers, enemies and waves in RON, systems as generic interpreters. This is
where the project *should* end up, and starting there is still wrong: an asset
loader, a hot-reload path and a schema, before anyone knows whether the game is
fun. `balance.rs` is const, type-checked, zero-infrastructure and one file.

**The rule: centralise now, externalise when the trigger fires.** The trigger is
"I am changing numbers more often than code". Then `balance.rs` becomes
`assets/balance.ron` and the systems do not change, because they already read a
table instead of literals.

---

## 2. The mental model: how plugins talk

This is the part that does not translate from OOP, so here it is directly.

In OOP a module boundary is **who owns the data**, and communication is one
object calling another's method. In ECS nobody owns data — it is all in the
world, and any system could read any of it. So the boundary has to be a
different thing:

> **A module boundary in ECS is: who is allowed to *write* a component.
> Reading is public. Writing is private.**

`hud.rs` reads `Treasury`, `Integrity`, `UBoatPen` and `WaveState` — four other
features' resources — and that is fine, because it writes none of them. If two
modules write the same component, they are one module wearing two names.

Plugins never call each other. There are exactly three shared vocabularies:

| Vocabulary | Use when | Example here |
|---|---|---|
| **Component** | Per-entity data. Anyone may read; one feature writes | `Health` written only by `combat.rs` |
| **Message** | Buffered, decoupled, ordered, possibly many writers | `Damage`, `EnemyReachedBunker`, `BuildTower` |
| **Event + observer** | Immediate, entity-targeted, must run before the entity is gone | `Sunk` |
| **Resource** | Durable global state | `Treasury`, `Integrity`, `UBoatPen` |

And one rule for direction: **the producer owns the message type.**
`enemy.rs` defines `EnemyReachedBunker`; `bunker.rs` imports it. So the arrow
points bunker → enemy, and enemies know nothing about bunkers. If two features
ever both produce *and* consume the same signal, that is when a shared
`contract.rs` becomes the better answer — not before.

Trace one interaction end to end to see the whole model:

```
orders.rs      mouse click on a river tile
   │           writes LaunchUBoat { station }       (Update, Frame::Input)
   ▼
uboat.rs       reads it, checks the pen, spawns a U-boat   (FixedUpdate, Sim::Spawn)
   │           writes a Projectile when in range            (Sim::Targeting)
   ▼
projectile.rs  flies, hits, writes Damage { target, Torpedo }  (Sim::Movement)
   ▼
combat.rs      applies armour maths, writes Health           (Sim::Damage)
   │           health <= 0 → triggers Sunk, despawns          (Sim::Death)
   ▼
economy.rs     observer reads Bounty, adds marks             (immediate)
```

Five modules, no function calls between them, no shared mutable state, and each
step is independently testable. Note that the chain is also *ordered* — which is
the other half of the model.

---

## 3. Ordering: declared once, in `game_plugin.rs`

```rust
(Sim::Spawn, Sim::Movement, Sim::Targeting, Sim::Damage, Sim::Death).chain()
```

Every gameplay system picks a set; no system anywhere says `.after(...)`.
The reason is that ordering is a *global* property: expressed as pairwise
constraints across eleven files it becomes a graph nobody can hold in their
head, and the bugs it produces ("damage applied one tick late, only sometimes")
are the worst kind.

The same `configure_sets` call also carries `.run_if(in_state(Screen::Gameplay))`,
so gameplay gating is written once instead of thirty times — and cannot be
forgotten on system number thirty-one.

Two schedules, and the split is not stylistic:

- **`FixedUpdate` (64 Hz)** — movement, targeting, damage. Deterministic, frame-rate independent, identical on a 60 Hz laptop and a 240 Hz monitor.
- **`Update`** — input sampling, HUD, gizmos. Anything whose job is "what is on screen right now".

The trap this avoids: `ButtonInput::just_pressed` flips once per *frame*, while
`FixedUpdate` may run twice or zero times in that frame. Reading input there
silently drops or doubles clicks. The fix is exactly what `orders.rs` does —
sample in `Update`, buffer the intent as a message, consume it in `FixedUpdate`.

---

## 4. Errors defined out of existence

Ousterhout's principle, applied concretely. Almost nothing in this codebase
returns `Result`, and that is by design rather than by neglect:

| Situation | Handled by |
|---|---|
| Two shells hit an already-despawned boat | `let Ok(..) = query.get_mut(..) else { continue }` |
| No camera or no window yet | `Single<..>` — the system is skipped, not errored |
| Integrity would go below zero | `saturating_sub` (an unsigned wrap would make the game unlosable) |
| Armour exceeds shell damage | 10% floor in the damage formula |
| Malformed level file | `expect` at startup — a broken level is a programmer error, and the earliest loud failure is the cheapest one |
| U-boat sunk while ordered home | The pen slot is freed by an observer on `Sunk`; no bookkeeping to forget |

---

## 5. Deliberate non-decisions

Things not adopted, with the trigger that would change the answer:

| Not used | Would use it when |
|---|---|
| A physics engine (avian / rapier) | Rigid-body juice: debris, wreckage, physical torpedoes. Range checks are `distance_squared` — a constraint solver adds tuning surface and determinism headaches for nothing |
| Spatial grid / broadphase | A profiler shows the O(towers × boats) loop mattering. At ~100 entities it does not |
| `bevy_transform_interpolation` | 64 Hz stepping becomes visibly choppy. Slow boats hide it; add the crate, do not move movement to `Update` |
| An input manager (leafwing / bevy_enhanced_input) | Rebindable controls or input contexts are needed |
| `bevy_hanabi` | Hand-rolled sprite particles genuinely are not enough |
| A tilemap crate | 144 sprites is not a performance problem |
| Sprite picking backend | Grid maths from the cursor is simpler here *and* teaches the coordinate conversion the game needs anyway |
| Asset loading / `bevy_asset_loader` | There are assets. There are currently none — every visual is a coloured rectangle |

Each of these is a dependency added *before* its trigger fires, which is
complexity bought with nothing.

---

## 6. What is genuinely contested

Do not take these as settled; they are not.

- **`Message` vs `Event` naming** is disputed among Bevy contributors (see bevyengine/bevy discussion #21492). What matters is the semantics — buffered-and-ordered versus immediate-and-targeted — not which word is nicer.
- **Observers vs systems + messages for gameplay logic** is an open debate. Observers are immediate and entity-targeted but **cannot be explicitly ordered**, and control flow jumps around, which is harder to trace. This codebase uses exactly two observers (bounty, pen slot) and both are for the same reason: they must run *before* the entity is despawned.
- **Whether small games need screen states at all.** Flat is defensible for jam-sized games. States are used here for one concrete payoff — `DespawnOnExit` makes restart-cleanup impossible to get wrong — not because they are mandatory.
