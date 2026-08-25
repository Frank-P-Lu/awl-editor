#!/usr/bin/env python3
"""Pixel-arithmetic checks over this gallery's PNGs — non-vacuity, not taste.

Run as `python3 captures/item-487-magpie-diagonal/measure.py <out-dir>` after
shoot.sh has written its PNGs + sidecars into `<out-dir>` (shoot.sh does this
itself as its last step). Prints, per check:

  * the selected row's chevron: how far the nearest ISOLATED mark-sized ink
    run sits from the row's own label ink box, current vs the SHORT-REACH
    candidate — confirms the far-flung mark in "current" is real (a genuine
    outlier far past any near-label noise) and confirms it collapses to
    directly-adjacent ink in "short" (fused onto the label's own run, not a
    separate far run at all).
  * the query caret: its x, found by XOR-ing the theme's own `primary` ink
    mask between two shots of the SAME world (isolates what moved — the
    caret — from what didn't, e.g. the document's own H1 accent glyph, which
    a plain single-shot color scan cannot tell apart) — current vs the
    QUERY-RIGHT candidate.
  * frost placement: how many pixels differ, and in what bounding box,
    between the current frost and the TOP-ABOVE-FIRST-LINE candidate —
    confirms the two shots actually differ (non-vacuity) and confirms the
    diff is confined to the card's own head band, not a whole-frame change.

Magpie only for the chevron/caret checks: Mangrove's `Pinstripe` background
is a fine dot texture, and a flat color-distance threshold (the simplest
correct tool for Magpie's flat/banded ground) cannot separate "ink" from
"background dot" there without a texture-aware oracle this throwaway script
does not build. Mangrove still gets the frost-diff check (texture-agnostic)
and its own gallery shots for a human's visual read — see README.md.

This is throwaway analysis tooling for the gallery, not a shipped law.
"""

import json
import sys
from pathlib import Path

from PIL import Image, ImageChops
import numpy as np


def load(out_dir: Path, name: str):
    png = Image.open(out_dir / f"{name}.png").convert("RGB")
    sidecar = json.loads((out_dir / f"{name}.json").read_text())
    return np.array(png).astype(np.float32), sidecar


def background_color(arr: np.ndarray) -> np.ndarray:
    corner = arr[2:10, -12:-2, :].reshape(-1, 3)
    return np.median(corner, axis=0)


def runs_in_band(arr: np.ndarray, y0: int, y1: int, bg: np.ndarray, thresh: float = 28.0, gap: int = 6):
    band = arr[y0:y1, :, :]
    dist = np.sqrt(((band - bg) ** 2).sum(axis=2))
    cols = np.nonzero((dist > thresh).any(axis=0))[0]
    if len(cols) == 0:
        return []
    out = []
    start = prev = cols[0]
    for c in cols[1:]:
        if c - prev > gap:
            out.append((int(start), int(prev)))
            start = c
        prev = c
    out.append((int(start), int(prev)))
    return out


def chevron_report(out_dir: Path, name: str):
    arr, sc = load(out_dir, name)
    bg = background_color(arr)
    # The row displayed as SELECTED this frame — read from the sidecar's own
    # `sel_row` (a display-window index) rather than a hardcoded roster
    # index, because a narrower logical canvas at 2x windows fewer rows and
    # `selected_index` (the full-roster item index) is no longer a valid
    # position in the drawn `rows` list.
    rows = sc["overlay"]["window"]["rows"]
    sel_display = sc["overlay"]["window"]["sel_row"]
    row = rows[sel_display]
    y0, y1 = int(row["y"]), int(row["y"] + row["h"])
    label_x0 = row["label"]["x"]
    label_x1 = row["label"]["x"] + row["label"]["w"]
    label_w = row["label"]["w"]
    segs = runs_in_band(arr, y0, y1, bg)
    # The run that overlaps (or directly abuts) the label's own box — its
    # width beyond the label's OWN reported ink width is ink fused onto the
    # label by a mark seated right next to it (the "short" candidate).
    label_run = next((s for s in segs if not (s[1] < label_x0 - 2 or s[0] > label_x1 + 2)), None)
    fused_extra = 0.0
    if label_run is not None:
        fused_extra = max(0.0, (label_run[1] - label_run[0]) - label_w)
    # Every OTHER run, more than 20px from the label — real isolated marks
    # and stray document glyphs alike (the fixture's own prose runs through
    # this same y-band on a wide card, so this list is NOT "the chevron
    # alone"; it is reported in full, position and all, so a reader can
    # cross-check it against the shot rather than trust a single collapsed
    # number). NOT the near-label seam noise (the spine's own hairline
    # passes within ~10-17px of the label on every shot, candidate or not).
    far_runs = []
    for a, b in segs:
        if label_run is not None and (a, b) == label_run:
            continue
        gap = min(abs(a - label_x0), abs(b - label_x1), abs(a - label_x1), abs(b - label_x0))
        if gap > 20 and (b - a) <= 40:
            far_runs.append((a, b))
    return {
        "far_runs_outside_label": far_runs,
        "label_fused_extra_px": round(fused_extra, 1),
    }


def query_caret_x(out_dir: Path, current_name: str, candidate_name: str):
    """XOR the theme's `primary` ink mask between two same-world shots to
    isolate what moved (the caret) from static primary-colored document
    decoration (e.g. an H1 accent glyph) a single-shot scan cannot tell
    apart from the caret."""
    arr_a, sc = load(out_dir, current_name)
    arr_b, _ = load(out_dir, candidate_name)
    primary_hex = sc["theme"]["primary"].lstrip("#")
    primary = np.array([int(primary_hex[i : i + 2], 16) for i in (0, 2, 4)], dtype=np.float32)
    mask_a = np.sqrt(((arr_a - primary) ** 2).sum(axis=2)) < 40
    mask_b = np.sqrt(((arr_b - primary) ** 2).sum(axis=2)) < 40
    moved_a = mask_a & ~mask_b
    moved_b = mask_b & ~mask_a

    def dense_groups(moved, min_col_count=12, gap=3):
        colcount = moved.sum(axis=0)
        cols = np.nonzero(colcount >= min_col_count)[0]
        if len(cols) == 0:
            return []
        groups = []
        start = prev = cols[0]
        for c in cols[1:]:
            if c - prev > gap:
                groups.append((start, prev))
                start = c
            prev = c
        groups.append((start, prev))
        return groups

    groups_a = dense_groups(moved_a)
    groups_b = dense_groups(moved_b)
    # This candidate only ever moves the caret to a LARGER x (right-aligned
    # against the card's own text column) — see gallery.rs's `head_left_override`.
    # So the caret's own contribution to each side is, respectively, the
    # SMALLEST-x group on the "current" side and the LARGEST-x group on the
    # "candidate" side; any other dense group is contamination from a
    # document decoration whose BLUR (not position) differed between shots.
    caret_a = min(groups_a, key=lambda g: g[0]) if groups_a else None
    caret_b = max(groups_b, key=lambda g: g[1]) if groups_b else None
    return {
        "current_caret_x": None if caret_a is None else sum(caret_a) / 2,
        "candidate_caret_x": None if caret_b is None else sum(caret_b) / 2,
    }


def frost_diff(out_dir: Path, a_name: str, b_name: str):
    """How much of the frame changed, and whether the change stayed inside
    the card's own head band.

    `Image.getbbox()` on the raw difference is the WRONG tool here: it
    treats a 1-unit rounding difference the same as a real change, and on
    this fixture that noise floor (sub-perceptual dithering variance from
    the changed footprint shape feeding back into the ordered-posterization
    signature `pipeline_prepare.rs` documents) spans nearly the WHOLE
    canvas — a bbox built from it would wrongly read as "the diff reaches
    the row band" on every world. The bbox below is built from the SAME
    `> 10` per-pixel threshold `differing_pixels` already counts, so the
    two numbers describe the same signal.
    """
    a = Image.open(out_dir / f"{a_name}.png").convert("RGB")
    b = Image.open(out_dir / f"{b_name}.png").convert("RGB")
    arr = np.array(ImageChops.difference(a, b))
    mask = arr.sum(axis=2) > 10
    ys, xs = np.nonzero(mask)
    bbox = None if len(ys) == 0 else (int(xs.min()), int(ys.min()), int(xs.max()) + 1, int(ys.max()) + 1)
    return {"bbox": bbox, "differing_pixels": int(mask.sum())}


def frost_diff_stays_above_row_band(out_dir: Path, current_name: str, diff: dict) -> bool:
    """Whether `frost_diff`'s own bbox stayed above the row band `current_name`'s
    own sidecar reports — the claim the README makes for the top0 candidate,
    checked per shot rather than assumed from one world's own number."""
    if diff["bbox"] is None:
        return True
    sc = json.loads((out_dir / f"{current_name}.json").read_text())
    first_top = sc["overlay"]["window"]["band"]["first_top"]
    return diff["bbox"][3] <= first_top


def main():
    out_dir = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("captures/item-487-magpie-diagonal")
    worlds = {"magpie": 13, "mangrove": 11}
    for dpi in ("1x", "2x"):
        print(f"== magpie, {dpi} (chevron + caret; Mangrove's textured ground skips these) ==")
        cur = f"magpie-current-{dpi}"
        short = f"magpie-chevron-short-{dpi}"
        qright = f"magpie-query-right-{dpi}"
        print(f"  chevron[current]   = {chevron_report(out_dir, cur)}")
        print(f"  chevron[short]     = {chevron_report(out_dir, short)}")
        print(f"  query_caret        = {query_caret_x(out_dir, cur, qright)}")
        first_item_x = json.loads((out_dir / f"{cur}.json").read_text())["overlay"]["window"]["rows"][0]["label"]["x"]
        print(f"  first_item_label_x = {first_item_x}")

    for world in worlds:
        for dpi in ("1x", "2x"):
            cur = f"{world}-current-{dpi}"
            top0 = f"{world}-frost-top0-{dpi}"
            diff = frost_diff(out_dir, cur, top0)
            contained = frost_diff_stays_above_row_band(out_dir, cur, diff)
            status = "CONTAINED above row band" if contained else "REACHES ROW BAND"
            print(f"  frost_top0_diff[{cur} vs {top0}] = {diff}  [{status}]")


if __name__ == "__main__":
    main()
