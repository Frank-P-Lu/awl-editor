#!/usr/bin/env python3
"""hero-verify.py — item 157 follow-up: prove the hero social image is a
COMPOSED image, not an editor screenshot with the fixture's own name leaked
into it.

Three checks, run against one candidate PNG + its sidecar JSON:

  1. `gutter.visible` is `false` in the sidecar (the STATE oracle — CAPTURE.md:
     "gutter.visible is true exactly when page mode is on and the buffer has
     a name"). This alone does not prove the pixels are clean — CAPTURE.md's
     own tripwire is that the sidecar is a state oracle, not an appearance
     one (`selected_index: 2` once rendered on a fully invisible row) — so:
  2. PIXEL evidence that the page-mode gutter's own drawn region (bottom-left
     margin, where the fixture's filename + `scripts/` project used to leak)
     is now clean ground: its luminance range must sit inside the ambient
     background pattern's own natural variance, not the ~180+ range a real
     line of ink produces. Calibrated against the ACTUAL leaked capture this
     item fixes (`--calibrate` below reproduces the numbers this law is
     named for), so this check is proven non-vacuous, not just plausible.
  3. Legibility arithmetic reported (not gated — taste stays the user's
     call): OG aspect (1200x630 ~= 1.91:1), a centred-square 90%-safe-area
     ink-overflow fraction, and a ~360px-thumbnail contrast ratio (WCAG
     relative-luminance formula) for the body-text row, sampling the same
     full-resolution ink pixels back off a box-downscaled thumbnail — this
     is what a heavily downscaled link-unfurl actually shows, not the sharp
     full-res ink color.

No third-party imports: the PNG is decoded here (8-bit RGB/RGBA,
non-interlaced), matching scripts/icons/verify.py and
scripts/probe-shot-check.py's own no-PIL/numpy convention.
"""

import json
import struct
import sys
import zlib


def decode_png(path):
    """Minimal PNG decoder: 8-bit RGB/RGBA, non-interlaced. -> (w, h, bpp, rows)."""
    raw = open(path, "rb").read()
    if raw[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError(f"{path}: not a PNG")
    pos, w, h, ctype, idat = 8, 0, 0, 0, []
    while pos < len(raw):
        (length,) = struct.unpack(">I", raw[pos : pos + 4])
        tag = raw[pos + 4 : pos + 8]
        data = raw[pos + 8 : pos + 8 + length]
        if tag == b"IHDR":
            w, h, depth, ctype, _, _, interlace = struct.unpack(">IIBBBBB", data)
            if depth != 8 or ctype not in (2, 6) or interlace != 0:
                raise ValueError(f"{path}: unsupported PNG (depth={depth} ctype={ctype})")
        elif tag == b"IDAT":
            idat.append(data)
        elif tag == b"IEND":
            break
        pos += 12 + length
    bpp = 4 if ctype == 6 else 3
    stream = zlib.decompress(b"".join(idat))
    stride = w * bpp
    rows, prev = [], bytearray(stride)
    p = 0
    for _ in range(h):
        filt = stream[p]
        line = bytearray(stream[p + 1 : p + 1 + stride])
        p += 1 + stride
        if filt == 1:
            for i in range(bpp, stride):
                line[i] = (line[i] + line[i - bpp]) & 0xFF
        elif filt == 2:
            for i in range(stride):
                line[i] = (line[i] + prev[i]) & 0xFF
        elif filt == 3:
            for i in range(stride):
                a = line[i - bpp] if i >= bpp else 0
                line[i] = (line[i] + ((a + prev[i]) >> 1)) & 0xFF
        elif filt == 4:
            for i in range(stride):
                a = line[i - bpp] if i >= bpp else 0
                b = prev[i]
                c = prev[i - bpp] if i >= bpp else 0
                pp = a + b - c
                pa, pb, pc = abs(pp - a), abs(pp - b), abs(pp - c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[i] = (line[i] + pr) & 0xFF
        elif filt != 0:
            raise ValueError(f"{path}: bad filter {filt}")
        rows.append(bytes(line))
        prev = line
    return w, h, bpp, rows


def px(rows, bpp, x, y):
    o = x * bpp
    row = rows[y]
    return (row[o], row[o + 1], row[o + 2])


def luminance(rgb):
    def lin(c):
        c = c / 255.0
        return c / 12.92 if c <= 0.04045 else ((c + 0.055) / 1.055) ** 2.4
    r, g, b = rgb
    return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)


def contrast_ratio(a, b):
    la, lb = luminance(a), luminance(b)
    lighter, darker = max(la, lb), min(la, lb)
    return (lighter + 0.05) / (darker + 0.05)


# Calibrated against the ACTUAL leak this item fixes (see `--calibrate`,
# which reproduces these numbers from the pre-fix candidates that shipped):
#
#   Saltpan  LEAK  range=166.0   Saltpan  FIXED range=35.2
#   Firetail LEAK  range=59.5    Firetail FIXED range=0.0
#   Wagtail  LEAK  range=255.0   Wagtail  FIXED range=0.0
#
# The floor sits between the clean worst case (35.2) and the leaked best case
# (59.5), so the law fails loudly on the bug it names (any real leaked
# filename/project text) instead of passing vacuously, and does not trip on
# a world's own decorative border (Wagtail draws a 2px column-edge outline —
# see the `-6` inset below, which steps just inside it) or ambient ground
# pattern (Saltpan's pinstripe / Firetail's blurred bokeh both stay well
# under the floor once the gutter is truly empty).
CLEAN_GROUND_MAX_RANGE = 45.0


def gutter_region_range(rows, bpp, w, h, col_left):
    """Luminance (min, max, range) over the bottom-left margin band where the
    page-mode gutter draws — x in [0, col_left - 6), y in the bottom 90px.
    The x inset steps just inside a world's own column-edge border stroke
    (Wagtail draws one ~2px in from `col_left`) so the border itself is never
    mistaken for leaked ink; the box is otherwise generous (not the exact
    glyph geometry) so it catches a leak regardless of the exact font/zoom
    that drew it."""
    x1 = max(1, min(w, int(col_left) - 6))
    y0 = max(0, h - 90)
    lo, hi = 255.0, 0.0
    for y in range(y0, h, 2):
        for x in range(0, x1, 2):
            l = luminance(px(rows, bpp, x, y)) * 255.0
            lo = min(lo, l)
            hi = max(hi, l)
    return lo, hi, hi - lo


def square_safe_area_fraction(rows_json, w, h):
    """Fraction of each shaped row's own ink extent (sidecar `layout.rows`
    xs bounds) that falls outside a centred-square crop's 90%-safe interior
    — the common "will an in-app square unfurl clip this" guideline."""
    sc = min(w, h)
    sq_left = (w - sc) // 2
    sq_right = sq_left + sc
    safe_margin = sc * 0.05
    safe_left = sq_left + safe_margin
    safe_right = sq_right - safe_margin
    total = outside = 0.0
    for r in rows_json:
        xs = r.get("xs") or []
        if not xs:
            continue
        x0, x1 = min(xs), max(xs)
        total += x1 - x0
        left_out = max(0.0, safe_left - x0)
        right_out = max(0.0, x1 - safe_right)
        outside += min(left_out, x1 - x0) + min(right_out, x1 - x0)
    return (outside / total) if total else 0.0


def thumb_contrast(rows, bpp, w, h, row_json, thumb_w=360):
    """~360px-thumbnail contrast of a body-text row: box-downscale the whole
    canvas, classify INK pixels at FULL res (>18% luminance delta from a
    local background sample), then read those SAME locations back off the
    downscaled thumbnail and average — this is what a heavily downscaled
    link-unfurl actually shows (thin strokes partially blended into the
    background by the resize filter), not a single sharp full-res sample."""
    scale = thumb_w / w
    thumb_h = max(1, round(h * scale))
    block = max(1, round(1.0 / scale))

    def thumb_px(tx, ty):
        sx0, sy0 = int(tx / scale), int(ty / scale)
        sx1, sy1 = min(w, sx0 + block), min(h, sy0 + block)
        r = g = b = n = 0
        for y in range(sy0, sy1):
            for x in range(sx0, sx1):
                c = px(rows, bpp, x, y)
                r += c[0]
                g += c[1]
                b += c[2]
                n += 1
        return (r / n, g / n, b / n) if n else (0, 0, 0)

    fx0, fx1 = int(min(row_json["xs"])), int(max(row_json["xs"]))
    fy0 = int(row_json["top"])
    fy1 = int(row_json["top"] + row_json["height"])
    bg_full = px(rows, bpp, max(0, fx0 - 8), min(h - 1, fy0 + 2))
    bg_lum_full = luminance(bg_full)

    ink_thumb = []
    for fy in range(fy0, fy1, 2):
        for fx in range(fx0, fx1, 2):
            if abs(luminance(px(rows, bpp, fx, fy)) - bg_lum_full) > 0.18:
                tx, ty = int(fx * scale), int(fy * scale)
                if 0 <= tx < thumb_w and 0 <= ty < thumb_h:
                    ink_thumb.append(thumb_px(tx, ty))
    bg_thumb = thumb_px(max(0, int(fx0 * scale) - 6), int((fy0 + fy1) / 2 * scale))
    if ink_thumb:
        avg = tuple(sum(c[i] for c in ink_thumb) / len(ink_thumb) for i in range(3))
    else:
        avg = bg_thumb
    return contrast_ratio(avg, bg_thumb), avg, bg_thumb, len(ink_thumb)


def report(png_path, json_path, label):
    w, h, bpp, rows = decode_png(png_path)
    d = json.load(open(json_path))
    ok = True

    gutter_visible = d["gutter"]["visible"]
    print(f"=== {label} ===")
    print(f"gutter.visible (sidecar) = {gutter_visible}")
    if gutter_visible:
        ok = False
        print("FAIL: gutter.visible is true — the fixture name/project would leak.")

    col_left = d["page"]["column"]["left"]
    lo, hi, rng = gutter_region_range(rows, bpp, w, h, col_left)
    verdict = "clean ground" if rng <= CLEAN_GROUND_MAX_RANGE else "LOOKS LIKE INK"
    print(
        f"bottom-left margin band pixels: luminance lo={lo:.1f} hi={hi:.1f} "
        f"range={rng:.1f} (floor {CLEAN_GROUND_MAX_RANGE}) -> {verdict}"
    )
    if rng > CLEAN_GROUND_MAX_RANGE:
        ok = False
        print("FAIL: bottom-left margin band shows ink-level contrast — a leak survived.")

    rows_json = d["layout"]["rows"]
    frac_outside = square_safe_area_fraction(rows_json, w, h)
    print(f"OG ratio: {w/h:.3f} (target ~1.91)")
    print(f"square-crop 90%-safe-area ink-outside fraction: {frac_outside:.3f}")

    body_rows = [r for r in rows_json if r["line"] >= 2 and r.get("xs")]
    if body_rows:
        row = body_rows[len(body_rows) // 2]
        ratio, ink, bg, n = thumb_contrast(rows, bpp, w, h, row)
        wcag = "PASS (>=4.5 WCAG AA)" if ratio >= 4.5 else "below WCAG AA 4.5:1"
        print(f"~360px-thumbnail body-text contrast: {ratio:.2f}:1 ({wcag})")
    print()
    return ok


def calibrate(png_path, json_path, label):
    """Print the SAME bottom-left-band numbers for a KNOWN-LEAKED capture, so
    CLEAN_GROUND_MAX_RANGE's floor is demonstrably non-vacuous — it sits
    between a real leak's range and an honestly-hidden gutter's range."""
    w, h, bpp, rows = decode_png(png_path)
    d = json.load(open(json_path))
    col_left = d["page"]["column"]["left"]
    lo, hi, rng = gutter_region_range(rows, bpp, w, h, col_left)
    print(f"=== CALIBRATION: {label} ===")
    print(f"gutter.visible = {d['gutter']['visible']}")
    print(f"bottom-left margin band: lo={lo:.1f} hi={hi:.1f} range={rng:.1f}")
    print()


def main(argv):
    if len(argv) >= 2 and argv[0] == "--calibrate":
        calibrate(argv[1], argv[2], argv[3] if len(argv) > 3 else argv[1])
        return 0
    if len(argv) < 2:
        print(
            "usage: hero-verify.py CANDIDATE.png CANDIDATE.json [LABEL]\n"
            "       hero-verify.py --calibrate LEAKED.png LEAKED.json [LABEL]",
            file=sys.stderr,
        )
        return 2
    label = argv[2] if len(argv) > 2 else argv[0]
    ok = report(argv[0], argv[1], label)
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
