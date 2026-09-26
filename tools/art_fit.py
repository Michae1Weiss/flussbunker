#!/usr/bin/env python3
"""Turn hand-drawn sheets into the files the game loads (docs/ART.md).

  art_fit.py sheet OUT [--set fx|hints]    guide sheet: one cell per sprite, one scale, line-weight legend
  art_fit.py slice SHEET [--set fx|hints]  cut the drawn sheet by the same cells, add the sticker, write assets/
  art_fit.py map-template OUT   river band, build spots, bunker on a tile grid: draw the map over it
  art_fit.py map-mask OUT       the exact water as white on transparent, for a clipping mask
  art_fit.py map DRAWN          the finished map into assets/map/background.png
  art_fit.py overlay MAP OUT    the geometry drawn onto a finished map, to check it
  art_fit.py check              what exists, what is still greybox; rebuilds assets/manifest.txt
                                (images and every .ogg under assets/audio)

Needs ImageMagick 7 (`magick`). Sizes mirror src/balance.rs, geometry mirrors src/game/map.rs;
change them there first, then here.
"""

import argparse
import os
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ASSETS = os.path.join(ROOT, "assets")

TILE = 48.0
MAP_TILES = (32, 18)
# Paper around the playfield, in tiles: a 16:10 screen sees one more tile top and bottom.
BLEED = 1
# Map canvas pixels per world unit, and the game's copy (WebGL caps textures near 4096 px).
MAP_DRAW_SCALE = 4
MAP_GAME_SCALE = 2
RIVER_HALF = 0.9 * TILE
SPOT_RADIUS = 0.5 * TILE
BUNKER_SIDE = 1.6 * TILE
CENTRELINE = [(-1, 3), (9, 3), (12, 8), (23, 8), (23, 13), (28, 13), (28, 17.5)]
BUILD_SPOTS = [(7, 5), (12.5, 5), (14, 10), (20.5, 6),
               (25, 11), (21, 11), (26, 15), (30, 14.5)]

HULL_BEAM_RATIO = 0.55
UBOAT_BEAM = 22.0

# Sheet pixels per world unit, and exported pixels per world unit: a Retina MacBook shows
# about 2.3 px per unit, and linear filtering stays clean up to about 2x reduction. Paper UI
# is shown larger than its world size (scaled with the window), so it keeps more pixels.
DRAW_SCALE = 16
SCALE = 3
UI_SCALE = 8
SHEET_WIDTH = 4096
SHEET_GAP = 96

# Line weights in world units: at 1920x1080 one unit is 1.25 px, at 1280x800 0.83 px.
OUTLINE = 1.2
DETAIL = 0.6
# The paper-cutout edge around every sticker, and how far its shadow falls, in world units.
BORDER = 1.0
SHADOW = 0.5
PAPER = "#f7f5ef"

SPRITES = {
    "sprites/ships/mtb.png": (29 * HULL_BEAM_RATIO, 29),
    "sprites/ships/corvette.png": (43 * HULL_BEAM_RATIO, 43),
    "sprites/ships/landing.png": (54 * HULL_BEAM_RATIO, 54),
    "sprites/ships/destroyer.png": (58 * HULL_BEAM_RATIO, 58),
    "sprites/ships/monitor.png": (72 * HULL_BEAM_RATIO, 72),
    "sprites/ships/gold.png": (50 * HULL_BEAM_RATIO, 50),
    "sprites/boats/viia.png": (UBOAT_BEAM, 54),
    "sprites/boats/viic.png": (UBOAT_BEAM, 61),
    "sprites/boats/ixc.png": (UBOAT_BEAM, 68),
    **{f"sprites/turrets/{kind}_{tier}.png": (side, side)
       for kind in ("flak", "gun88", "heavy15")
       for tier, side in enumerate((36, 43.2, 50.4))},
    "sprites/bunker.png": (77, 77),
    "sprites/spot.png": (48, 48),
    "sprites/splash.png": (48, 48),
    "sprites/crater.png": (22, 22),
    "sprites/debris_0.png": (14, 14),
    "sprites/debris_1.png": (6, 6),
    "sprites/debris_2.png": (7, 7),
    "sprites/debris_3.png": (8, 8),
    "sprites/debris_4.png": (9, 9),
    "sprites/debris_5.png": (10, 10),
    "sprites/torpedo.png": (5, 14),
    "sprites/shell.png": (4, 4),
}

ICONS = ("coin", "bunker", "torpedo", "hull", "dive", "surface",
         "first", "strongest", "weakest", "nearest", "upgrade", "sell")

# The second sheet: barrels, effects, paper UI. Its own layout, so the first sheet stays valid.
FX = {
    **{f"sprites/barrels/{kind}_{tier}.png": (side, side)
       for kind in ("flak", "gun88", "heavy15")
       for tier, side in enumerate((36, 43.2, 50.4))},
    "fx/flash_0.png": (16, 16),
    "fx/flash_1.png": (16, 16),
    **{f"fx/explosion_{i}.png": (48, 48) for i in range(4)},
    "fx/smoke.png": (32, 32),
    "ui/note.png": (48, 32),
    "ui/tag.png": (32, 16),
    "ui/banner.png": (96, 32),
    "ui/disc.png": (48, 48),
    **{f"ui/icons/{name}.png": (24, 24) for name in ICONS},
    "ui/stroke.png": (96, 8),
}

# The third sheet: onboarding pictures in two frames each, and the mouse cursors.
CURSORS = ("pencil", "steer", "build", "hand")
HINTS = {
    **{f"ui/hints/mouse_{i}.png": (32, 40) for i in range(2)},
    **{f"ui/hints/space_{i}.png": (72, 24) for i in range(2)},
    **{f"ui/cursors/{name}.png": (12, 12) for name in CURSORS},
}
# Cursors are screen pixels, not world units: 48 px, whatever the window.
CURSOR_SCALE = 4

SETS = {"sprites": SPRITES, "fx": FX, "hints": HINTS}
SIZES = {**SPRITES, **FX, **HINTS}

# Drawn on the paper, under the water, or as ink on a note: no cutout edge, no shadow.
FLAT = {"sprites/crater.png", "sprites/splash.png", "sprites/spot.png",
        "sprites/shell.png", "sprites/torpedo.png", "ui/stroke.png",
        *(k for k in FX if k.startswith(("fx/", "ui/icons/"))),
        *(k for k in HINTS if k.startswith("ui/cursors/"))}
# Stretched by the game: corners stay, edges repeat along one axis, the middle fills.
NINE_SLICE = {"ui/note.png", "ui/tag.png", "ui/banner.png"}
NINE_SLICE_BORDER = 0.25


def export_scale(key):
    if key.startswith("ui/cursors/"):
        return CURSOR_SCALE
    return UI_SCALE if key.startswith("ui/") else SCALE


def magick(*args):
    """The last argument is often the output; its folder is created on the way."""
    last = args[-1] if args else ""
    if last.endswith(".png"):
        os.makedirs(os.path.dirname(os.path.abspath(last)), exist_ok=True)
        # Without timestamps a re-slice of the same drawing leaves git clean.
        args = (*args[:-1], "-define", "png:exclude-chunks=date,time", last)
    result = subprocess.run(["magick", *args], capture_output=True, text=True)
    if result.returncode:
        sys.exit(f"magick failed: {result.stderr.strip()}")
    return result.stdout


def size_of(path):
    w, h = magick(path, "-format", "%w %h", "info:").split()
    return int(w), int(h)


def font_args():
    fonts = ["/System/Library/Fonts/Supplemental/Arial.ttf", "/System/Library/Fonts/Menlo.ttc",
             "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"]
    return next((["-font", f] for f in fonts if os.path.exists(f)), [])


def border(key):
    """Small debris would drown in a full-width edge."""
    w, h = SIZES[key]
    return 0.0 if key in FLAT else min(BORDER, 0.1 * min(w, h))


def margin(key):
    """Distance from the cell edge to the drawing, in world units: room for edge and shadow."""
    b = border(key)
    return b + b * SHADOW / BORDER


def record(present):
    """The web build cannot stat files; it compiles this list in."""
    with open(os.path.join(ASSETS, "manifest.txt"), "w") as f:
        f.write("".join(k + "\n" for k in sorted(present)))


def sheet_cells(name):
    """Rows in the order of the set; the same layout draws the guide and slices the drawing."""
    cells, x, y, row_h = [], SHEET_GAP, SHEET_GAP, 0
    for key, (ww, wh) in SETS[name].items():
        w, h = round(ww * DRAW_SCALE), round(wh * DRAW_SCALE)
        if x + w + SHEET_GAP > SHEET_WIDTH:
            x, y, row_h = SHEET_GAP, y + row_h + SHEET_GAP, 0
        cells.append((key, x, y, w, h))
        x += w + SHEET_GAP
        row_h = max(row_h, h)
    return cells, y + row_h + SHEET_GAP


def sheet_height(name, legend_y):
    """The first sheet's size is fixed by drawings already made; the fx legend has more lines."""
    return legend_y + (5 if name == "sprites" else 7) * SHEET_GAP


def underlays(cells):
    """A barrel cell shows its drawn base faintly, so the barrel lands on the right spot."""
    args = []
    for key, x, y, cw, ch in cells:
        base = os.path.join(ASSETS, key.replace("/barrels/", "/turrets/"))
        if "/barrels/" in key and os.path.exists(base):
            args += ["(", base, "-resize", f"{cw}x{ch}!", "-channel", "A", "-evaluate", "multiply",
                     "0.25", "+channel", ")", "-geometry", f"+{x}+{y}", "-composite"]
    return args


def sheet(out, name):
    cells, legend_y = sheet_cells(name)
    w, h = SHEET_WIDTH, sheet_height(name, legend_y)
    frame = ["-fill", "none", "-stroke", "rgba(60,140,220,0.35)", "-strokewidth", "2"]
    inner = ["-stroke", "rgba(60,140,220,0.9)", "-strokewidth", "3"]
    labels = ["-stroke", "none", "-fill", "rgba(60,140,220,1)", "-pointsize", "22", *font_args()]
    for key, x, y, cw, ch in cells:
        m = round(margin(key) * DRAW_SCALE)
        frame += ["-draw", f"rectangle {x},{y} {x + cw},{y + ch}",
                  "-draw", f"line {x + cw / 2:.0f},{y} {x + cw / 2:.0f},{y + ch}",
                  "-draw", f"line {x},{y + ch / 2:.0f} {x + cw},{y + ch / 2:.0f}"]
        if m:
            inner += ["-draw", f"rectangle {x + m},{y + m} {x + cw - m},{y + ch - m}"]
        if key in NINE_SLICE:
            bx, by = cw * NINE_SLICE_BORDER, ch * NINE_SLICE_BORDER
            frame += ["-draw", f"line {x + bx:.0f},{y} {x + bx:.0f},{y + ch}",
                      "-draw", f"line {x + cw - bx:.0f},{y} {x + cw - bx:.0f},{y + ch}",
                      "-draw", f"line {x},{y + by:.0f} {x + cw},{y + by:.0f}",
                      "-draw", f"line {x},{y + ch - by:.0f} {x + cw},{y + ch - by:.0f}"]
        label = key.split("/", 1)[1][:-4]
        labels += ["-draw", f"text {x},{y - 10} '{label}'"]
    ly = legend_y + SHEET_GAP
    legend = ["-fill", "none", "-stroke", "black",
              "-strokewidth", f"{OUTLINE * DRAW_SCALE:.0f}", "-draw", f"line {SHEET_GAP},{ly} {SHEET_GAP + 400},{ly}",
              "-strokewidth", f"{DETAIL * DRAW_SCALE:.0f}", "-draw", f"line {SHEET_GAP},{ly + 80} {SHEET_GAP + 400},{ly + 80}",
              "-stroke", "#c8372d", "-strokewidth", f"{OUTLINE * DRAW_SCALE:.0f}",
              "-draw", f"line {SHEET_GAP},{ly + 160} {SHEET_GAP + 400},{ly + 160}",
              "-stroke", "none", "-fill", "rgba(60,140,220,1)", "-pointsize", "28", *font_args(),
              "-draw", f"text {SHEET_GAP + 440},{ly + 10} 'outline: the thinnest a silhouette edge may be'",
              "-draw", f"text {SHEET_GAP + 440},{ly + 90} 'detail: the thinnest line anywhere'",
              "-draw", f"text {SHEET_GAP + 440},{ly + 170} 'the one red: the player side only'",
              "-draw", f"text {SHEET_GAP + 1800},{ly + 10} 'Draw inside the dark frame. Bow up. The centre is the pivot.'",
              "-draw", f"text {SHEET_GAP + 1800},{ly + 90} 'Hide this layer, export PNG at 100%, run: art_fit.py slice'",
              "-draw", f"text {SHEET_GAP + 1800},{ly + 170} 'Empty cells are skipped; draw in any order.'"]
    if name == "fx":
        legend += ["-draw", f"text {SHEET_GAP},{ly + 250} 'Barrels: from the centre (the pivot) upward; the faint base is only a guide.'",
                   "-draw", f"text {SHEET_GAP},{ly + 330} 'note, tag, banner: the thin lines are 9-slice cuts. Corners stay, edges stretch: keep edges even and the middle plain.'",
                   "-draw", f"text {SHEET_GAP},{ly + 410} 'stroke: one pencil line along the middle, left and right ends must join seamlessly.'"]
    if name == "hints":
        legend += ["-draw", f"text {SHEET_GAP},{ly + 250} 'mouse, space: two frames of one drawing. Trace frame 0 for frame 1; only the pressed part changes. No lettering.'",
                   "-draw", f"text {SHEET_GAP},{ly + 330} 'mouse_1: the left button filled red, two or three short click strokes. space_1: the cap pressed down, its side thinner.'",
                   "-draw", f"text {SHEET_GAP},{ly + 410} 'cursors: ink with a thin white outline, no paper. Click point: pencil tip, hammer head, fingertip top left; steer mark in the centre.'"]
    magick("-size", f"{w}x{h}", "xc:white", *underlays(cells), *frame, *inner, *labels, *legend, out)
    print(f"{out}  {w}x{h}, {DRAW_SCALE} px per world unit")


def sticker(src, dst, key):
    """Paper behind the drawing, filled up to the outline, a cut edge and a soft shadow."""
    b = border(key) * DRAW_SCALE
    if not b:
        magick(src, dst)
        return
    w, h = size_of(src)
    with tempfile.TemporaryDirectory() as tmp:
        mask, edge, shade = (os.path.join(tmp, n) for n in ("mask.png", "edge.png", "shade.png"))
        magick(src, "-alpha", "extract", "-threshold", "10%",
               "-morphology", "Dilate", f"Disk:{b:.1f}", "-alpha", "off", "-colorspace", "sRGB",
               "-fill", "red", "-draw", "color 0,0 floodfill",
               "-fill", "white", "+opaque", "red", "-fill", "black", "-opaque", "red",
               "-colorspace", "Gray", mask)
        magick(mask, "-background", PAPER, "-alpha", "shape", edge)
        magick(mask, "-blur", f"0x{b / 3:.1f}", "-evaluate", "multiply", "0.35",
               "-background", "black", "-alpha", "shape", shade)
        off = round(b * SHADOW / BORDER)
        magick("-size", f"{w}x{h}", "xc:none", shade, "-geometry", f"+{off}+{off}", "-composite",
               edge, "-composite", src, "-composite", dst)


def slice_sheet(sheet_png, name):
    """The drawn sheet, guide layer hidden, exported at the guide's size."""
    cells, legend_y = sheet_cells(name)
    expected = (SHEET_WIDTH, sheet_height(name, legend_y))
    if size_of(sheet_png) != expected:
        sys.exit(f"{sheet_png} is {size_of(sheet_png)}, the guide is {expected}; export at 100%")
    with tempfile.TemporaryDirectory() as tmp:
        piece, done = os.path.join(tmp, "piece.png"), os.path.join(tmp, "done.png")
        for key, x, y, cw, ch in cells:
            magick(sheet_png, "-crop", f"{cw}x{ch}+{x}+{y}", "+repage", piece)
            if magick(piece, "-format", "%[fx:maxima.a]", "info:") == "0":
                continue
            sticker(piece, done, key)
            out = os.path.join(ASSETS, key)
            os.makedirs(os.path.dirname(out), exist_ok=True)
            scale = export_scale(key)
            magick(done, "-resize", f"{round(SIZES[key][0] * scale)}x{round(SIZES[key][1] * scale)}!", out)
            print(f"wrote {key}")
    check()


def map_px(scale):
    """Canvas size with bleed at `scale` pixels per world unit."""
    return tuple(round((n + 2 * BLEED) * TILE * scale) for n in MAP_TILES)


def at(t, x, y):
    """Play-area tile coordinates to canvas pixels on a bleed canvas with `t` px per tile."""
    return (x + BLEED) * t, (y + BLEED) * t


def river_strokes(t, colour):
    """One round-capped line per segment: a wide polyline on a transparent canvas draws phantom chords."""
    draws = ["-fill", "none", "-stroke", colour, "-strokewidth", f"{2 * RIVER_HALF / TILE * t:.0f}"]
    for (x1, y1), (x2, y2) in zip(CENTRELINE, CENTRELINE[1:]):
        (ax, ay), (bx, by) = at(t, x1, y1), at(t, x2, y2)
        draws += ["-draw", f"stroke-linecap round line {ax:.0f},{ay:.0f} {bx:.0f},{by:.0f}"]
    return draws


def geometry_draws(t, band):
    path = " ".join("{:.0f},{:.0f}".format(*at(t, x, y)) for x, y in CENTRELINE)
    draws = river_strokes(t, band) + [
        "-stroke", "rgba(220,60,40,0.9)", "-strokewidth", "3", "-draw", f"polyline {path}"]
    for x, y in BUILD_SPOTS:
        cx, cy = at(t, x, y)
        draws += ["-draw", f"circle {cx:.0f},{cy:.0f} {cx + SPOT_RADIUS / TILE * t:.0f},{cy:.0f}"]
    bx, by = at(t, *CENTRELINE[-1])
    half = BUNKER_SIDE / TILE * t / 2
    draws += ["-draw", f"rectangle {bx - half:.0f},{by - half:.0f} {bx + half:.0f},{by + half:.0f}"]
    x0, y0 = at(t, 0, 0)
    x1, y1 = at(t, *MAP_TILES)
    draws += ["-stroke", "rgba(60,140,220,0.8)", "-strokewidth", "2",
              "-draw", f"rectangle {x0:.0f},{y0:.0f} {x1:.0f},{y1:.0f}"]
    return draws


def scale_references(t):
    """Legend and true-size outlines in the free land at the bottom left of the playfield."""
    u = t / TILE
    x, y = at(t, 1, 12.2)
    draws = ["-fill", "none", "-stroke", "black"]
    for i, (weight, label) in enumerate(((OUTLINE, "bank, at most"), (DETAIL, "flow, grass, detail"))):
        ly = y + i * 0.9 * t
        draws += ["-strokewidth", f"{weight * u:.1f}", "-draw", f"line {x:.0f},{ly:.0f} {x + 2 * t:.0f},{ly:.0f}"]
        draws += ["-stroke", "none", "-fill", "rgba(60,140,220,1)", "-pointsize", f"{0.35 * t:.0f}", *font_args(),
                  "-draw", f"text {x + 2.2 * t:.0f},{ly + 0.12 * t:.0f} '{label}'", "-fill", "none", "-stroke", "black"]
    shapes = [("MTB", 29), ("destroyer", 58), ("monitor", 72)]
    sx, sy = x, y + 2.2 * t
    draws += ["-stroke", "rgba(60,140,220,1)", "-strokewidth", "2"]
    for name, length in shapes:
        w, h = length * HULL_BEAM_RATIO * u, length * u
        draws += ["-draw", f"roundrectangle {sx:.0f},{sy:.0f} {sx + w:.0f},{sy + h:.0f} {w / 2:.0f},{w / 2:.0f}"]
        sx += w + 0.5 * t
    turret = 43.2 * u
    draws += ["-draw", f"rectangle {sx:.0f},{sy:.0f} {sx + turret:.0f},{sy + turret:.0f}"]
    sx += turret + 0.5 * t
    draws += ["-draw", f"circle {sx + 0.5 * t:.0f},{sy + 0.5 * t:.0f} {sx + t:.0f},{sy + 0.5 * t:.0f}"]
    draws += ["-stroke", "none", "-fill", "rgba(60,140,220,1)", "-pointsize", f"{0.3 * t:.0f}", *font_args(),
              "-draw", f"text {x:.0f},{sy - 0.15 * t:.0f} 'true size: MTB, destroyer, monitor, turret; a tree at most one tile'"]
    return draws


def map_template(out):
    w, h = map_px(MAP_DRAW_SCALE)
    t = TILE * MAP_DRAW_SCALE
    grid = []
    for i in range(MAP_TILES[0] + 2 * BLEED + 1):
        grid += ["-draw", f"line {i * t:.0f},0 {i * t:.0f},{h}"]
    for i in range(MAP_TILES[1] + 2 * BLEED + 1):
        grid += ["-draw", f"line 0,{i * t:.0f} {w},{i * t:.0f}"]
    magick("-size", f"{w}x{h}", "xc:white", "-stroke", "rgba(0,0,0,0.08)", "-strokewidth", "1",
           *grid, *geometry_draws(t, "#bcd6f3"), *scale_references(t), out)
    print(f"{out}  {w}x{h}: water in the blue band, circles and square stay bare, "
          f"the blue frame is a 16:9 screen")


def map_mask(out):
    w, h = map_px(MAP_DRAW_SCALE)
    magick("-size", f"{w}x{h}", "xc:none", *river_strokes(TILE * MAP_DRAW_SCALE, "white"), out)
    print(f"{out}  {w}x{h}: white is water, the rest is land")


def map_in(drawn):
    """Takes a bleed canvas, or an old 16:9 map that gets paper around it."""
    w, h = size_of(drawn)
    gw, gh = map_px(MAP_GAME_SCALE)
    out = os.path.join(ASSETS, "map", "background.png")
    if abs(w / h - gw / gh) < 0.01:
        magick(drawn, "-alpha", "off", "-resize", f"{gw}x{gh}!", out)
    elif abs(w / h - MAP_TILES[0] / MAP_TILES[1]) < 0.01:
        pw, ph = (round(n * TILE * MAP_GAME_SCALE) for n in MAP_TILES)
        magick(drawn, "-alpha", "off", "-resize", f"{pw}x{ph}!", "-background", "white",
               "-gravity", "center", "-extent", f"{gw}x{gh}", out)
    else:
        sys.exit(f"{drawn} is {w}x{h}; draw on the canvas from map-template")
    print("wrote map/background.png")
    check()


def overlay(map_png, out):
    w, _ = size_of(map_png)
    magick(map_png, *geometry_draws(w / (MAP_TILES[0] + 2 * BLEED), "rgba(40,120,220,0.3)"), out)
    print(f"{out}: water should fill the blue band, circles should sit on land")


def audio_files():
    """Every .ogg under assets/audio; the web build learns of them only through the manifest."""
    found = []
    for folder, _, files in os.walk(os.path.join(ASSETS, "audio")):
        found += [os.path.relpath(os.path.join(folder, f), ASSETS) for f in files if f.endswith(".ogg")]
    return sorted(found)


def check():
    keys = [*SIZES, "map/background.png", "fonts/pencil.ttf"]
    present = [k for k in keys if os.path.exists(os.path.join(ASSETS, k))]
    record(present + audio_files())
    for key in keys:
        print(f"{'x' if key in present else '-'}  {key}")
    print(f"{len(present)}/{len(keys)} drawn, {len(audio_files())} sound files")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    for name, arg in (("sheet", "out"), ("slice", "sheet")):
        p = sub.add_parser(name)
        p.add_argument(arg)
        p.add_argument("--set", choices=SETS, default="sprites")
    for name, arg in (("map-template", "out"), ("map-mask", "out"), ("map", "drawn")):
        sub.add_parser(name).add_argument(arg)
    p = sub.add_parser("overlay")
    p.add_argument("map")
    p.add_argument("out")
    sub.add_parser("check")
    args = parser.parse_args()
    {
        "sheet": lambda: sheet(args.out, args.set),
        "slice": lambda: slice_sheet(args.sheet, args.set),
        "map-template": lambda: map_template(args.out),
        "map-mask": lambda: map_mask(args.out),
        "map": lambda: map_in(args.drawn),
        "overlay": lambda: overlay(args.map, args.out),
        "check": check,
    }[args.command]()


if __name__ == "__main__":
    main()
