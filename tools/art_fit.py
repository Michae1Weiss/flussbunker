#!/usr/bin/env python3
"""Fit generated images to the asset contract in docs/ART.md.

  art_fit.py fit SOURCE KEY [--keep-background]  trim, pad to the key's aspect, write assets/KEY
  art_fit.py split SHEET OUT_DIR                  cut one drawing per PNG out of a generated sheet
  art_fit.py check                                which keys exist, which sources look wrong
  art_fit.py map-template OUT                     river band, build spots, bunker: draw over this
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

# Exported pixels per world unit. The game scales to world size; 4x stays crisp on a 4K screen.
SCALE = 4
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


def geometry_draws(px_per_tile):
    t = px_per_tile
    path = " ".join(f"{x * t:.0f},{y * t:.0f}" for x, y in CENTRELINE)
    draws = [
        "-fill", "none", "-stroke", "rgba(40,120,220,0.35)", "-strokewidth", f"{2 * RIVER_HALF / TILE * t:.0f}",
        "-draw", f"stroke-linejoin round polyline {path}",
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
    sub.add_parser("map-template").add_argument("out")
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
    elif args.command == "map-template":
        map_template(args.out)
    elif args.command == "overlay":
        overlay(args.map, args.out)


if __name__ == "__main__":
    main()
