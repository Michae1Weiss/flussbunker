# Playtest protocol

## Setup

- Five testers, one at a time, on the tester's own machine or the author's, cold: no explanation
  beyond "it's a tower defense, the game will show you".
- Screen and voice recorded. Think-aloud: the tester may talk the whole time.
- **The observer says nothing.** Questions are answered with "what would you try?". Helping once
  invalidates the session for onboarding metrics.
- One run minimum; the offer of a second run is itself a metric.

## Telemetry

The greybox writes one CSV per run to a local directory. Nothing is uploaded.

| Field | Why |
|---|---|
| wave reached, won/lost | difficulty curve |
| RM at end, RM never spent | 60 % rule check |
| turrets built per type, upgrades bought, sells | build diversity, mono-build detection |
| torpedoes fired / hit / into the bank | lane understanding |
| time to first torpedo, time to first build, time to first dive | onboarding |
| boat sunk: count and cause (shelling, ramming) | fairness of loss |
| ram kills, chain kills, debris kills | pillar 2 in numbers |
| overlay on/off time share | is the minimal UI viable |
| session length, second run started | fun test |

## Success thresholds, playtest 1

- 5/5 fire a torpedo in the prologue without help
- 4/5 build a turret in wave 1 without help
- 3/5 dive in the first destroyer wave
- ≥ 3/5 want a second run
- no newcomer clears wave 12 on the first try (a cold first-run win means the curve is too flat;
  wave 12 must still be winnable on a second or third run)

A missed threshold is a redesign item before M8, not a balance tweak.

## After the session

Open questions only. Never "was it fun?".

- "What were you trying to do at [moment from the recording]?"
- "What did the boat do when you right-clicked?"
- "What did you spend your money on, and why?"
- "Which ship did you fear most? Why?"
- "What would you do differently in a second run?"

## Reading the recording

Watch for hesitation, repeated clicks on the same spot, and any sentence starting with "why did".
Those are the bugs. Cross-check against the CSV before changing a number.
