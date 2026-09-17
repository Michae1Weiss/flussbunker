# Economy

## Rules

- **Reichsmark (RM) is the only currency.** No XP, ranks, veterancy or debris currency.
  Why: no kill-stealing between turret and boat, no currency clutter in a minimal UI, and an
  all-in-on-the-boat playstyle becomes possible.
- RM is paid at the moment of the kill, shown as a brief transaction on the paper note, never as a
  permanent panel.
- A small fixed **wave-clear bonus**, independent of kills. A badly played wave still funds a
  correction (anti death-spiral).
- **Target:** a perfect run can afford ~**60 %** of "everything built and maxed". Every run leaves
  things unbought. May be tightened to 50 % later.
- A fully upgraded boat ≈ half the defence budget ≈ 3–4 maxed turrets.
- Selling at 60 % to re-tool before a telegraphed wave is intended play.
- **Starting money: (almost) none.** The first income is the prologue bounty (`enemies.md`),
  sized for one real decision: two flak, or one flak plus savings toward the next boat model.
- Boat models unlock by wave reached, are bought at the docked boat. Repair bill after a loss
  ~30 % of the boat's value (`uboat.md`).
- No bunker upgrade. Integrity is the loss counter, not a stat to buy.

## Sinks and sources

| Source | Sink |
|---|---|
| kill bounty by class | turret build (3 prices) |
| wave-clear bonus | turret upgrades (2 per turret) |
| prologue bounty | boat models (2 purchases) |
| sell refund (60 %) | boat repair |

## Numbers

All numbers are round-3 balance work, derived from the 60 % rule in a spreadsheet model before
they go into `balance.rs`. Nothing in this file is a number for that reason.

## Backlog

- Repair integrity for RM as a deliberate decision.
- Debris salvage, if a later mechanic gives it a purpose.
- Equipment upgrades for the boat, once there are more waves.
