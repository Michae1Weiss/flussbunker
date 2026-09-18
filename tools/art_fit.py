#!/usr/bin/env python3
"""Fit generated images to the asset contract in docs/ART.md.

  art_fit.py fit SOURCE KEY [--keep-background]  trim, pad to the key's aspect, write assets/KEY
  art_fit.py split SHEET OUT_DIR                  cut one drawing per PNG out of a generated sheet
  art_fit.py check                                which keys exist, which sources look wrong
  art_fit.py sheet OUT                            guide sheet to draw every sprite on, one scale
  art_fit.py slice SHEET                          cut a drawn sheet by the guide's cells into assets/
  art_fit.py map-template OUT                     river band, build spots, bunker: draw over this
  art_fit.py map-mask OUT                         the exact water as white on transparent, to paint under
  art_fit.py overlay MAP OUT                      the same geometry drawn onto a finished map

KEY is a path from docs/ART.md, e.g. sprites/ships/mtb.png. Needs ImageMagick 7 (`magick`).
Sizes mirror docs/ART.md, geometry mirrors src/game/map.rs; re-run after changing either.
"""

import argparse
import os
import re
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ASSETS = os.path.join(ROOT, "assets")

TILE = 48.0
MAP_TILES = (32, 18)
RIVER_HALF = 0.9 * TILE
SPOT_RADIUS = 0.5 * TILE
BUNKER_SIDE = 1.6 * TILE
CENTRELINE = [(-1, 3), (9, 3), (12, 8), (23, 8), (23, 13), (28, 13), (28, 17.5)]
BUILD_SPOTS = [(7.5, 5.5), (13, 4.5), (14, 10.5), (20.5, 5.5),
               (25.5, 10.5), (20.5, 14.5), (25.5, 15.5), (30.5, 14.5)]

# Pixels per world unit: drawn on the sheet, and exported. The game scales to world size;
# 4x stays crisp on a 4K screen, 16x gives the pencil room.
DRAW_SCALE = 16
SCALE = 4
SHEET_GAP = 64
BACKGROUND_FUZZ = "12%"

# key -> (world w, world h). Sprites on the paper (decals) keep their background when asked.
SIZES = {
    "sprites/ships/mtb.png": (9, 16),
    "sprites/ships/corvette.png": (13, 24),
    "sprites/ships/landing.png": (17, 30),
    "sprites/ships/destroyer.png": (18, 32),
    "sprites/ships/monitor.png": (22, 40),
    "sprites/ships/gold.png": (15, 28),
    "sprites/boats/viia.png": (12, 30),
    "sprites/boats/viic.png": (12, 34),
    "sprites/boats/ixc.png": (12, 38),
    "sprites/torpedo.png": (5, 14),
    "sprites/shell.png": (4, 4),
    "sprites/debris_0.png": (14, 14),
    "sprites/debris_1.png": (6, 6),
    "sprites/debris_2.png": (7, 7),
    "sprites/debris_3.png": (8, 8),
    "sprites/debris_4.png": (9, 9),
    "sprites/debris_5.png": (10, 10),
    "sprites/crater.png": (22, 22),
    "sprites/splash.png": (48, 48),
    "sprites/bunker.png": (77, 77),
    "sprites/spot.png": (48, 48),
    "map/background.png": (MAP_TILES[0] * TILE, MAP_TILES[1] * TILE),
}
for kind in ("flak", "gun88", "heavy15"):
    for tier, side in enumerate((24, 29, 34)):
        SIZES[f"sprites/turrets/{kind}_{tier}.png"] = (side, side)


def magick(*args):
    return subprocess.run(["magick", *args], check=True, capture_output=True, text=True).stdout


def size_of(path):
    w, h = magick(path, "-format", "%w %h", "info:").split()
    return int(w), int(h)


def strip_background(src, dst):
    """Flood the four corners to transparent; interior whites survive."""
    w, h = size_of(src)
    corners = [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)]
    draws = []
    for x, y in corners:
        draws += ["-draw", f"alpha {x},{y} floodfill"]
    magick(src, "-alpha", "set", "-fuzz", BACKGROUND_FUZZ, "-fill", "none", *draws, dst)


def fit(source, key, keep_background=False):
    if key not in SIZES:
        sys.exit(f"unknown key {key}; see docs/ART.md")
    ww, wh = SIZES[key]
    out = os.path.join(ASSETS, key)
    os.makedirs(os.path.dirname(out), exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp:
        work = os.path.join(tmp, "work.png")
        if keep_background:
            magick(source, work)
        else:
            strip_background(source, work)
            magick(work, "-trim", "+repage", work)
        w, h = size_of(work)
        aspect = ww / wh
        if w / h > aspect:
            ew, eh = w, round(w / aspect)
        else:
            ew, eh = round(h * aspect), h
        args = [work, "-background", "none", "-gravity", "center", "-extent", f"{ew}x{eh}"]
        if key != "map/background.png":
            args += ["-resize", f"{round(ww * SCALE)}x{round(wh * SCALE)}!"]
        magick(*args, out)
    fw, fh = size_of(out)
    record(key)
    drift = abs(w / h - aspect) / aspect
    flag = "  <- padded a lot, check the drawing" if drift > 0.15 else ""
    print(f"{key}: {source} {w}x{h} -> {fw}x{fh}{flag}")


def record(key):
    """The web build cannot stat files; it compiles this list in."""
    path = os.path.join(ASSETS, "manifest.txt")
    keys = set(open(path).read().split()) if os.path.exists(path) else set()
    keys.add(key)
    with open(path, "w") as f:
        f.write("".join(k + "\n" for k in sorted(keys)))


def split(sheet, out_dir, min_area=400):
    """Connected components after a dilation, so a ship and its detached barrel stay one piece."""
    os.makedirs(out_dir, exist_ok=True)
    w, h = size_of(sheet)
    radius = max(2, round(min(w, h) * 0.015))
    with tempfile.TemporaryDirectory() as tmp:
        mask = os.path.join(tmp, "mask.png")
        strip_background(sheet, mask)
        magick(mask, "-alpha", "extract", "-threshold", "10%",
               "-morphology", "Dilate", f"Disk:{radius}", mask)
        report = magick(mask, "-define", "connected-components:verbose=true",
                        "-define", f"connected-components:area-threshold={min_area}",
                        "-connected-components", "8", "null:")
    boxes = []
    for line in report.splitlines():
        m = re.match(r"\s*\d+: (\d+)x(\d+)\+(\d+)\+(\d+) [\d.,]+ \d+ (\S+)", line)
        if m and m.group(5) in ("srgb(255,255,255)", "gray(255)", "white"):
            bw, bh, bx, by = (int(v) for v in m.groups()[:4])
            boxes.append((by // (h // 4), bx, bw, bh, by))
    boxes.sort()
    for i, (_, bx, bw, bh, by) in enumerate(boxes):
        out = os.path.join(out_dir, f"piece_{i:02}.png")
        magick(sheet, "-crop", f"{bw}x{bh}+{bx}+{by}", "+repage", out)
        print(f"{out}  {bw}x{bh}")
    print(f"{len(boxes)} pieces; fit each with: art_fit.py fit PIECE KEY")


def check():
    """Also rebuilds the manifest from disk, so a deleted file leaves it."""
    present = [key for key in SIZES if os.path.exists(os.path.join(ASSETS, key))]
    with open(os.path.join(ASSETS, "manifest.txt"), "w") as f:
        f.write("".join(k + "\n" for k in sorted(present)))
    for key, (ww, wh) in SIZES.items():
        path = os.path.join(ASSETS, key)
        if not os.path.exists(path):
            print(f"      -  {key}")
            continue
        w, h = size_of(path)
        drift = abs(w / h - ww / wh) / (ww / wh)
        note = "" if drift < 0.01 else f"  aspect {w}x{h} is not {ww}x{wh}; it will stretch"
        print(f"{w:>5}x{h:<5} {key}{note}")


def sheet_cells():
    """Cells sorted tallest first, packed into rows; the same layout draws the guide and slices it."""
    keys = [k for k in SIZES if k != "map/background.png"]
    keys.sort(key=lambda k: (-SIZES[k][1], -SIZES[k][0]))
    cells, x, y, row_h, width = [], SHEET_GAP, SHEET_GAP, 0, 4096
    for key in keys:
        w, h = SIZES[key][0] * DRAW_SCALE, SIZES[key][1] * DRAW_SCALE
        if x + w + SHEET_GAP > width:
            x, y, row_h = SHEET_GAP, y + row_h + SHEET_GAP, 0
        cells.append((key, x, y, w, h))
        x += w + SHEET_GAP
        row_h = max(row_h, h)
    return cells, width, y + row_h + SHEET_GAP


def sheet(out):
    cells, w, h = sheet_cells()
    draws = ["-fill", "none", "-stroke", "rgba(60,140,220,0.6)", "-strokewidth", "2"]
    for key, x, y, cw, ch in cells:
        draws += ["-draw", f"rectangle {x},{y} {x + cw},{y + ch}"]
        draws += ["-draw", f"line {x + cw / 2:.0f},{y} {x + cw / 2:.0f},{y + ch}",
                  "-draw", f"line {x},{y + ch / 2:.0f} {x + cw},{y + ch / 2:.0f}"]
    labels = ["-stroke", "none", "-fill", "rgba(60,140,220,0.9)", "-pointsize", "18"]
    fonts = ["/System/Library/Fonts/Supplemental/Arial.ttf", "/System/Library/Fonts/Menlo.ttc",
             "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"]
    labels += next((["-font", f] for f in fonts if os.path.exists(f)), [])
    for key, x, y, cw, ch in cells:
        name = os.path.basename(key)[:-4]
        labels += ["-draw", f"text {x},{y - 6} '{name} {SIZES[key][0]}x{SIZES[key][1]}'"]
    magick("-size", f"{w}x{h}", "xc:white", *draws, *labels, out)
    print(f"{out}  {w}x{h}, {DRAW_SCALE} px per world unit; draw on a layer above, bow up")


def slice_sheet(sheet_png):
    """The drawn sheet, guide layer hidden, exported at the guide's size."""
    cells, w, h = sheet_cells()
    sw, sh = size_of(sheet_png)
    if (sw, sh) != (w, h):
        sys.exit(f"{sheet_png} is {sw}x{sh}, the guide is {w}x{h}; export at 100%")
    with tempfile.TemporaryDirectory() as tmp:
        for key, x, y, cw, ch in cells:
            piece = os.path.join(tmp, "piece.png")
            magick(sheet_png, "-crop", f"{cw}x{ch}+{x}+{y}", "+repage", piece)
            if magick(piece, "-format", "%[fx:maxima.a]", "info:") != "0":
                fit(piece, key)
    check()


def map_mask(out):
    """The water exactly as the code has it; paint banks around it, keep it as a clipping mask."""
    t = 96
    w, h = MAP_TILES[0] * t, MAP_TILES[1] * t
    magick("-size", f"{w}x{h}", "xc:none", *river_strokes(t, "white"), out)
    print(f"{out}  {w}x{h}: white is water, everything else is land")


def river_strokes(t, colour):
    """One round-capped line per segment: a wide polyline on a transparent canvas draws phantom chords."""
    draws = ["-fill", "none", "-stroke", colour, "-strokewidth", f"{2 * RIVER_HALF / TILE * t:.0f}"]
    for (x1, y1), (x2, y2) in zip(CENTRELINE, CENTRELINE[1:]):
        draws += ["-draw", f"stroke-linecap round line {x1 * t:.0f},{y1 * t:.0f} {x2 * t:.0f},{y2 * t:.0f}"]
    return draws


def geometry_draws(px_per_tile):
    t = px_per_tile
    path = " ".join(f"{x * t:.0f},{y * t:.0f}" for x, y in CENTRELINE)
    draws = river_strokes(t, "rgba(40,120,220,0.35)") + [
        "-stroke", "rgba(220,60,40,0.9)", "-strokewidth", "3",
        "-draw", f"polyline {path}",
    ]
    for x, y in BUILD_SPOTS:
        draws += ["-draw", f"circle {x * t:.0f},{y * t:.0f} {x * t + SPOT_RADIUS / TILE * t:.0f},{y * t:.0f}"]
    bx, by = CENTRELINE[-1]
    half = BUNKER_SIDE / TILE * t / 2
    draws += ["-draw", f"rectangle {bx * t - half:.0f},{by * t - half:.0f} {bx * t + half:.0f},{by * t + half:.0f}"]
    return draws


def map_template(out):
    t = 96
    w, h = MAP_TILES[0] * t, MAP_TILES[1] * t
    grid = []
    for i in range(MAP_TILES[0] + 1):
        grid += ["-draw", f"line {i * t},0 {i * t},{h}"]
    for i in range(MAP_TILES[1] + 1):
        grid += ["-draw", f"line 0,{i * t} {w},{i * t}"]
    magick("-size", f"{w}x{h}", "xc:white", "-stroke", "rgba(0,0,0,0.08)", "-strokewidth", "1",
           *grid, *geometry_draws(t), out)
    print(f"{out}  {w}x{h}: water inside the blue band, circles empty, square is the bunker")


def overlay(map_png, out):
    w, h = size_of(map_png)
    magick(map_png, *geometry_draws(w / MAP_TILES[0]), out)
    print(f"{out}: water should fill the blue band, circles should sit on land")


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    p = sub.add_parser("fit")
    p.add_argument("source")
    p.add_argument("key")
    p.add_argument("--keep-background", action="store_true")
    p = sub.add_parser("split")
    p.add_argument("sheet")
    p.add_argument("out_dir")
    sub.add_parser("check")
    sub.add_parser("sheet").add_argument("out")
    sub.add_parser("slice").add_argument("sheet")
    sub.add_parser("map-template").add_argument("out")
    sub.add_parser("map-mask").add_argument("out")
    p = sub.add_parser("overlay")
    p.add_argument("map")
    p.add_argument("out")
    args = parser.parse_args()
    if args.command == "fit":
        fit(args.source, args.key, args.keep_background)
    elif args.command == "split":
        split(args.sheet, args.out_dir)
    elif args.command == "check":
        check()
    elif args.command == "sheet":
        sheet(args.out)
    elif args.command == "slice":
        slice_sheet(args.sheet)
    elif args.command == "map-mask":
        map_mask(args.out)
    elif args.command == "map-template":
        map_template(args.out)
    elif args.command == "overlay":
        overlay(args.map, args.out)


if __name__ == "__main__":
    main()
