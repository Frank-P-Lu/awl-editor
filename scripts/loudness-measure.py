#!/usr/bin/env python3
"""loudness-measure.py — territory and contrast arithmetic over the item 118
idle-loudness Room sweep (`scripts/capture-loudness-118.sh`).

WHAT THIS IS NOT. Item 118 is explicit: "pixel/sidecar arithmetic may prove
territory and contrast but never claims the taste score." Nothing here emits a
1-5. It measures the things a taste judgement should not have to guess at —
how much of the window the ground owns, how hard the ground's marks push, how
colorful the room is, how far the page separates from its margins — so the
human scoring it argues with numbers instead of impressions.

Geometry comes from the sidecar, never from pixel-hunting: `page.column.left`
and `page.column.width` are the app's own answer for where the writing column
is, so the margin bands below are exact rather than inferred.

THE RIGHT MARGIN IS THE CLEAN GROUND SAMPLE. The left margin carries the
persistent Outline rail and (bottom-left) the page-mode gutter — both chrome,
both ink, neither part of the ground pattern this audit is weighing. The right
margin is pattern and nothing else, so every ground statistic is taken there;
`margin_frac` still reports BOTH margins, because territory is territory.

Per (world, arm) it reports:

  margin_frac   fraction of canvas width outside the writing column. The
                ground's whole stage. MEASURED, and the measurement says it is
                world-INDEPENDENT: the adaptive column resolves `measure` to
                the same pixel width (1008px at 70, 1440px at 100) for every
                world regardless of display face, so at one arm every world's
                ground gets exactly the same stage. Kept in the table because
                a constant that was worth checking is worth showing.
  g_sd          luminance standard deviation inside the right margin, x1000.
                A flat gradient sits near zero; a hard-edged mark field does
                not. This is the ground's contrast against itself.
  g_sd_lp       the same, after a 4x4 box downsample. THIS IS THE ONE TO READ
                FOR FIELD CONTRAST, and the pair (g_sd, g_sd_lp) is the point:
                a dithered ground (Mangrove) and a 1px-hairline ground
                (Saltpan, Magpie) both post a large raw g_sd that a viewer
                never perceives as contrast, because the structure carrying it
                is below the eye's integration scale. Item 186 recorded the
                same trap from the other side — a per-pixel metric passed its
                own mutation because doubling a whisper ramp made it gentler
                per pixel. A ground whose g_sd collapses under low-pass is
                textured; one that holds it is genuinely high-contrast.
  g_p99_p01     luminance span (99th - 1st percentile) in the right margin,
                x1000. `g_sd` can be small while a sparse, high-contrast mark
                still stabs; this catches that.
  g_edge        fraction of horizontally adjacent right-margin pixel pairs
                differing by more than EDGE_DELTA in 8-bit luminance. Busy-
                ness: how many mark boundaries the eye crosses scanning
                across. A gradient has almost none at any amplitude.
  g_chroma      mean (max-min) over sRGB channels in the right margin, 0-255.
                How saturated the ground is, independent of how light it is.
  step          |mean right-margin luminance - modal page-column luminance|,
                x1000. How far the page floats off its own ground. A large
                step draws the eye to the page edge every time it passes.
  ink_cr        WCAG contrast ratio between the page column's modal (ground)
                color and its ink extreme. Reported because readability is a
                confound a loudness reading must not silently absorb:
                Wagtail is the roster's LOUDEST ink contrast and its QUIETEST
                world, which is exactly why contrast alone cannot be the score.
  accent        fraction of page-column pixels within ACCENT_TOL of the
                theme's `primary`, x10000. The caret and any accent ink.
  ink=accent    `y` when the world's `primary` IS its ink (Wagtail, Cassowary:
                THEMES.md's ink-caret pattern). On those two the accent column
                is degenerate — every glyph counts as accent — so it must be
                read as "no separate accent hue exists", never as "this world
                is drenched in accent".

Usage:
  scripts/loudness-measure.py                     # every arm, tsv to stdout
  scripts/loudness-measure.py laptop wide         # named arms only
"""

import importlib.util
import json
import os
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
_ROOT = os.path.dirname(_HERE)

# Reuse the repo's one minimal PNG decoder rather than adding a fourth copy of
# it (CLAUDE.md: same behavior => same code). `hero-verify.py` is not an
# importable identifier, so it is loaded by path; it is guarded by
# `if __name__ == "__main__"` and runs nothing on import.
_spec = importlib.util.spec_from_file_location(
    "awl_hero_verify", os.path.join(_HERE, "hero-verify.py")
)
_hero = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_hero)
decode_png = _hero.decode_png
luminance = _hero.luminance
contrast_ratio = _hero.contrast_ratio

RUN_DIR = os.path.join(_ROOT, "gallery", "item-118-loudness")
ARMS = ["narrow", "laptop", "wide", "code"]

# An 8-bit luminance step of 3 is above the dither/quantization floor these
# grounds are drawn with (the dither round deliberately spreads sub-step
# differences across neighbours) and below any authored mark edge, so it
# counts mark boundaries rather than noise.
EDGE_DELTA = 3
# Channel-wise tolerance for "this pixel is the accent", loose enough to catch
# the caret's own anti-aliased body and tight enough to exclude ordinary ink.
ACCENT_TOL = 24


def lum8(rgb):
    """Perceptual luminance on a 0-255 scale (WCAG relative luminance x255)."""
    return luminance(rgb) * 255.0


def band_pixels(rows, bpp, x0, x1, y0, y1, xstep=1, ystep=1):
    out = []
    for y in range(y0, y1, ystep):
        row = rows[y]
        for x in range(x0, x1, xstep):
            o = x * bpp
            out.append((row[o], row[o + 1], row[o + 2]))
    return out


def percentile(sorted_vals, q):
    if not sorted_vals:
        return 0.0
    i = int(round(q * (len(sorted_vals) - 1)))
    return sorted_vals[i]


def edge_fraction(rows, bpp, x0, x1, y0, y1, ystep=1):
    """Fraction of horizontally adjacent pairs crossing EDGE_DELTA."""
    crossings = 0
    pairs = 0
    for y in range(y0, y1, ystep):
        row = rows[y]
        prev = None
        for x in range(x0, x1):
            o = x * bpp
            cur = lum8((row[o], row[o + 1], row[o + 2]))
            if prev is not None:
                pairs += 1
                if abs(cur - prev) > EDGE_DELTA:
                    crossings += 1
            prev = cur
    return crossings / pairs if pairs else 0.0


def boxdown_luma(rows, bpp, x0, x1, y0, y1, box=4):
    """Luminances of `box`x`box` block means — the eye's integration scale.

    Dither patterns and 1px hairlines average away here; an authored field
    keeps its contrast.
    """
    out = []
    for by in range(y0, y1 - box + 1, box):
        for bx in range(x0, x1 - box + 1, box):
            acc = 0.0
            for y in range(by, by + box):
                row = rows[y]
                for x in range(bx, bx + box):
                    o = x * bpp
                    acc += lum8((row[o], row[o + 1], row[o + 2]))
            out.append(acc / (box * box))
    return out


def modal_color(pixels, bucket=4):
    """Most common quantized color — the page's own ground, not its ink."""
    hist = {}
    for p in pixels:
        k = (p[0] // bucket, p[1] // bucket, p[2] // bucket)
        hist[k] = hist.get(k, 0) + 1
    k = max(hist, key=hist.get)
    # Average the real pixels in the winning bucket back out.
    sel = [p for p in pixels if (p[0] // bucket, p[1] // bucket, p[2] // bucket) == k]
    n = len(sel)
    return (
        sum(p[0] for p in sel) // n,
        sum(p[1] for p in sel) // n,
        sum(p[2] for p in sel) // n,
    )


def hexrgb(s):
    s = s.lstrip("#")
    return (int(s[0:2], 16), int(s[2:4], 16), int(s[4:6], 16))


def measure(png, sidecar):
    w, h, bpp, rows = decode_png(png)
    meta = json.load(open(sidecar))
    col = meta["page"]["column"]
    left, cw = int(col["left"]), int(col["width"])
    right_x0 = left + cw

    # Inset every band away from its own boundary so a page-frame line or a
    # card border never contaminates a ground statistic.
    inset = 8
    y0, y1 = inset, h - inset

    rm_x0, rm_x1 = right_x0 + inset, w - inset
    if rm_x1 - rm_x0 < 32:
        raise SystemExit(f"{png}: right margin too narrow to sample ({rm_x1-rm_x0}px)")

    # Sub-sample rows for the O(area) statistics; every ground here has a
    # period far above 3px, so a 3-row stride cannot alias a mark away.
    gm = band_pixels(rows, bpp, rm_x0, rm_x1, y0, y1, ystep=3)
    gl = sorted(lum8(p) for p in gm)
    g_mean = sum(gl) / len(gl)
    g_sd = (sum((v - g_mean) ** 2 for v in gl) / len(gl)) ** 0.5
    g_span = percentile(gl, 0.99) - percentile(gl, 0.01)
    g_chroma = sum(max(p) - min(p) for p in gm) / len(gm)
    g_edge = edge_fraction(rows, bpp, rm_x0, rm_x1, y0, y1, ystep=6)

    lp = boxdown_luma(rows, bpp, rm_x0, rm_x1, y0, y1, box=4)
    lp_mean = sum(lp) / len(lp)
    g_sd_lp = (sum((v - lp_mean) ** 2 for v in lp) / len(lp)) ** 0.5

    page = band_pixels(rows, bpp, left + inset, right_x0 - inset, y0, y1, ystep=3)
    page_ground = modal_color(page)
    pl = sorted(lum8(p) for p in page)
    # The ink extreme is the tail AWAY from the page ground: dark ink on a
    # light page, light ink on a dark one.
    pg_l = lum8(page_ground)
    ink_l = percentile(pl, 0.005) if pg_l > 127 else percentile(pl, 0.995)
    ink_px = min(page, key=lambda p: abs(lum8(p) - ink_l))
    ink_cr = contrast_ratio(page_ground, ink_px)

    step = abs(g_mean - pg_l)

    primary = hexrgb(meta["theme"]["primary"])
    acc = sum(
        1
        for p in page
        if all(abs(p[i] - primary[i]) <= ACCENT_TOL for i in range(3))
    )
    accent = acc / len(page)
    ink_is_accent = all(abs(ink_px[i] - primary[i]) <= ACCENT_TOL for i in range(3))

    margin_frac = (w - cw) / w

    return {
        "margin_frac": margin_frac,
        "ink_is_accent": ink_is_accent,
        "g_sd_lp": g_sd_lp * 1000 / 255,
        "g_sd": g_sd * 1000 / 255,
        "g_p99_p01": g_span * 1000 / 255,
        "g_edge": g_edge,
        "g_chroma": g_chroma,
        "step": step * 1000 / 255,
        "ink_cr": ink_cr,
        "accent": accent * 10000,
        "ground": meta["page"]["background"]["kind"],
        "ambient": meta["page"].get("ambient", {}).get("style", "none"),
        "mode": meta["theme"]["mode"],
        "face": meta["theme"]["font_family"],
    }


def main(argv):
    arms = argv[1:] or ARMS
    hdr = (
        "arm\tworld\tmode\tground\tambient\tface\t"
        "margin_frac\tg_sd\tg_sd_lp\tg_p99_p01\tg_edge\tg_chroma\tstep\tink_cr\taccent\tink=accent"
    )
    print(hdr)
    for arm in arms:
        d = os.path.join(RUN_DIR, arm)
        if not os.path.isdir(d):
            raise SystemExit(f"missing arm dir {d} — run scripts/capture-loudness-118.sh")
        for f in sorted(os.listdir(d)):
            if not f.endswith(".png"):
                continue
            world = f[:-4]
            m = measure(os.path.join(d, f), os.path.join(d, world + ".json"))
            print(
                f"{arm}\t{world}\t{m['mode']}\t{m['ground']}\t{m['ambient']}\t{m['face']}\t"
                f"{m['margin_frac']:.3f}\t{m['g_sd']:.1f}\t{m['g_sd_lp']:.1f}\t"
                f"{m['g_p99_p01']:.1f}\t"
                f"{m['g_edge']:.4f}\t{m['g_chroma']:.1f}\t{m['step']:.1f}\t"
                f"{m['ink_cr']:.2f}\t{m['accent']:.1f}\t"
                f"{'y' if m['ink_is_accent'] else 'n'}"
            )


if __name__ == "__main__":
    main(sys.argv)
