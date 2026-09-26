# UI and input

Pillar 4: paper and pencil. Minimal, diegetic UI by default; a tactical overlay on demand.

## Input intents

Code names the intent, never the key. Touch is a later mapping, not a redesign.

| Intent | Desktop | Touch (later) | Note |
|---|---|---|---|
| `Interact` | left-click on spot / turret / docked boat | tap | opens the radial menu |
| `MoveBoat` | left-click on river | tap on river | clicking the bunker = go home |
| `FireTorpedo` | Space, hold = salvo | fire button | accepted only when the bow is aligned |
| `ToggleDive` | right-click, `T` secondary | dive button | never taught: discovered, like ramming |
| `StartWave` | Enter, or the next-wave button | button | |
| `ToggleOverlay` | Tab | button | toggle, persisted |
| `ToggleSpeed` | `F`, or the 1×/2× button | button | resets to 1× at every wave start |
| `Pause` / `Back` | Esc | button | closes the radial menu first, then opens the pause menu; in a menu: back |

Input priority: paper menu (pause, settings) > HUD buttons > radial menu > world. Under a paper
menu the world takes no input at all; a click on a HUD button never reaches the map.

## Minimal UI (default)

- RM as a small paper note in the corner; transactions appear briefly next to it.
- Torpedo count and boat HP shown at the boat only while hovering or commanding it.
- The boat's torpedo lane is visible while hovering or commanding it — "where does my straight
  end" is the most important information in the game and never hides behind Tab.
- Hover on a turret shows its range. Build spots show as pencil circles in the prep phase.
- The cursor says what a click would do: pencil by default, a steering mark over the water, a
  hammer over an empty build spot, a hand over a turret, the docked boat, a radial entry or a
  button. Hovering something clickable taps, a click that acts stamps (`AUDIO.md`). Immediate
  feedback for invalid or empty actions (dry click, one-line note), never a dialog.
- Four paper notes: RM top right (coin and figure), wave top left ("Welle 5/12"), boat bottom
  left (model, a dive icon while under water, one torpedo picture per tube, hull), bunker
  bottom right beside the bunker (its icon and integrity). Pictures where drawn, words only
  where not. No key hint on any note.
- Two paper buttons bottom centre, clear of the bunker: 1×/2× (with a small "F") and, in the
  build phase only, "Welle 6 »" (with a small "Enter"), inked red and breathing so the way on
  is never in doubt.
- **Phase banners**: a big paper banner slams in and slides away for the prologue, each wave
  start (the last one says so), a cleared wave with its bonus and a lost boat. Two words, a
  number at most. Real time, never takes input, never stops the game.
- Everything drawn on the map (ranges, rings, lanes, route, bars) is a pencil stroke; labels on
  the map sit on paper tags.

## Tab overlay (toggle)

Turret ranges, reload arcs, targeting mode (as its icon once drawn), exact HP of damaged ships as
pencil notes (an undamaged ship says nothing), the boat's lane, enemy route, destroyer shelling
range. The boat's hull and torpedoes stay on its note. Designed as a real mode, because most
players will leave it on.

Web note: check `Window::prevent_default_event_handling` so Tab does not leave the canvas.

## Radial menus

See `turrets.md` (spot, turret) and `uboat.md` (docked boat: repair, next model). Open on click,
close on Esc or click elsewhere, game keeps running.

## Prep phase

- Enemy route always faintly visible.
- **Wave preview** at the river mouth, where the ships will come from: each group as its ship
  picture with "8×" on a paper tag. The wave name appears only on the wave's banner.
- The next-wave button or Enter starts the wave; unlimited time before that.

## Menus

Paper cards over the faded map, pencil font, buttons that ink red and grow under the cursor.

- **Main menu**: "Flussbunker", Spielen (big), Einstellungen, Beenden (not in the web build).
  No instructions: the game teaches itself (`onboarding.md`).
- **Pause** (Esc in the game): the clock stops. Weiter, Neustart, Einstellungen, Hauptmenü.
- **Einstellungen** (from either): music and sound volume in 10 steps, hints on/off. For the
  session only. Back or Esc returns to where it was opened.
- **End card**: "Der Fluss hält" / "Bunker durchbrochen", Nochmal (big), Hauptmenü.

## Onboarding

See `onboarding.md`.
