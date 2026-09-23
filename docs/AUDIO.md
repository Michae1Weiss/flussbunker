# Audio: what to record, how long, how loud

Every sound the game plays, with the moment it plays, so each can be recorded or found on its own.
The audio layer works like the art layer: a file that exists under `assets/audio/` plays, a
missing one is silence. Deliver them in any order.

Tone, from `docs/VISION.md`: playful, a little cartoonish, never grim. Everything should sound
like a toy war on a kitchen table seen through a 1940s newsreel: real sounds, slightly
exaggerated, never gory, no screams, no voices.

## Rules for every file

1. **Format:** OGG Vorbis, quality 5 to 6, 48 kHz (44.1 is fine). Effects **mono**, music
   **stereo**.
2. **Level:** normalise every file to a **peak of -1 dBFS**. Do not make quiet sounds quiet in
   the file; the game sets each sound's loudness from the "Level" column. You only deliver
   clean, full-level files.
3. **Start:** cut the silence before the sound; the first transient within 5 ms. A late start
   feels like input lag.
4. **End:** let the tail ring out to silence; a 10 to 30 ms fade at the very end, never a cut.
5. **Loops** (marked "loop"): the end must flow into the start without a click; cut on zero
   crossings, test by looping in Audacity.
6. **Variants:** where the table asks for several, record several takes; name them
   `name_1.ogg`, `name_2.ogg` ... The game picks one at random and also shifts the pitch a little
   ("Pitch" column), so 3 takes sound like 20.
7. **Nothing from films, games or music** you do not have the rights to. Every file you did not
   record yourself gets a line in `assets/audio/CREDITS.md`: file, source URL, author, licence.

## Where to get sounds

- **Record them.** A phone in a quiet room, 20 to 30 cm away, several takes. Kitchen tricks are
  in the notes column.
- **Freesound.org** with the licence filter on **CC0** (no credit needed). CC-BY is fine too, but
  must go into `CREDITS.md`.
- **Kenney.nl** audio packs (CC0): "Impact Sounds", "Interface Sounds", "UI Audio".
- **Sonniss GDC Game Audio Bundle** (free yearly bundles, royalty-free in games, no credit
  needed; the licence forbids sharing the raw files outside the game, which a game does not do).
- **Audacity** (free) for trimming, normalising, fading, pitch shift and low-pass (for distance).
  Layering two sounds (a low boom plus a crackle) is how most game explosions are made.

## Weapons

| File | Plays when | Length | Variants | Level | Pitch | Character, how to get it |
|---|---|---|---|---|---|---|
| `sfx/flak_shot` | a Flak emplacement fires (every shot; tier 2 fires two) | 0.15-0.3 s | 3 | -14 dB | ±8 % | short dry "pock", automatic cannon, almost no tail. Knuckle on a cardboard box, pitched down |
| `sfx/gun88_shot` | an 8.8 emplacement fires | 0.4-0.7 s | 3 | -10 dB | ±6 % | sharp crack with a short tail. Book slapped flat on a table, low-passed tail |
| `sfx/heavy15_shot` | a 15 cm emplacement fires | 1.0-1.5 s | 2 | -6 dB | ±4 % | deep boom with rumble; rare, so it may be big. Balloon pop pitched down an octave plus reverb |
| `sfx/deck_gun` | the U-boat's deck gun fires | 0.4-0.6 s | 2 | -10 dB | ±6 % | lighter and more hollow than the 8.8, "on a ship" |
| `sfx/enemy_gun` | a destroyer fires at the boat | 0.6-1.0 s | 2 | -12 dB | ±6 % | the same kind of crack, muffled and further away (low-pass) |
| `sfx/ranging` | a destroyer starts ranging in on the surfaced boat (3 s before its shells can hit) | 1.5-2.5 s | 1 | -6 dB | none | a rising whistle or a rhythmic ping that says "dive now". Must be unmistakable, the most important warning in the game |
| `sfx/torpedo_launch` | the boat fires a torpedo | 0.6-1.0 s | 2 | -8 dB | ±5 % | compressed-air "thunk" then a small splash. Popping a cork plus a hand slapped into water |
| `sfx/torpedo_run` | loop, while at least one torpedo is running | 2-4 s, loop | 1 | -18 dB | none | a low propeller whir with bubbles, calm. One loop no matter how many torpedoes |

## Hits and destruction

| File | Plays when | Length | Variants | Level | Pitch | Character, how to get it |
|---|---|---|---|---|---|---|
| `sfx/hit` | a turret shell or deck-gun shell hits a ship | 0.1-0.25 s | 4 | -16 dB | ±10 % | metallic clank, a spoon on a pot. Very frequent: short and dry |
| `sfx/torpedo_hit` | a torpedo strikes a hull | 1.5-2.5 s | 2 | -3 dB | ±4 % | the payoff of the game: a deep underwater thump, a rising water column, a metal groan. Take time over this one |
| `sfx/explode_small` | a light ship (MTB) explodes | 0.8-1.2 s | 3 | -8 dB | ±8 % | a pop with crackle |
| `sfx/explode_medium` | a medium ship (corvette, landing craft) explodes or breaks | 1.2-2.0 s | 3 | -5 dB | ±6 % | a solid boom with debris |
| `sfx/explode_large` | a destroyer or monitor breaks, and every blast in a chain | 2.0-3.5 s | 2 | 0 dB | ±5 % | the loudest sound in the game: a long boom, rumble, debris raining down |
| `sfx/bank_hit` | a torpedo runs into the bank | 1.0-1.5 s | 2 | -6 dB | ±6 % | a dull explosion into earth, dirt falling. Duller than a ship hit: "wasted" |
| `sfx/splash` | a shell or a piece of debris lands in the water | 0.3-0.6 s | 3 | -16 dB | ±10 % | a plop and a short spray. A hand or a cup dropped into a filled sink |
| `sfx/debris_land` | debris lands on land | 0.2-0.4 s | 3 | -16 dB | ±10 % | a thud on dirt, a bag of rice dropped on the floor |
| `sfx/burn` | a ship dies by fire and drifts to the bank | 1.5-2.5 s | 2 | -14 dB | ±6 % | crackle of flames, a short "whoomp" at the start |
| `sfx/sink` | a wreck starts sinking | 2.0-3.0 s | 2 | -12 dB | ±6 % | hull groan, gurgling water |
| `sfx/breach` | a ship reaches the bunker and costs integrity | 1.0-1.5 s | 1 | -4 dB | none | a heavy impact on concrete plus a short alarm bell: something went wrong |

## The boat

| File | Plays when | Length | Variants | Level | Pitch | Character, how to get it |
|---|---|---|---|---|---|---|
| `sfx/dive` | the boat starts to dive | 1.5-2.5 s | 1 | -8 dB | none | a short dive klaxon, then air venting and bubbles. Blowing through a straw into a glass of water |
| `sfx/surface` | the boat starts to surface | 1.5-2.0 s | 1 | -10 dB | none | ballast blowing, water rushing off |
| `sfx/boat_hit` | the boat takes damage | 0.3-0.6 s | 2 | -8 dB | ±6 % | a hollow metal clang, clearly different from `hit` (that one is the enemy's) |
| `sfx/ram` | the boat rams a light ship | 0.5-1.0 s | 2 | -8 dB | ±6 % | crunch of wood and metal |
| `sfx/boat_lost` | the boat is sunk | 2.0-3.0 s | 1 | -4 dB | none | a deep implosion groan; sad, not horrible |

## Building and money

| File | Plays when | Length | Variants | Level | Pitch | Character, how to get it |
|---|---|---|---|---|---|---|
| `sfx/build` | construction starts on a spot | 0.5-1.0 s | 2 | -12 dB | ±6 % | sandbags thrown down, a shovel in dirt |
| `sfx/built` | construction is finished | 0.3-0.6 s | 1 | -10 dB | ±4 % | a metal clank and a bolt locking |
| `sfx/upgrade` | an emplacement is upgraded | 0.6-1.0 s | 1 | -10 dB | ±4 % | a ratchet and heavier metal |
| `sfx/sell` | an emplacement is sold | 0.5-0.8 s | 1 | -10 dB | ±4 % | coins poured into a hand |
| `sfx/too_expensive` | a click on something the treasury cannot pay | 0.2-0.4 s | 1 | -14 dB | none | a dull wooden knock, a friendly "no" |
| `sfx/coin` | bounty paid for a kill | 0.1-0.2 s | 3 | -20 dB | ±10 % | a single soft coin tick. Frequent: very quiet |

## Interface

| File | Plays when | Length | Variants | Level | Pitch | Character, how to get it |
|---|---|---|---|---|---|---|
| `sfx/ui_hover` | the cursor enters a radial-menu entry | 0.03-0.08 s | 3 | -22 dB | ±10 % | a pencil tapped on paper |
| `sfx/ui_click` | a radial-menu entry is chosen | 0.05-0.15 s | 2 | -16 dB | ±6 % | a rubber stamp on paper |
| `sfx/menu_open` | the radial menu opens | 0.15-0.3 s | 2 | -18 dB | ±6 % | paper sliding across a table |
| `sfx/menu_close` | the radial menu closes | 0.1-0.2 s | 1 | -20 dB | ±6 % | a shorter slide |
| `sfx/overlay` | the Tab map is opened or closed | 0.3-0.5 s | 1 | -16 dB | ±4 % | a map being unfolded |
| `sfx/pause` | pause on or off | 0.3 s | 1 | -14 dB | none | a page turned |

## Phase banners

These play with the big pop-ups between phases, so they carry the change of mode.

| File | Plays when | Length | Variants | Level | Character |
|---|---|---|---|---|---|
| `sfx/prologue` | the run starts with the gold transport | 2-3 s | 1 | -8 dB | a distant foghorn |
| `sfx/wave_start` | a wave starts | 1.5-2.5 s | 1 | -4 dB | a ship's horn or a bell struck twice, a drum hit under it |
| `sfx/wave_clear` | a wave is cleared | 1.5-2.5 s | 1 | -6 dB | a bright bell ring or a short brass sting, relief |
| `sfx/last_wave` | the last wave starts | 2 s | 1 | -6 dB | a short siren, more urgent than `wave_start` |
| `sfx/victory` | the run is won | 3-5 s | 1 | -4 dB | a short cheerful sting, not a march |
| `sfx/defeat` | the bunker falls | 3-4 s | 1 | -4 dB | a falling, deflating sting, a little comic |

## Music

| File | Plays when | Length | Level | Character |
|---|---|---|---|---|
| `music/build.ogg` | title, prologue and every build phase between waves; loop | 60-120 s, loop | -18 dB | calm, curious, a bit wistful: piano, accordion, plucked strings, light percussion |
| `music/wave.ogg` | while a wave runs; loop | 60-120 s, loop | -16 dB | the same key and tempo as `build.ogg`, with drums and tension; the game cross-fades between the two in 1 s |

No real period marches, anthems or military songs, including instrumental versions. They carry
the associations the game stays away from (`docs/VISION.md`, theme and tone), and most are not
free to use anyway. Newly written music "in the style of" old newsreels is fine.

## In the game

Mute and unmute with `M`. Sounds that fire often (Flak, `hit`, `coin`) are limited to a few at
the same time and spaced out a little, so twenty turrets do not turn into noise. In the web
build the browser allows sound only after the first click or key press; that is expected.
