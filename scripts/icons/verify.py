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
  ground     the tile's `ground` (`base_100`, or a world's blend toward
             `base_300`) is the dominant colour

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

# The HARD presence gate runs at the Dock size; the ladder below it is
# reported. Geometry is a separate roster law over the shipped assignment at
# every native review size. It deliberately does NOT turn 32/24px legibility
# into a decision: item 99a owns that taste call.
ASSERT_SIZES = [128]
LADDER = [128, 64, 32, 24]
GEOMETRY_SIZES = [256, 128, 64, 44, 32, 24]


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


def geometry_cursor_region(mask, w, h):
    """Rightmost cursor bbox, robust to same-colour ink and tiny split slabs."""
    # Crop before component finding: at tiny sizes Cassowary's `w` can touch
    # its same-colour block through antialiasing, making the whole word one
    # component. The cursor itself always starts well right of 60% of the tile.
    right_mask = bytearray(mask)
    for y in range(h):
        for x in range(round(0.60 * w)):
            right_mask[y * w + x] = 0
    parts = blobs(right_mask, w, h)
    right = [
        part
        for part in parts
        if (part[1][0] + part[1][2]) / 2 >= 0.65 * w
    ]
    if not right:
        return 0, None
    size = sum(part[0] for part in right)
    x0 = min(part[1][0] for part in right)
    y0 = min(part[1][1] for part in right)
    x1 = max(part[1][2] for part in right)
    y1 = max(part[1][3] for part in right)
    return size, (x0, y0, x1, y1)


def bbox(points):
    if not points:
        return None
    return (
        min(p[0] for p in points),
        min(p[1] for p in points),
        max(p[0] for p in points),
        max(p[1] for p in points),
    )


def dist2(px, rgb):
    return sum((px[i] - rgb[i]) ** 2 for i in range(3))


def blend_coverage(px, back, front):
    """Coverage of `front` over `back`, or None when px is off that colour line."""
    axis = tuple(front[i] - back[i] for i in range(3))
    denom = sum(v * v for v in axis)
    if denom == 0:
        return None
    t = sum((px[i] - back[i]) * axis[i] for i in range(3)) / denom
    if t < 0 or t > 1.08:
        return None
    expect = tuple(back[i] + t * axis[i] for i in range(3))
    residual = sum((px[i] - expect[i]) ** 2 for i in range(3))
    return t if residual <= 36 else None


def geometry(path, world):
    """Rendered ink/cursor boxes for the shipped lockup.

    `aw` is found from its authored ink outside the already-located cursor.
    The knocked-out `l` is found as the non-cursor ink between each cursor
    scanline's solid edges. That scanline rule handles the two-value worlds:
    there, `primary_content == base_100`, so colour naming alone cannot tell
    the real glyph hole from the tile ground.
    """
    w, h, bpp, buf = decode_png(path)
    # The tile's ACTUAL ground (item 121: `base_100` unless the world opted
    # into a blend toward `base_300`) — never `base_100` directly, since the
    # rendered pixels follow `ground`, not the raw token.
    ground = hexrgb(world["ground"])
    ink = hexrgb(world["base_content"])
    cursor = hexrgb(world["primary"])
    curink = hexrgb(world["primary_content"])

    def at(x, y):
        i = (y * w + x) * bpp
        return (buf[i], buf[i + 1], buf[i + 2])

    def opaque(x, y):
        return bpp != 4 or buf[(y * w + x) * bpp + 3] >= 128

    cursor_mask = bytearray(w * h)
    for y in range(h):
        for x in range(w):
            coverage = blend_coverage(at(x, y), ground, cursor)
            if opaque(x, y) and coverage is not None and coverage >= 0.08:
                cursor_mask[y * w + x] = 1
    _count, cursor_box = geometry_cursor_region(cursor_mask, w, h)
    if cursor_box is None:
        return {"cursor": None, "aw": None, "l": None, "wordmark": None, "l_outside": 0}

    cx0, cy0, cx1, cy1 = cursor_box
    aw_points = []
    l_mask = bytearray(w * h)
    outside_points = []
    for y in range(h):
        row_x = [x for x in range(cx0, cx1 + 1) if cursor_mask[y * w + x]]
        row_span = (min(row_x), max(row_x)) if row_x else None
        for x in range(w):
            px = at(x, y)
            if not opaque(x, y):
                continue
            in_cursor_box = cx0 <= x <= cx1 and cy0 <= y <= cy1
            ink_coverage = blend_coverage(px, ground, ink)
            if not in_cursor_box and ink_coverage is not None and ink_coverage >= 0.08:
                aw_points.append((x, y))
            if row_span is not None and row_span[0] <= x <= row_span[1]:
                if dist2(px, curink) + 4 < dist2(px, cursor):
                    l_mask[y * w + x] = 1
            elif cx0 <= x <= cx1 and (y < cy0 or y > cy1):
                # A visible ascender/descender outside the cursor is the
                # containment defect. Require positive separation from ground
                # so a two-value world's invisible ground cannot fabricate it.
                if dist2(px, curink) + 4 < dist2(px, ground) and near(px, curink, 6):
                    outside_points.append((x, y))

    l_parts = blobs(l_mask, w, h)
    if l_parts:
        # The glyph hole is the tall central component; edge antialias flecks
        # are smaller and farther from the cursor's horizontal centre.
        center = (cx0 + cx1) / 2
        _size, l_box = max(
            l_parts,
            key=lambda part: (
                part[1][3] - part[1][1],
                part[0],
                -abs((part[1][0] + part[1][2]) / 2 - center),
            ),
        )
        l_points = [
            (x, y)
            for y in range(l_box[1], l_box[3] + 1)
            for x in range(l_box[0], l_box[2] + 1)
            if l_mask[y * w + x]
        ]
    else:
        l_points = []
    l_points.extend(outside_points)
    aw_box = bbox(aw_points)
    l_box = bbox(l_points)
    wordmark_box = bbox(aw_points + l_points)
    return {
        "cursor": cursor_box,
        "aw": aw_box,
        "l": l_box,
        "wordmark": wordmark_box,
        "l_outside": len(outside_points),
    }


def analyse(path, world):
    w, h, bpp, buf = decode_png(path)
    # The tile's ACTUAL ground (item 121: `base_100` unless the world opted
    # into a blend toward `base_300`) — never `base_100` directly, since the
    # rendered pixels follow `ground`, not the raw token.
    ground = hexrgb(world["ground"])
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


def analyse_favicon(path, world):
    """Tiny companion mark: transparent corner, dominant ground, cursor + `a`."""
    w, h, bpp, buf = decode_png(path)
    ground = hexrgb(world["ground"])
    cursor = hexrgb(world["primary"])
    curink = hexrgb(world["primary_content"])
    counts = {"ground": 0, "cursor": 0, "cursor_ink": 0}
    pixels = []
    for y in range(h):
        for x in range(w):
            i = (y * w + x) * bpp
            if bpp == 4 and buf[i + 3] < 128:
                continue
            px = (buf[i], buf[i + 1], buf[i + 2])
            pixels.append((x, y, px))
            nearest = min(
                ((dist2(px, color), name) for name, color in (
                    ("ground", ground), ("cursor", cursor), ("cursor_ink", curink)
                )),
                key=lambda pair: pair[0],
            )
            if nearest[0] <= 24 * 24 * 3:
                counts[nearest[1]] += 1
    cursor_mask = bytearray(w * h)
    for x, y, px in pixels:
        coverage = blend_coverage(px, ground, cursor)
        if coverage is not None and coverage >= 0.08:
            cursor_mask[y * w + x] = 1
    # Use the envelope of every cursor pixel. The `a` can divide a tiny cursor
    # into several connected pieces (especially in two-value worlds), while it
    # is still plainly one authored shape to the eye.
    box = bbox([(x, y) for y in range(h) for x in range(w) if cursor_mask[y * w + x]])
    if box is not None:
        x0, y0, x1, y1 = box
        # The highlighted glyph is the contrasting hole inside the cursor.
        # This also covers two-value worlds where cursor ink equals ground.
        counts["cursor_ink"] = sum(
            1 for x, y, px in pixels
            if x0 <= x <= x1 and y0 <= y <= y1
            and dist2(px, curink) + 4 < dist2(px, cursor)
        )
    corner_alpha = buf[3] if bpp == 4 else 255
    return {**counts, "corner_alpha": corner_alpha, "area": w * h}


def legible(r):
    """Is the knocked-out "l" actually resolvable on the cursor at this size?"""
    return r["cursor_ink"] >= max(3, r["area"] * 0.0006)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--manifest", required=True)
    ap.add_argument("--tiles", required=True)
    ap.add_argument("--presets", default="block,pill,narrow")
    ap.add_argument("--report", help="write the legibility ladder here")
    ap.add_argument("--geometry-report", help="write the shipped-roster geometry table here")
    ap.add_argument("--favicons", help="directory containing per-world paired favicons")
    args = ap.parse_args()
    manifest = json.loads(pathlib.Path(args.manifest).read_text())
    tiles = pathlib.Path(args.tiles)
    presets = args.presets.split(",")

    failures = []
    checked = 0

    if args.favicons:
        favicon_dir = pathlib.Path(args.favicons)
        favicon_sizes = [16, 32, 48, 64, 180]
        for world in manifest["worlds"]:
            for size in favicon_sizes:
                p = favicon_dir / f"{world['name']}-{size}.png"
                if not p.exists():
                    failures.append(f"{p.name}: missing paired favicon")
                    continue
                r = analyse_favicon(p, world)
                area = r["area"]
                if r["corner_alpha"] >= 128:
                    failures.append(f"{p.name}: corner is visibly opaque")
                if r["ground"] < area * 0.20:
                    failures.append(f"{p.name}: theme ground missing")
                if r["cursor"] < max(2, area * 0.015):
                    failures.append(f"{p.name}: theme cursor missing")
                if r["cursor_ink"] < max(1, area * 0.004):
                    failures.append(f"{p.name}: highlighted `a` missing")
        if not failures:
            print(
                f"verify.py: paired favicons carry theme ground, cursor and highlighted `a` "
                f"for {len(manifest['worlds'])} worlds x {len(favicon_sizes)} sizes"
            )
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

    # THE VERTICAL-RHYTHM LAW. Sweep the complete shipped roster rather than a
    # hand-picked Bilby/Gumtree pair: every display face must land on the same
    # optical seat, the raised cursor must bracket the visible `l`, and no new
    # face may silently reintroduce a vertical jump. The two Dock surfaces
    # compose these exact transparent tiles, so one raster measurement applies
    # identically to both.
    geometry_rows = []
    by_size = {size: [] for size in GEOMETRY_SIZES}
    for world in manifest["worlds"]:
        preset = world["cursor"]
        for size in GEOMETRY_SIZES:
            p = tiles / f"{world['name']}-{preset}-{size}.png"
            if not p.exists():
                failures.append(f"{p.name}: missing for shipped geometry law")
                continue
            g = geometry(p, world)
            name = f"{world['name']}/{preset}@{size}"
            if g["cursor"] is None or g["aw"] is None or g["wordmark"] is None:
                failures.append(f"{name}: no measurable cursor/wordmark ink bbox")
                continue
            cursor_box, aw_box, l_box = g["cursor"], g["aw"], g["l"]
            baseline = aw_box[3]
            by_size[size].append((world["name"], baseline))
            top_lead = aw_box[1] - cursor_box[1]
            min_lead = max(1, round(size * 0.08))
            if top_lead < min_lead:
                failures.append(
                    f"{name}: cursor still sits low (top lead {top_lead}px, need >= {min_lead}px)"
                )
            if l_box is not None and (
                l_box[0] < cursor_box[0]
                or l_box[1] < cursor_box[1]
                or l_box[2] > cursor_box[2]
                or l_box[3] > cursor_box[3]
                or g["l_outside"] > 0
            ):
                failures.append(
                    f"{name}: `l` ink {l_box} escapes cursor {cursor_box} "
                    f"({g['l_outside']} outside pixels)"
                )
            geometry_rows.append(
                (
                    world["name"],
                    world["font"],
                    preset,
                    size,
                    g["wordmark"],
                    cursor_box,
                    l_box,
                    baseline,
                )
            )
    for size, seats in by_size.items():
        if len(seats) != len(manifest["worlds"]):
            continue
        low = min(seats, key=lambda x: x[1])
        high = max(seats, key=lambda x: x[1])
        # Native raster quantisation contributes a one-pixel choice at each
        # edge; a two-pixel full spread is the measured noise floor.
        tolerance = 2
        if high[1] - low[1] > tolerance:
            failures.append(
                f"shipped@{size}: optical-seat spread {high[1] - low[1]}px exceeds "
                f"{tolerance}px ({low[0]}={low[1]}, {high[0]}={high[1]})"
            )

    if failures:
        for f in failures:
            print(f"  FAIL {f}")
        sys.exit(1)
    print(
        f"verify.py: shipped vertical rhythm pinned for {len(manifest['worlds'])} worlds "
        f"x {len(GEOMETRY_SIZES)} sizes x both Dock surfaces"
    )

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

    geometry_lines = [
        "world           face                  preset   px   wordmark ink bbox   cursor bbox         l ink bbox          aw baseline",
    ]
    for name, font, preset, size, wordmark, cursor, letter, baseline in geometry_rows:
        geometry_lines.append(
            f"{name:<15} {font:<21} {preset:<8} {size:>3}  "
            f"{str(wordmark):<19} {str(cursor):<19} {str(letter):<19} {baseline}"
        )
    geometry_text = "\n".join(geometry_lines) + "\n"
    print()
    print(geometry_text, end="")
    if args.geometry_report:
        pathlib.Path(args.geometry_report).write_text(geometry_text)


if __name__ == "__main__":
    main()
