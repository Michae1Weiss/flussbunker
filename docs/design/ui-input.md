# UI and input

Pillar 4: paper and pencil. Minimal, diegetic UI by default; a tactical overlay on demand.

## Input intents

Code names the intent, never the key. Touch is a later mapping, not a redesign.

| Intent | Desktop | Touch (later) | Note |
|---|---|---|---|
| `Interact` | left-click on spot / turret / docked boat | tap | opens the radial menu |
| `MoveBoat` | left-click on river | tap on river | clicking the bunker = go home |
| `FireTorpedo` | Space, hold = salvo | fire button | accepted only when the bow is aligned |
| `ToggleDive` | right-click, `T` secondary | dive button | whole boat control on the mouse |
| `StartWave` | Enter | button at the bunker | |
| `ToggleOverlay` | Tab | button | toggle, persisted |
| `ToggleSpeed` | `F` | button | 1× / 2×, resets to 1× at every wave start |
| `Pause` / `Back` | Esc | button | closes menus first, then the pause menu |

Input priority: modal UI > radial menu > world. Menus never swallow game intents (Space, Enter).

## Minimal UI (default)

- RM as a small paper note in the corner; transactions appear briefly next to it.
- Torpedo count and boat HP shown at the boat only while hovering or commanding it.
- The boat's torpedo lane is visible while hovering or commanding it — "where does my straight
  end" is the most important information in the game and never hides behind Tab.
- Hover on a turret shows its range. Build spots show as pencil circles in the prep phase.
- Cursor changes for interaction hints. Immediate feedback for invalid or empty actions (dry click,
  one-line note), never a dialog.
- Three paper notes, nothing else on screen: RM (coin and figure), wave (number and name,
  bunker integrity, the preview line between waves, a 2x badge while fast), boat (model and
  trim, torpedoes and hull). Key hints appear on the boat note only until dive and torpedo were
  each used once; no other key hint is ever on screen (M9).
- **Phase banners**: a big paper banner slams in and slides away for the prologue, each wave
  start (the last one says so), a cleared wave with its bonus and a lost boat; a pause banner
  stays while paused. Real time, never takes input, never stops the game.
- Everything drawn on the map (ranges, rings, lanes, route, bars) is a pencil stroke; labels on
  the map sit on paper tags.

## Tab overlay (toggle)

Turret ranges, reload arcs, targeting mode (as its icon once drawn), exact HP of damaged ships as
pencil notes (an undamaged ship says nothing), boat hull and torpedoes, the boat's lane, enemy
route, destroyer shelling range. Designed as a real mode, because most
players will leave it on.

Web note: check `Window::prevent_default_event_handling` so Tab does not leave the canvas.

## Radial menus

See `turrets.md` (spot, turret) and `uboat.md` (docked boat: repair, next model). Open on click,
close on Esc or click elsewhere, game keeps running.

## Prep phase

- Enemy route always faintly visible.
- **Wave preview** as a radio message on the paper note: exact composition as icons with counts,
  plus the wave name as one word of flavour. No hint texts.
- Enter starts the wave; unlimited time before that.

## Onboarding

See `onboarding.md`.
