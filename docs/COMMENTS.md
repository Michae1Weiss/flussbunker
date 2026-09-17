# Comment convention

Rust + Bevy. Based on Ousterhout, *A Philosophy of Software Design*, ch. 13–15, adapted for a
codebase largely written with an AI assistant. AI assistants err toward too many comments, so every
rule here is a **test**, not a taste.

## 0. The one rule

A comment states what the reader cannot get from the code next to it. Nothing else.

## 1. When a comment is mandatory

A declaration (`//!` module, `///` item or field) gets a comment **if and only if** at least one of
these has a non-empty answer:

- unit or time base: seconds, RM, world units, per `FixedUpdate` tick
- valid range, inclusive/exclusive bounds, what `None` / empty / `0` means
- invariant, and who is the sole writer
- side effect: despawns, commands applied at the next flush, message written
- ordering or schedule assumption
- `# Panics` / `# Errors`, only on real API surface, not on every `pub(super)`

Nothing to say → no comment. `/// The wave index.` on `index: usize` is a bug.
Every plugin file has a `//!`. `pub(super) fn plugin` needs no `///`; the `//!` is its doc.

## 2. Two kinds of "why", two homes

| Why the **game** has this rule | `docs/design/*.md` |
|---|---|
| Why the **code** is shaped like this: schedule, data shape, ordering, error handling | a comment |

Code may point at a rule: `// Rule: docs/design/uboat.md#dive`. One pointer per module, in the
`//!`, is the norm; a pointer inside a body only for a cross-module contract (Ousterhout 13.7).
The pointer never replaces the contract: the comment still says what the code does.
Targets are heading anchors of in-repo docs, never line numbers, never external documents.

## 3. Budget

- `//!`: at most 5 lines. What the plugin owns, what it is sole writer of, where its rules live.
- `///`: 1–3 lines. Longer means the interface is too complex or it is design text. Fix that instead.
- `//` inside a body: only where a reader would ask "why?". Never narrate steps.

## 4. Banned

- **History**: "was", "used to", "tightened from", "v1/v2", "playtest", "we", "you", or any
  description of a previous state. Git and `docs/DECISIONS.md` hold history.
- A comment that could be written from the line next to it, or that re-spells the identifier.
  Test: the comment must use different words than the name.
- Implementation detail in an interface comment. If the interface cannot be described without it,
  the module is shallow. Redesign, do not document.
- Commented-out code, banner separators (`// ==== SYSTEMS ====`), author or changelog notes,
  `// TODO` without an issue number, `/// Returns a bool.`

## 5. Bevy specifics

- Schedule and set rationale (`FixedUpdate` vs `Update`, set order) is written **once**, at the
  `SystemSet` definition in `game_plugin.rs`. A system comments only a deviation from that rule.
- `Message` vs `Event` + observer: state the reason only when it is the constraint itself
  (observers cannot be ordered; something must run before a despawn).
- `.run_if` that encodes a game rule: name the rule, not the predicate.
- Required components and relationships: name the invariant they make unrepresentable.
- Cleanup: say who despawns, unless `DespawnOnExit` does.
- Balance constants: comment only a **constraint** ("must be shorter than the longest straight",
  "below 0.3 boats overlap"). No history, no "feels right".

## 6. Before / after, from this codebase

Module header, 35 lines of design history →

```rust
//! The player's U-boat: river movement, dive state, torpedoes, rearming.
//! Sole writer of `Trim`, `Torpedoes`, `Station`.
//! Rules: docs/design/uboat.md
```

Enum with three lines on why the transition variants "are the cost of diving" →

```rust
/// Dive state. Transitions take `UBOAT_TRIM_SECONDS`; orders during a transition are ignored.
pub enum Trim { .. }
```

`/// Tightened from 260. You never lost, and a big opening float …` →

```rust
pub const STARTING_MARKS: u32 = 0;
```

Eight lines on why the kill tally lives in a resource, not on the entity →

```rust
/// Outlives the boat entity on purpose; reset by `on_boat_lost`.
pub struct Flotilla { .. }
```

## 7. Names

- Precise and evocative: `remaining_cooldown`, not `time`. Short scope → short name.
- One concept, one name, everywhere.
- Hard to name means the thing does two jobs. Split it.
- Tests are documentation: `a_branching_river_is_rejected` beats any comment. Name a test after the
  rule it guards.

## 8. Workflow

Write the `//!` before the first system and the `///` before the body. A long or awkward interface
comment means the design is wrong. Fix the design, not the sentence.

## 9. Review

Delete on: repeats code · history · game-design text in code · interface leaks implementation.
Comment and code disagree → bug. Non-obvious code without a comment → simplify first, comment second.
