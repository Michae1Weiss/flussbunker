# Audio: what to record, how long, how loud

Every sound the game plays, with the moment it plays, so each can be recorded or found on its own.
The audio layer works like the art layer: a file that exists under `assets/audio/` plays, a
missing one is silence. Deliver them in any order.

Tone, from `docs/VISION.md`: playful, a little cartoonish, never grim. Everything should sound
like a toy war on a kitchen table seen through a 1940s newsreel: real sounds, slightly
exaggerated, never gory, no screams, no voices.

## Where the files go

Sources live in `sound/takes/`, the game's files in `assets/audio/`, and `tools/sound.sh` turns
one into the other. Never edit or drop files into `assets/audio/` by hand: the next run replaces
them.

`sfx/flak_shot` in the tables means the source `sound/takes/sfx/flak_shot.flac` (`.wav` and
`.ogg` work too) and becomes `assets/audio/sfx/flak_shot.ogg`, or the variants
`flak_shot_1.ogg`, `_2.ogg` ... (up to 8) when the source holds several takes. Music sources are
`sound/takes/music/build.flac` and `wave.flac`. `sound/suno/` keeps generated originals untouched,
`sound/old/` what was replaced. `cargo run` logs how many sounds it found
(`audio: 12 of 41 sounds, music: 2`).

## Workflow

1. **One Audacity project per family** (weapons, explosions, ui, boat, music), kept outside the
   repo in `~/Documents/Audacity4/Projects/`.
2. **One track per sound, named like it** (`flak_shot`). Record every take onto that track, at
   least **1 s of silence** between takes and less than 1 s of silence inside a take. Delete the
   bad takes; a rough cut is enough. Noise Reduction, pitch, layering and the loop point of a loop
   are done here.
3. **Export the track:** select it, Export, *Selected audio*, FLAC, to `sound/takes/sfx/`. Only
   the tracks that changed.
4. **`tools/sound.sh`** (needs `brew install sox` and ffmpeg). It splits each source into its
   takes, cuts the silence before and after, normalises, fades, and writes mono 48 kHz OGG; loops
   and music are only levelled and encoded. It lists every file with its length and warns about
   names the game does not know, sounds longer than 4 s and more than 8 takes. It also rebuilds
   `assets/manifest.txt`, which the web build needs. A noisy recording splits badly: run
   `THRESHOLD=-40 tools/sound.sh` (the default is -48 dB below the loudest take).

Record at 48 kHz with the loudest peaks around -12 to -6 dB, never in the red; the script brings
the level up. Record five or six takes and keep the best three.

## Rules for every file

1. **Format:** OGG Vorbis, quality 5, 48 kHz. Effects **mono**, music **stereo**. Done by
   `tools/sound.sh`.
2. **Level:** every file at a **peak of -1 dBFS**. Do not make quiet sounds quiet in the file;
   the game sets each sound's loudness from the "Level" column. Done by `tools/sound.sh`.
3. **Start:** no silence before the sound; a late start feels like input lag. Done by
   `tools/sound.sh`.
4. **End:** let the tail ring out; the script cuts the silence after it and fades the last 20 ms.
5. **Loops** (marked "loop"): the end must flow into the start without a click. Keep only the
   steady part, split it in the middle, swap the halves and crossfade the seam; test with loop
   playback in Audacity. No fade at either end. The script leaves loops uncut; add a new one to
   `LOOPS` in `tools/sound.sh`.
6. **Variants:** where the table asks for several, record several takes on the one track; the
   script names them `name_1.ogg`, `name_2.ogg` ... The game picks one at random and also shifts
   the pitch a little ("Pitch" column), so 3 takes sound like 20.
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
- **Suno** for music. Songs made on the free plan are for non-commercial use only, and a later
  subscription does not cover them; generate the final tracks under Pro or Premier and note the
  plan and date in `CREDITS.md`. `build` and `wave` must share key and tempo, which two separate
  generations rarely do: use stems or extend one song. Keep the download in `sound/suno/` and
  edit the loop in Audacity.
- **Audacity** for pitch shift, low-pass (for distance) and layering: two sounds on two tracks (a
  low boom plus a crackle) is how most game explosions are made.

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
| `sfx/explode_large` | a destroyer or monitor explodes or breaks, also when a chain kills it | 2.0-3.5 s | 2 | 0 dB | ±5 % | the loudest sound in the game: a long boom, rumble, debris raining down |
| `sfx/bank_hit` | a torpedo runs into the bank | 1.0-1.5 s | 2 | -6 dB | ±6 % | a dull explosion into earth, dirt falling. Duller than a ship hit: "wasted" |
| `sfx/splash` | a shell or a piece of debris lands in the water | 0.3-0.6 s | 3 | -16 dB | ±10 % | a plop and a short spray. A hand or a cup dropped into a filled sink |
| `sfx/debris_land` | debris lands on land | 0.2-0.4 s | 3 | -16 dB | ±10 % | a thud on dirt, a bag of rice dropped on the floor |
| `sfx/burn` | a ship dies by fire and drifts to the bank | 1.5-2.5 s | 2 | -14 dB | ±6 % | crackle of flames, a short "whoomp" at the start |
| `sfx/sink` | a rammed ship capsizes and goes down | 2.0-3.0 s | 2 | -12 dB | ±6 % | hull groan, gurgling water |
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
