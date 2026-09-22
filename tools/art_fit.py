#!/usr/bin/env python3
"""Turn hand-drawn sheets into the files the game loads (docs/ART.md).

  art_fit.py sheet OUT          guide sheet: one cell per sprite, one scale, line-weight legend
  art_fit.py slice SHEET        cut the drawn sheet by the same cells, add the sticker, write assets/
  art_fit.py map-template OUT   river band, build spots, bunker on a tile grid: draw the map over it
  art_fit.py map-mask OUT       the exact water as white on transparent, for a clipping mask
  art_fit.py map DRAWN          the finished map into assets/map/background.png
  art_fit.py overlay MAP OUT    the geometry drawn onto a finished map, to check it
  art_fit.py check              what exists, what is still greybox; rebuilds assets/manifest.txt

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
MAP_PX = (3072, 1728)
RIVER_HALF = 0.9 * TILE
SPOT_RADIUS = 0.5 * TILE
BUNKER_SIDE = 1.6 * TILE
CENTRELINE = [(-1, 3), (9, 3), (12, 8), (23, 8), (23, 13), (28, 13), (28, 17.5)]
BUILD_SPOTS = [(7.5, 5.5), (13, 4.5), (14, 10.5), (20.5, 5.5),
               (25.5, 10.5), (20.5, 14.5), (25.5, 15.5), (30.5, 14.5)]

HULL_BEAM_RATIO = 0.55
UBOAT_BEAM = 22.0

# Sheet pixels per world unit, and exported pixels per world unit: a Retina MacBook shows
# about 2.3 px per unit, and linear filtering stays clean up to about 2x reduction.
DRAW_SCALE = 16
SCALE = 3
SHEET_WIDTH = 4096
SHEET_GAP = 96

# Line weights in world units: at 1920x1080 one unit is 1.25 px, at 1280x800 0.83 px.
OUTLINE = 1.2
DETAIL = 0.6
# The paper-cutout edge around every sticker, and how far its shadow falls, in world units.
BORDER = 1.0
SHADOW = 0.5
PAPER = "#f7f5ef"

SIZES = {
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

# Drawn on the paper or under the water: no cutout edge, no shadow.
FLAT = {"sprites/crater.png", "sprites/splash.png", "sprites/spot.png",
        "sprites/shell.png", "sprites/torpedo.png"}


def magick(*args):
    """The last argument is often the output; its folder is created on the way."""
    last = args[-1] if args else ""
    if last.endswith(".png"):
        os.makedirs(os.path.dirname(os.path.abspath(last)), exist_ok=True)
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


def sheet_cells():
    """Rows in the order of SIZES; the same layout draws the guide and slices the drawing."""
    cells, x, y, row_h = [], SHEET_GAP, SHEET_GAP, 0
    for key in SIZES:
        w, h = round(SIZES[key][0] * DRAW_SCALE), round(SIZES[key][1] * DRAW_SCALE)
        if x + w + SHEET_GAP > SHEET_WIDTH:
            x, y, row_h = SHEET_GAP, y + row_h + SHEET_GAP, 0
        cells.append((key, x, y, w, h))
        x += w + SHEET_GAP
        row_h = max(row_h, h)
    return cells, y + row_h + SHEET_GAP


def sheet(out):
    cells, legend_y = sheet_cells()
    w, h = SHEET_WIDTH, legend_y + 5 * SHEET_GAP
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
        name = os.path.basename(key)[:-4]
        labels += ["-draw", f"text {x},{y - 10} '{name}'"]
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
    magick("-size", f"{w}x{h}", "xc:white", *frame, *inner, *labels, *legend, out)
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


def slice_sheet(sheet_png):
    """The drawn sheet, guide layer hidden, exported at the guide's size."""
    cells, legend_y = sheet_cells()
    expected = (SHEET_WIDTH, legend_y + 5 * SHEET_GAP)
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
            magick(done, "-resize", f"{round(SIZES[key][0] * SCALE)}x{round(SIZES[key][1] * SCALE)}!", out)
            print(f"wrote {key}")
    check()


def river_strokes(t, colour):
    """One round-capped line per segment: a wide polyline on a transparent canvas draws phantom chords."""
    draws = ["-fill", "none", "-stroke", colour, "-strokewidth", f"{2 * RIVER_HALF / TILE * t:.0f}"]
    for (x1, y1), (x2, y2) in zip(CENTRELINE, CENTRELINE[1:]):
        draws += ["-draw", f"stroke-linecap round line {x1 * t:.0f},{y1 * t:.0f} {x2 * t:.0f},{y2 * t:.0f}"]
    return draws


def geometry_draws(t):
    path = " ".join(f"{x * t:.0f},{y * t:.0f}" for x, y in CENTRELINE)
    draws = river_strokes(t, "rgba(40,120,220,0.35)") + [
        "-stroke", "rgba(220,60,40,0.9)", "-strokewidth", "3", "-draw", f"polyline {path}"]
    for x, y in BUILD_SPOTS:
        draws += ["-draw", f"circle {x * t:.0f},{y * t:.0f} {x * t + SPOT_RADIUS / TILE * t:.0f},{y * t:.0f}"]
    bx, by = CENTRELINE[-1]
    half = BUNKER_SIDE / TILE * t / 2
    draws += ["-draw", f"rectangle {bx * t - half:.0f},{by * t - half:.0f} {bx * t + half:.0f},{by * t + half:.0f}"]
    return draws


def map_template(out):
    t = MAP_PX[0] / MAP_TILES[0]
    grid = []
    for i in range(MAP_TILES[0] + 1):
        grid += ["-draw", f"line {i * t:.0f},0 {i * t:.0f},{MAP_PX[1]}"]
    for i in range(MAP_TILES[1] + 1):
        grid += ["-draw", f"line 0,{i * t:.0f} {MAP_PX[0]},{i * t:.0f}"]
    magick("-size", f"{MAP_PX[0]}x{MAP_PX[1]}", "xc:white", "-stroke", "rgba(0,0,0,0.08)",
           "-strokewidth", "1", *grid, *geometry_draws(t), out)
    print(f"{out}  {MAP_PX[0]}x{MAP_PX[1]}: water in the blue band, circles and square stay bare")


def map_mask(out):
    t = MAP_PX[0] / MAP_TILES[0]
    magick("-size", f"{MAP_PX[0]}x{MAP_PX[1]}", "xc:none", *river_strokes(t, "white"), out)
    print(f"{out}  {MAP_PX[0]}x{MAP_PX[1]}: white is water, the rest is land")


def map_in(drawn):
    w, h = size_of(drawn)
    if abs(w / h - MAP_PX[0] / MAP_PX[1]) > 0.01:
        sys.exit(f"{drawn} is {w}x{h}, not 16:9; draw on the template's canvas")
    out = os.path.join(ASSETS, "map", "background.png")
    os.makedirs(os.path.dirname(out), exist_ok=True)
    magick(drawn, "-alpha", "off", "-resize", f"{MAP_PX[0]}x{MAP_PX[1]}!", out)
    print("wrote map/background.png")
    check()


def overlay(map_png, out):
    w, _ = size_of(map_png)
    magick(map_png, *geometry_draws(w / MAP_TILES[0]), out)
    print(f"{out}: water should fill the blue band, circles should sit on land")


def check():
    keys = [*SIZES, "map/background.png", "fonts/pencil.ttf"]
    present = [k for k in keys if os.path.exists(os.path.join(ASSETS, k))]
    record(present)
    for key in keys:
        print(f"{'x' if key in present else '-'}  {key}")
    print(f"{len(present)}/{len(keys)} drawn")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    for name, arg in (("sheet", "out"), ("slice", "sheet"), ("map-template", "out"),
                      ("map-mask", "out"), ("map", "drawn")):
        sub.add_parser(name).add_argument(arg)
    p = sub.add_parser("overlay")
    p.add_argument("map")
    p.add_argument("out")
    sub.add_parser("check")
    args = parser.parse_args()
    {
        "sheet": lambda: sheet(args.out),
        "slice": lambda: slice_sheet(args.sheet),
        "map-template": lambda: map_template(args.out),
        "map-mask": lambda: map_mask(args.out),
        "map": lambda: map_in(args.drawn),
        "overlay": lambda: overlay(args.map, args.out),
        "check": check,
    }[args.command]()


if __name__ == "__main__":
    main()
