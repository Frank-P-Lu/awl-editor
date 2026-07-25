#!/usr/bin/env python3
"""PIXEL ARITHMETIC over the exported tiles — the appearance oracle.

A manifest and a screenshot both being "green" proves nothing about what the
icon LOOKS like; this repo has already been burned once by a state oracle
reporting a selected row that rendered fully invisible. So every claim the
export makes is asserted here by counting pixels:

  shape      the tile's corner is transparent — the squircle is a real shape,
             not a square the Dock will happily draw as a square
  cursor     the world's `primary` appears as a solid block of pixels: the fake
             logo-cursor is actually painted
  cursor ink the "l" reads INSIDE that cursor's bounding box, in
             `primary_content` — the knocked-out letter, not a blank slab
  wordmark   `base_content` ink exists OUTSIDE the cursor box: "aw" is there
  ground     `base_100` is the dominant colour

Deliberately bbox-relative rather than palette-nearest, because Wagtail's four
tokens collapse to two values (black/white): "which colour is this pixel" cannot
tell its ink from its ground, but "is there dark ink inside the light cursor"
can, and that is the claim that matters.

No third-party imports: the PNG is decoded here (8-bit RGB/RGBA, non-interlaced
— what Chromium writes).
"""

import argparse
import json
import pathlib
import struct
import sys
import zlib

# The HARD gate runs at the Dock size; the ladder below it is reported.
ASSERT_SIZES = [128]
LADDER = [128, 64, 32, 24]


def decode_png(path):
    data = path.read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise SystemExit(f"{path}: not a PNG")
    pos, idat, hdr, pal_alpha = 8, [], None, False
    while pos < len(data):
        (length,) = struct.unpack(">I", data[pos : pos + 4])
        ctype = data[pos + 4 : pos + 8]
        body = data[pos + 8 : pos + 8 + length]
        if ctype == b"IHDR":
            hdr = struct.unpack(">IIBBBBB", body)
        elif ctype == b"IDAT":
            idat.append(body)
        elif ctype == b"IEND":
            break
        pos += 12 + length
    w, h, depth, color, _comp, _filt, interlace = hdr
    if depth != 8 or interlace != 0 or color not in (2, 6):
        raise SystemExit(f"{path}: unsupported PNG ({depth=} {color=} {interlace=})")
    bpp = 4 if color == 6 else 3
    raw = zlib.decompress(b"".join(idat))
    stride = w * bpp
    out = bytearray(h * stride)
    prev = bytearray(stride)
    p = 0
    for y in range(h):
        ft = raw[p]
        p += 1
        line = bytearray(raw[p : p + stride])
        p += stride
        if ft == 1:
            for i in range(bpp, stride):
                line[i] = (line[i] + line[i - bpp]) & 0xFF
        elif ft == 2:
            for i in range(stride):
                line[i] = (line[i] + prev[i]) & 0xFF
        elif ft == 3:
            for i in range(stride):
                left = line[i - bpp] if i >= bpp else 0
                line[i] = (line[i] + ((left + prev[i]) >> 1)) & 0xFF
        elif ft == 4:
            for i in range(stride):
                a = line[i - bpp] if i >= bpp else 0
                b = prev[i]
                c = prev[i - bpp] if i >= bpp else 0
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[i] = (line[i] + pr) & 0xFF
        elif ft != 0:
            raise SystemExit(f"{path}: bad filter {ft}")
        out[y * stride : (y + 1) * stride] = line
        prev = line
    return w, h, bpp, bytes(out)


def hexrgb(s):
    s = s.lstrip("#")
    return (int(s[0:2], 16), int(s[2:4], 16), int(s[4:6], 16))


def near(px, rgb, tol):
    return abs(px[0] - rgb[0]) <= tol and abs(px[1] - rgb[1]) <= tol and abs(px[2] - rgb[2]) <= tol


def blobs(mask, w, h):
    """Every 4-connected region in `mask`, as (size, (x0, y0, x1, y1))."""
    seen = bytearray(w * h)
    found = []
    for start in range(w * h):
        if not mask[start] or seen[start]:
            continue
        stack, size = [start], 0
        seen[start] = 1
        x0 = x1 = start % w
        y0 = y1 = start // w
        while stack:
            i = stack.pop()
            size += 1
            x, y = i % w, i // w
            x0, x1 = min(x0, x), max(x1, x)
            y0, y1 = min(y0, y), max(y1, y)
            for j, ok in ((i - 1, x > 0), (i + 1, x < w - 1), (i - w, y > 0), (i + w, y < h - 1)):
                if ok and mask[j] and not seen[j]:
                    seen[j] = 1
                    stack.append(j)
        found.append((size, (x0, y0, x1, y1)))
    return found


def cursor_region(mask, w, h):
    """The fake cursor's size + bounding box, found as SHAPE rather than colour.

    Two complications, both real and both handled here rather than by naming a
    world:

      * A knocked-out "l" can SPLIT the cursor into a left and a right sliver
        (the super-narrow pill does this whenever the glyph is wider than the
        pill), so the pieces are re-merged — a piece counts as part of the same
        cursor when it spans essentially the same VERTICAL extent and sits
        right beside it.
      * Wagtail and Cassowary paint `primary` and `base_content` the SAME
        value, so the "aw" letters are the same colour as the cursor. They are
        excluded by that same vertical test: x-height letters cover well under
        60% of the cursor's height.
    """
    parts = blobs(mask, w, h)
    if not parts:
        return 0, None
    size, (x0, y0, x1, y1) = max(parts, key=lambda b: b[0])
    height = y1 - y0 + 1
    for s, (a0, b0, a1, b1) in parts:
        if (a0, b0, a1, b1) == (x0, y0, x1, y1):
            continue
        overlap = min(y1, b1) - max(y0, b0) + 1
        gap = max(a0 - x1, x0 - a1, 0)
        if overlap >= 0.6 * height and gap <= 0.15 * w:
            size += s
            x0, y0, x1, y1 = min(x0, a0), min(y0, b0), max(x1, a1), max(y1, b1)
    return size, (x0, y0, x1, y1)


def analyse(path, world):
    w, h, bpp, buf = decode_png(path)
    ground = hexrgb(world["base_100"])
    ink = hexrgb(world["base_content"])
    cursor = hexrgb(world["primary"])
    curink = hexrgb(world["primary_content"])

    def at(x, y):
        i = (y * w + x) * bpp
        return (buf[i], buf[i + 1], buf[i + 2], buf[i + 3] if bpp == 4 else 255)

    corner_alpha = at(0, 0)[3]
    mask = bytearray(w * h)
    counts = {"ground": 0, "cursor": 0}
    for y in range(h):
        base = y * w * bpp
        for x in range(w):
            i = base + x * bpp
            if bpp == 4 and buf[i + 3] < 128:
                continue
            px = (buf[i], buf[i + 1], buf[i + 2])
            if near(px, cursor, 6):
                mask[y * w + x] = 1
            elif near(px, ground, 6):
                counts["ground"] += 1
    counts["cursor"], box = cursor_region(mask, w, h)

    inside_curink = 0
    outside_ink = 0
    for y in range(h):
        base = y * w * bpp
        for x in range(w):
            i = base + x * bpp
            if bpp == 4 and buf[i + 3] < 128:
                continue
            px = (buf[i], buf[i + 1], buf[i + 2])
            in_box = box is not None and box[0] <= x <= box[2] and box[1] <= y <= box[3]
            if in_box and near(px, curink, 24):
                inside_curink += 1
            elif not in_box and near(px, ink, 24):
                outside_ink += 1
    return {
        "corner_alpha": corner_alpha,
        "ground": counts["ground"],
        "cursor": counts["cursor"],
        "cursor_ink": inside_curink,
        "wordmark_ink": outside_ink,
        "area": w * h,
    }


def legible(r):
    """Is the knocked-out "l" actually resolvable on the cursor at this size?"""
    return r["cursor_ink"] >= max(3, r["area"] * 0.0006)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--manifest", required=True)
    ap.add_argument("--tiles", required=True)
    ap.add_argument("--presets", default="block,pill,narrow")
    ap.add_argument("--report", help="write the legibility ladder here")
    args = ap.parse_args()
    manifest = json.loads(pathlib.Path(args.manifest).read_text())
    tiles = pathlib.Path(args.tiles)
    presets = args.presets.split(",")

    failures = []
    checked = 0
    ladder = []
    for world in manifest["worlds"]:
        for preset in presets:
            floor = None
            for size in LADDER:
                p = tiles / f"{world['name']}-{preset}-{size}.png"
                if not p.exists():
                    failures.append(f"{p.name}: missing")
                    continue
                r = analyse(p, world)
                area = r["area"]
                name = f"{world['name']}/{preset}@{size}"
                if legible(r) and r["cursor"] >= area * 0.004:
                    floor = size
                if size in ASSERT_SIZES:
                    # THE HARD GATE, at the size a Dock icon is actually seen.
                    checked += 1
                    if r["corner_alpha"] != 0:
                        failures.append(f"{name}: corner not transparent (alpha={r['corner_alpha']})")
                    if r["cursor"] < area * 0.004:
                        failures.append(f"{name}: cursor block missing ({r['cursor']}px of {area})")
                    if not legible(r):
                        failures.append(f"{name}: 'l' not legible inside the cursor ({r['cursor_ink']}px)")
                    if r["wordmark_ink"] < area * 0.004:
                        failures.append(f"{name}: 'aw' ink missing ({r['wordmark_ink']}px)")
                    if r["ground"] < area * 0.30:
                        failures.append(f"{name}: ground is not dominant ({r['ground']}px of {area})")
            ladder.append((world["name"], world["font"], preset, floor))

    print(f"verify.py: {checked} tiles asserted at {ASSERT_SIZES}, ladder walked over {LADDER}")
    if failures:
        for f in failures:
            print(f"  FAIL {f}")
        sys.exit(1)
    print("verify.py: every tile carries ground, cursor, cursor-ink and wordmark ink")

    # The LEGIBILITY LADDER is reported, never gated: how far down each candidate
    # keeps its knocked-out "l". Below the Dock sizes every 3-glyph wordmark
    # eventually becomes a smudge, so a floor of 32 or 24 is information for
    # whoever assigns a preset to a world — not a defect to hide behind a
    # loosened threshold.
    lines = ["world           face                  preset   'l' legible down to"]
    for name, font, preset, floor in ladder:
        lines.append(f"{name:<15} {font:<21} {preset:<8} {(str(floor) + 'px') if floor else 'not at any size'}")
    text = "\n".join(lines) + "\n"
    print()
    print(text, end="")
    if args.report:
        pathlib.Path(args.report).write_text(text)


if __name__ == "__main__":
    main()
