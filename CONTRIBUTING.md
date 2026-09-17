# Conventions

Solo project with AI assistance. These rules exist so that code written on different evenings, by a
person or by an assistant, reads as one codebase.

## Language

Everything in the repo is English: identifiers, comments, commits, docs, README. German appears
only in-game as unit and flavour names (8.8 cm, Reichsmark, wave names).

## Git

- Branch `main`, commits straight to `main`. Feature branches only for experiments you may throw away.
- **Conventional Commits** with a scope: `feat(uboat): …`, `fix(tower): …`, `docs: …`,
  `refactor(wave): …`, `chore: …`, `balance: …`. Subject in imperative mood, ≤ 72 chars, no period.
  Body explains *why* when the diff does not.
- One logical change per commit. `git log --stat` on a feature commit touches one or two files.
- No history rewriting on `main`.

## Tracking

Design rules live in `docs/design/`. Bugs, tasks and ideas live in GitHub Issues with labels
`bug`, `idea`, `chore`, `design`; a project board with columns Backlog / Next / Doing / Done;
milestones mirror `docs/ROADMAP.md`. A rule change gets a line in `docs/DECISIONS.md`.

## Code

- Bevy 0.19, pinned. A migration is a separate decision by the maintainer, never proposed unprompted.
- One plugin per feature file, `pub(super) fn plugin(app: &mut App)`. Layout and boundaries in
  `docs/design/architecture.md`.
- Tuning numbers only in `balance.rs` (later RON). Never a literal in a system.
- Gameplay in `FixedUpdate`, visuals and input sampling in `Update`. Ordering declared once.
- Dev tooling behind `#[cfg(feature = "dev")]`.
- New crates need a fired trigger from the non-decisions table in `architecture.md`.
- `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test` pass before a commit. CI later.

## Comments

`docs/COMMENTS.md`. Short version: a comment says what the code next to it cannot; game-design
"why" goes to `docs/design/`, code-design "why" goes in the comment; no history in code.

## Definition of done for a milestone

The deliverable runs, the kill criterion from `docs/ROADMAP.md` was checked and the result noted in
`docs/DECISIONS.md`, and the docs describe the game as it now is.
