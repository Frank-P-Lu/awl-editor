#!/usr/bin/env python3
"""pretag-journeys.py — THE PRE-TAG JOURNEY SWEEP, as a committed instrument.

CLAUDE.md's standing spot-check policy lists "pre-tag (a journey sweep across
worlds)" as an audit trigger, and until now nothing in the tree discharged it:
`scripts/capture-worlds.sh` walks two surfaces (the Room and the command
palette) and nothing walked the palette's facet lenses, the theme picker, the
staged Settings workspace, or the caret and selection at document extremes.
Each tag therefore paid for a bespoke driver that then evaporated. This is that
driver, kept.

It is NOT a gate. It is run deliberately before a tag, so it may be slow and
thorough where a per-push check could not be. Nothing here belongs in CI.

    scripts/pretag-journeys.py                 # release build, whole roster
    scripts/pretag-journeys.py --debug         # debug build (faster to get to)
    scripts/pretag-journeys.py --bin PATH      # use an already-built binary
    scripts/pretag-journeys.py --worlds A,B    # subset, for exercising the sweep
    scripts/pretag-journeys.py --journeys x,y  # subset, same purpose
    scripts/pretag-journeys.py --jobs 4        # worlds swept concurrently

Output: a REPLACEABLE gitignored run directory (`gallery/pretag/`), wiped and
rebuilt on every invocation, plus `report.txt` and `report.json` beside the
captures. The captures are never committed.

WHAT THE SHAPE OF THIS SWEEP IS, AND WHY EACH PART OF IT IS THAT SHAPE
---------------------------------------------------------------------

* **The roster comes from the binary.** `awl --list-worlds` prints
  `theme::world_names()` over `theme::THEMES`; a hand-written list is how a
  roster sweep silently stops covering a world someone added last week. The
  only world names in this file are the ones a `--worlds` subset names on the
  command line, and those are checked against the printed roster before use.

* **Both DPIs, as the same LOGICAL window.** Every journey runs at
  `WxH @1` and `2W x 2H @2`. A check that only ever ran at `--capture-dpi 1`
  is the exact configuration in which a chrome pad left in device pixels looks
  correct, and that shipped once. So the pixel arithmetic runs at both scales,
  not just the geometry.

* **The two doors are never compared to each other.** A plain `--screenshot`
  replays into the shared core at zoom 1.0; `--screenshot-app` builds a real
  headless `App`, which takes `app::INITIAL_ZOOM` — 0.8. That is a 25% size
  difference between two pictures of the "same" state, and reading one against
  the other has already produced a defect report about nothing. Every A/B pair
  and every DPI pair here is within ONE door; the doors are only ever reported
  side by side, never differenced.

* **An empty `--config` is pinned on every capture.** A bare `--screenshot`
  reads the operator's own `~/.config/awl/config.toml` and is therefore not
  hermetic. The menu-bar journeys pin a second, one-key config instead of an
  empty one, because the drawn bar's default is the tree's one platform fork
  and an empty config would make those journeys sweep nothing on a macOS host.

* **Appearance is asserted over the PNG, geometry over the sidecar.** The
  sidecar is a state oracle that has reported a perfectly selected row while
  the row rendered invisible; `overlay.window` is a geometry oracle that says
  where the plan put a rect but nothing about whether anything was drawn there.
  Every "is it there / did it change" claim below is arithmetic over pixels,
  and every floor compares one rendered pixel to another rendered pixel from
  the same run — never to an authored theme constant.

TWO THINGS THE HAND-RUN SWEEP PAID TO LEARN, CARRIED HERE
---------------------------------------------------------

1. **The theme picker's selection cannot be A/B'd at all.** Moving the
   selection PREVIEWS the next world, so the "before" and "after" frames are
   two different worlds and their difference says nothing about selection. It
   is graded here by a weaker within-frame probe, and the report says so out
   loud rather than implying the same rigour everywhere. The command palette,
   which previews nothing, carries the true A/B that the theme picker cannot.
   THROUGH THIS DOOR, that is. The coupling is a property of the ACTION path, not
   of the renderer, so one tier down a Rust law holds the world fixed while the
   selection moves and does run the true A/B on every world —
   `render::tests::theme_picker_selection_law`. This probe's abstention is a
   statement about captures, and the report says which.

2. **A zero-row Settings band at a narrow width is CORRECT.** Below its
   staging threshold the workspace shows one region at a time, and the stage
   showing the OTHER region publishes an empty row window faithfully.
   `src/render/tests/workspace_stage_reach.rs` owns that state and records that
   this exact reading was filed as a defect once and refuted. So an empty row
   window is accepted here when the surface is a workspace, and it is paired
   with its other stage under a PRESENCE FLOOR: some stage must have rows.
   "This stage has none" is satisfied perfectly by a card whose rows exist on
   no stage at any width, which is why the floor is not optional.

The PNG decoder is this file's own rather than the one in
`scripts/probe-shot-check.py`: that one materialises a tuple per pixel, which
is a fine shape for a handful of comparison shots and the wrong one for a
several-hundred-capture sweep whose larger canvas is 2400x1600. Flat
`bytearray`, same pure-stdlib rule (zlib only, no PIL, no numpy).
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import struct
import subprocess
import sys
import tempfile
import time
import zlib
from collections import Counter
from concurrent.futures import ProcessPoolExecutor
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SPECIMEN = Path(__file__).resolve().parent / "pretag-journey-specimen.md"

# The run directory. `/gallery` is gitignored, deliberately: these captures are
# a review artifact, never repository content.
RUN_DIR = ROOT / "gallery" / "pretag"

# The two logical windows. Physical canvas = logical * dpi at each scale, so
# both entries of a DPI pair are the SAME window at two device scales.
WIDE = ("wide", 1200, 800, 66)
NARROW = ("narrow", 640, 800, 40)
DPIS = (1, 2)

ORDINARY = "ordinary"  # --screenshot: shared-core replay, zoom 1.0
LIVE_APP = "live-app"  # --screenshot-app: a real headless App, launch zoom 0.8

# Pixel floors. Both operands of every comparison below are rendered pixels
# from this run; these are the DETECTION thresholds that separate "something is
# drawn here" from "nothing is", not legibility standards — contrast floors are
# Rust laws over theme roles, and belong there.
INK_DISTANCE = 24  # max-channel distance from a region's OWN modal colour
INK_FRACTION = 0.001  # of the region's pixels, at least, must be that far
DPI_TOLERANCE = 0.02  # the +/-2% the scaled family is asserted within
EPS = 0.75  # geometry slack, in physical pixels


def change_floor(row_height: float) -> int:
    """How many pixels of a row band must change for the change to be real.

    Derived from the frame rather than written down: the thinnest mark this
    sweep ever asks to see is the caret, which is at least a one-pixel column
    the full height of its own shaped row. So the floor IS that row's height,
    which also means it scales with the device scale on its own — a floor
    written as a number would have been tuned at one DPI and loose at the other.
    """
    return max(1, int(row_height))


# --------------------------------------------------------------------------
# PNG
# --------------------------------------------------------------------------


def decode_png(path: Path) -> tuple[int, int, int, bytearray]:
    """8-bit RGB/RGBA, non-interlaced -> (width, height, bytes-per-pixel, flat buffer)."""
    raw = path.read_bytes()
    if raw[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError(f"{path}: not a PNG")
    pos, width, height, ctype, idat = 8, 0, 0, 0, []
    while pos < len(raw):
        (length,) = struct.unpack(">I", raw[pos : pos + 4])
        tag = raw[pos + 4 : pos + 8]
        data = raw[pos + 8 : pos + 8 + length]
        if tag == b"IHDR":
            width, height, depth, ctype, _, _, interlace = struct.unpack(">IIBBBBB", data)
            if depth != 8 or ctype not in (2, 6) or interlace != 0:
                raise ValueError(f"{path}: unsupported PNG (depth={depth} ctype={ctype})")
        elif tag == b"IDAT":
            idat.append(data)
        elif tag == b"IEND":
            break
        pos += 12 + length
    bpp = 4 if ctype == 6 else 3
    data = zlib.decompress(b"".join(idat))
    stride = width * bpp
    out = bytearray(height * stride)
    prev = bytearray(stride)
    read = 0
    for y in range(height):
        ft = data[read]
        read += 1
        line = bytearray(data[read : read + stride])
        read += stride
        if ft == 1:
            for i in range(bpp, stride):
                line[i] = (line[i] + line[i - bpp]) & 255
        elif ft == 2:
            for i in range(stride):
                line[i] = (line[i] + prev[i]) & 255
        elif ft == 3:
            for i in range(stride):
                a = line[i - bpp] if i >= bpp else 0
                line[i] = (line[i] + ((a + prev[i]) >> 1)) & 255
        elif ft == 4:
            for i in range(stride):
                a = line[i - bpp] if i >= bpp else 0
                b = prev[i]
                c = prev[i - bpp] if i >= bpp else 0
                pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
                pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                line[i] = (line[i] + pr) & 255
        elif ft != 0:
            raise ValueError(f"{path}: unknown PNG filter {ft} on row {y}")
        out[y * stride : (y + 1) * stride] = line
        prev = line
    return width, height, bpp, out


class Frame:
    """One decoded capture, with the region arithmetic the assertions need."""

    def __init__(self, path: Path):
        self.path = path
        self.w, self.h, self.bpp, self.buf = decode_png(path)

    def rect(self, x0: float, y0: float, x1: float, y1: float) -> tuple[int, int, int, int]:
        """Clamp a float rect to a non-empty integer one inside the frame."""
        ix0 = max(0, min(self.w - 1, int(x0)))
        iy0 = max(0, min(self.h - 1, int(y0)))
        ix1 = max(ix0 + 1, min(self.w, int(round(x1))))
        iy1 = max(iy0 + 1, min(self.h, int(round(y1))))
        return ix0, iy0, ix1, iy1

    def region(self, r: tuple[int, int, int, int]) -> bytes:
        """The region's RGB bytes, rows concatenated (alpha dropped)."""
        x0, y0, x1, y1 = r
        stride = self.w * self.bpp
        out = bytearray()
        for y in range(y0, y1):
            row = self.buf[y * stride + x0 * self.bpp : y * stride + x1 * self.bpp]
            if self.bpp == 3:
                out += row
            else:
                del row[3::4]
                out += row
        return bytes(out)

    def ground_and_ink(self, r: tuple[int, int, int, int]) -> tuple[tuple[int, int, int], float, int]:
        """The region's OWN modal colour, the fraction of pixels further than
        INK_DISTANCE from it, and the furthest distance any pixel reached.

        Both operands are rendered pixels of this same frame: the ground is the
        colour the region is mostly made of, the ink is whatever stands off it.
        """
        px = self.region(r)
        counts = Counter(px[i : i + 3] for i in range(0, len(px), 3))
        ground = counts.most_common(1)[0][0]
        gr, gg, gb = ground[0], ground[1], ground[2]
        # Over the DISTINCT colours, weighted by how many pixels wear each. A
        # rendered frame has a few thousand of those and a few million pixels.
        far = 0
        worst = 0
        for colour, n in counts.items():
            d = max(abs(colour[0] - gr), abs(colour[1] - gg), abs(colour[2] - gb))
            if d > worst:
                worst = d
            if d >= INK_DISTANCE:
                far += n
        total = sum(counts.values())
        return (gr, gg, gb), (far / total if total else 0.0), worst

    def mode_colour(self, r: tuple[int, int, int, int]) -> tuple[int, int, int]:
        px = self.region(r)
        counts = Counter(px[i : i + 3] for i in range(0, len(px), 3))
        top = counts.most_common(1)[0][0]
        return (top[0], top[1], top[2])


def band_change(a: bytes, b: bytes) -> tuple[int, float]:
    """(pixels that changed at all, mean per-byte change) between two regions.

    The COUNT is what the floors are stated in, because it is the quantity a
    drawn mark actually has: a thin caret changes a tall narrow column of
    pixels hard, which a mean over a 950-pixel-wide band dilutes almost to
    nothing. The mean rides along as a reported margin.
    """
    if len(a) != len(b) or not a:
        return -1, float("inf")
    if a == b:
        return 0, 0.0
    changed = 0
    total = 0
    for i in range(0, len(a), 3):
        d = abs(a[i] - b[i]) + abs(a[i + 1] - b[i + 1]) + abs(a[i + 2] - b[i + 2])
        total += d
        if d:
            changed += 1
    return changed, total / len(a)


# --------------------------------------------------------------------------
# The journeys
# --------------------------------------------------------------------------


@dataclass(frozen=True)
class Journey:
    jid: str
    door: str
    shape: tuple[str, int, int, int]
    keys: str  # "{a}"/"{b}" expand to a Down run reaching the derived A/B lines
    config: str  # "empty" | "menubar"
    what: str


JOURNEYS: tuple[Journey, ...] = (
    # -- the document, at rest and at both extremes ------------------------
    Journey("doc-top", ORDINARY, WIDE, "", "empty", "the document as opened: caret home, unscrolled"),
    Journey("doc-mid-caret-a", ORDINARY, WIDE, "{a}", "empty", "caret parked mid-document, no selection"),
    Journey("doc-mid-caret-b", ORDINARY, WIDE, "{b}", "empty", "the same frame with the caret two rows down"),
    Journey("doc-mid-selected", ORDINARY, WIDE, "{a} S-Down S-Down", "empty", "the same caret, now with a multi-row selection"),
    Journey("doc-end", ORDINARY, WIDE, "s-Down", "empty", "the far end of the document, which must be a real scroll"),
    # -- the command palette, its facet strip, and its filter ---------------
    Journey("palette", ORDINARY, WIDE, "s-p", "empty", "the summoned command palette"),
    Journey("palette-selected", ORDINARY, WIDE, "s-p Down", "empty", "the same palette with the selection moved one row"),
    Journey("palette-lens", ORDINARY, WIDE, "s-p Right Down", "empty", "a facet lens stepped, and the selection moved inside it"),
    Journey("palette-query", ORDINARY, WIDE, "s-p s e t", "empty", "the palette narrowed by a typed query"),
    # -- the theme picker ---------------------------------------------------
    Journey("theme-picker", ORDINARY, WIDE, "s-t", "empty", "the whole-world audition surface"),
    # -- the drawn menu bar, over the document and under a summoned card ----
    Journey("menubar-doc", ORDINARY, WIDE, "s-Down", "menubar", "the drawn menu bar over the document"),
    Journey("menubar-palette", ORDINARY, WIDE, "s-Down s-p", "menubar", "a summoned card yielding to the drawn menu bar"),
    # -- the Settings workspace, on the door whose zoom the product ships ---
    Journey("settings-wide", LIVE_APP, WIDE, "s-,", "empty", "the Settings workspace with room for both regions"),
    Journey("settings-narrow-rail", LIVE_APP, NARROW, "s-,", "empty", "the narrow regime, staged on its navigation rail"),
    Journey("settings-narrow-detail", LIVE_APP, NARROW, "s-, Tab", "empty", "the narrow regime, staged on its content region"),
)

# The narrow Settings pair that shares a presence floor: neither stage is
# required to have rows, but between them some stage must.
STAGE_PAIR = ("settings-narrow-rail", "settings-narrow-detail")


# --------------------------------------------------------------------------
# Findings
# --------------------------------------------------------------------------


@dataclass
class Ledger:
    checks: int = 0
    failures: list[str] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)
    measures: dict[str, list[float]] = field(default_factory=dict)

    def check(self, ok: bool, where: str, message: str) -> bool:
        self.checks += 1
        if not ok:
            self.failures.append(f"{where}: {message}")
        return ok

    def measure(self, name: str, value: float) -> None:
        self.measures.setdefault(name, []).append(value)

    def note(self, text: str) -> None:
        self.notes.append(text)


# --------------------------------------------------------------------------
# Driving the binary
# --------------------------------------------------------------------------


def roster(binary: Path) -> list[str]:
    names = [w for w in subprocess.check_output([str(binary), "--list-worlds"], text=True).split() if w]
    if not names:
        raise SystemExit("error: --list-worlds printed an empty roster")
    dupes = [w for w, n in Counter(names).items() if n > 1]
    if dupes:
        raise SystemExit(f"error: --list-worlds printed duplicate names: {dupes}")
    return names


def plain_run(text: str, span: int) -> int:
    """The first line index starting `span` consecutive PLAIN prose lines.

    Plain means non-empty and free of every character the markdown layer gives
    a treatment to, so that moving the caret or drawing a selection across
    these lines changes the caret and the selection and NOTHING ELSE — no
    conceal reveal, no changed glyph advances, no heading crossing that would
    move the margin outline with it. Derived from the specimen rather than
    written down as a line number, so editing the fixture cannot silently point
    the A/B pairs at a heading.
    """
    lines = text.splitlines()
    marked = re.compile(r"[#*_`\[\]()<>|~=+\\]|^\s*[-+]\s|^\s*\d+\.\s")
    run = 0
    for i, line in enumerate(lines):
        if line.strip() and not marked.search(line):
            run += 1
            if run == span:
                return i - span + 1
        else:
            run = 0
    raise SystemExit(f"error: {SPECIMEN} has no run of {span} plain prose lines")


def capture(
    binary: Path,
    journey: Journey,
    world: str,
    dpi: int,
    out: Path,
    doc: Path,
    configs: dict[str, Path],
    keys: str,
) -> dict:
    _, lw, lh, measure = journey.shape
    mode = "--screenshot" if journey.door == ORDINARY else "--screenshot-app"
    cmd = [
        str(binary), mode, str(out),
        "--capture-size", f"{lw * dpi}x{lh * dpi}",
        "--capture-dpi", str(dpi),
        "--measure", str(measure),
        "--page", "on",
        "--theme", world,
        "--config", str(configs[journey.config]),
        "--root", str(doc.parent),
    ]
    if keys:
        cmd += ["--keys", keys]
    cmd.append(str(doc))
    done = subprocess.run(cmd, capture_output=True, text=True)
    if done.returncode != 0:
        raise SystemExit(
            f"error: capture failed for {world}/{journey.jid}@{dpi}\n"
            f"  {' '.join(cmd)}\n{done.stdout}{done.stderr}"
        )
    return json.loads(out.with_suffix(".json").read_text())


# --------------------------------------------------------------------------
# Assertions
# --------------------------------------------------------------------------


def doc_column(side: dict) -> tuple[float, float]:
    col = side["page"]["column"]
    return col["left"], col["left"] + col["width"]


def row_band(side: dict, index: int) -> tuple[float, float]:
    row = side["layout"]["rows"][index]
    return row["top"], row["top"] + row["height"]


def per_capture(led: Ledger, where: str, journey: Journey, world: str, dpi: int, side: dict, frame: Frame) -> None:
    """Everything assertable about one capture on its own."""
    canvas = side["canvas"]
    led.check(
        (frame.w, frame.h) == (canvas["width"], canvas["height"]),
        where,
        f"PNG is {frame.w}x{frame.h} but the sidecar's canvas is {canvas['width']}x{canvas['height']}",
    )
    led.check(side["theme"]["name"] == world, where, f"sidecar reports theme {side['theme']['name']!r}")
    expected_driver = "replay" if journey.door == ORDINARY else "live-app"
    led.check(side.get("driver") == expected_driver, where, f"sidecar driver is {side.get('driver')!r}, wanted {expected_driver!r}")
    led.check(not side.get("replay_skips"), where, f"the replay skipped live-App-only effects: {side.get('replay_skips')}")

    # -- geometry: the summoned card's planned rows -------------------------
    overlay = side["overlay"]
    window = overlay.get("window")
    if overlay["active"] and window:
        rows = window.get("rows") or []
        band = window.get("band")
        led.check(window["card_h"] <= window["canvas_h"] + EPS, where, f"card_h {window['card_h']} exceeds canvas_h {window['canvas_h']}")
        if band:
            led.check(band["first_top"] >= -EPS, where, f"band starts above the frame at y={band['first_top']}")
            led.check(band["footer_top"] <= window["canvas_h"] + EPS, where, f"band footer at {band['footer_top']} is past canvas_h {window['canvas_h']}")
        for i, row in enumerate(rows):
            tag = f"{where} row {i}"
            led.check(row["y"] >= -EPS and row["y"] + row["h"] <= window["canvas_h"] + EPS, tag, f"row rect {row['y']}..{row['y'] + row['h']} escapes the canvas")
            if band:
                # OVERLAP, not containment. A STAGGERING composition steps its
                # selected row OUTWARD past the band edge on purpose, so a row
                # whose span pokes out of the band by a step is a shipped
                # composition rather than a defect — `capture::tests::
                # plan_geometry` and `render::plan::tests::accessory_law` both
                # say so in as many words. What is still a defect is a row that
                # does not meet the band at all, or has no extent.
                led.check(
                    row["x"] + row["w"] > band["x"] and row["x"] < band["x"] + band["w"] and row["w"] > 1.0 and row["h"] > 1.0,
                    tag,
                    f"row span {row['x']}..{row['x'] + row['w']} ({row['w']}x{row['h']}) does not meet the band "
                    f"{band['x']}..{band['x'] + band['w']}",
                )
                led.check(row["y"] + row["h"] <= band["footer_top"] + EPS, tag, f"row bottom {row['y'] + row['h']} passes the footer at {band['footer_top']}")
            for lane in ("label", "value"):
                cell = row.get(lane)
                if cell:
                    led.check(cell["x"] >= row["x"] - EPS and cell["x"] + cell["w"] <= row["x"] + row["w"] + EPS, tag, f"the {lane} lane {cell['x']}..{cell['x'] + cell['w']} escapes its row")
            # The two lanes must not OVERLAP; which one is on the left is a
            # world's own composition and not a fact about rows. A card that
            # mirrors its lanes — name to the right, value to the left — is a
            # perfectly ordinary member of the roster, and an assertion written
            # as "label then value" would grade that world's whole card as
            # broken while catching nothing anywhere else.
            label, value = row.get("label"), row.get("value")
            if label and value:
                led.check(
                    label["x"] + label["w"] <= value["x"] + EPS or value["x"] + value["w"] <= label["x"] + EPS,
                    tag,
                    f"the label lane {label['x']}..{label['x'] + label['w']} overlaps the value lane {value['x']}..{value['x'] + value['w']}",
                )
            if i:
                prev = rows[i - 1]
                led.check(prev["y"] + prev["h"] <= row["y"] + EPS, tag, f"overlaps the row above ({prev['y'] + prev['h']} > {row['y']})")

    # -- appearance: ink against the surface's OWN ground -------------------
    region, subject = ink_region(side, frame)
    ground, fraction, worst = frame.ground_and_ink(region)
    led.measure(f"ink-fraction/{subject}", fraction)
    led.measure(f"ink-distance/{subject}", worst)
    led.check(
        fraction >= INK_FRACTION,
        where,
        f"the {subject} is blank: only {fraction * 100:.3f}% of its pixels stand at least "
        f"{INK_DISTANCE} off its own ground {ground} (furthest was {worst})",
    )


def ink_region(side: dict, frame: Frame) -> tuple[tuple[int, int, int, int], str]:
    """Where to look for ink, and what to call it.

    The region matters more than the floor. Measured over a whole frame, the
    check would be satisfied by an ambient margin shader while the writing
    column rendered empty — a floor whose subject can vanish without it
    noticing. So it is the WRITING COLUMN for a document frame and the
    CANDIDATE BAND for a summoned one: the surface the journey is about.
    """
    overlay = side["overlay"]
    window = overlay.get("window")
    if overlay["active"] and window:
        band = window.get("band")
        if band and band["footer_top"] > band["first_top"] + 1:
            return frame.rect(band["x"], band["first_top"], band["x"] + band["w"], band["footer_top"]), "candidate band"
        # A staged workspace can publish no band at all; its card is the frame.
        return frame.rect(0, window["top"], frame.w, window["top"] + window["card_h"]), "workspace card"
    left, right = doc_column(side)
    top = side["text_origin"]["top"]
    rows = side["layout"]["rows"]
    bottom = min(frame.h, rows[-1]["top"] + rows[-1]["height"]) if rows else frame.h
    return frame.rect(left, top, right, bottom), "writing column"


def stage_presence(led: Ledger, world: str, dpi: int, sides: dict[tuple[str, int], dict]) -> None:
    """The narrow Settings pair's PRESENCE FLOOR, and the only place an empty
    row window is allowed to be an answer rather than a defect."""
    counts = {}
    for jid in STAGE_PAIR:
        side = sides.get((jid, dpi))
        if side is None:
            return
        window = side["overlay"].get("window") or {}
        counts[jid] = len(window.get("rows") or [])
        if not counts[jid]:
            led.check(
                side["overlay"]["workspace"],
                f"{world}/{jid}@{dpi}",
                "published no candidate rows and is not a workspace, so nothing was staged instead",
            )
    led.check(
        any(counts.values()),
        f"{world}/{'+'.join(STAGE_PAIR)}@{dpi}",
        f"neither stage of the narrow workspace has rows ({counts}) — the rows are unreachable at this width",
    )
    led.note(f"{world}@{dpi}: narrow Settings staged {counts[STAGE_PAIR[0]]} / {counts[STAGE_PAIR[1]]} rows (a zero on ONE stage is the staging regime, not a defect)")


def dpi_parity(led: Ledger, world: str, journey: Journey, one: dict, two: dict) -> None:
    """What genuinely doubles between the same logical window at two scales.

    Asserted: the canvas, the scaled text metric, the document's own wrap (a
    row COUNT, which must not change at all), the number of planned candidate
    lines, and the candidate band's row pitch. NOT asserted: the card's own
    exterior. Its outer margin and inner pad are unscaled, so a summoned card's
    x/y/w legitimately do not double — a parity check that demanded they did
    would be asserting something untrue of the surface.
    """
    where = f"{world}/{journey.jid} dpi-parity"
    led.check(
        two["canvas"]["width"] == 2 * one["canvas"]["width"] and two["canvas"]["height"] == 2 * one["canvas"]["height"],
        where,
        f"canvas {one['canvas']} vs {two['canvas']} is not the same logical window",
    )

    def ratio(name: str, a: float, b: float) -> None:
        if not a:
            return
        r = b / a
        led.measure(f"dpi-ratio/{name}", r)
        led.check(abs(r - 2.0) <= 2.0 * DPI_TOLERANCE, where, f"{name} scales {r:.4f}x, outside 2x +/-{DPI_TOLERANCE * 100:.0f}%")

    ratio("line_height", one["font"]["line_height"], two["font"]["line_height"])
    led.check(
        len(one["layout"]["rows"]) == len(two["layout"]["rows"]),
        where,
        f"the document shaped {len(one['layout']['rows'])} visual rows at dpi 1 and {len(two['layout']['rows'])} at dpi 2 — the same logical window must wrap the same",
    )
    w1, w2 = one["overlay"].get("window"), two["overlay"].get("window")
    if one["overlay"]["active"] and w1 and w2:
        led.check(w1["lines"] == w2["lines"], where, f"the card planned {w1['lines']} display lines at dpi 1 and {w2['lines']} at dpi 2")
        led.check(len(w1.get("rows") or []) == len(w2.get("rows") or []), where, "the card published a different number of row rects at the two scales")
        b1, b2 = w1.get("band"), w2.get("band")
        if b1 and b2 and b1["pitch"]:
            ratio("band pitch", b1["pitch"], b2["pitch"])


def caret_ab(led: Ledger, world: str, dpi: int, a_side: dict, a: Frame, b_side: dict, b: Frame, rows: tuple[int, int, int]) -> None:
    """TRUE A/B for the caret: one frame, two caret positions, nothing else.

    The two captures differ by a caret two rows apart and by nothing else, so
    the changed rows must change and the control row must be untouched. Neither
    side of that is an authored number: the control row is required to change
    by exactly zero pixels, and the caret rows to change by at least their own
    shaped height's worth.
    """
    where = f"{world}/caret-A-B@{dpi}"
    ra, rb, rc = rows
    if not led.check(
        (a_side["scroll_lines"], a_side["scroll_px"]) == (b_side["scroll_lines"], b_side["scroll_px"]),
        where,
        f"the view scrolled between the two frames ({a_side['scroll_lines']}/{a_side['scroll_px']} vs {b_side['scroll_lines']}/{b_side['scroll_px']}) — the comparison would be of two different pictures",
    ):
        return
    led.check(a_side["layout"]["caret"]["row"] == ra and b_side["layout"]["caret"]["row"] == rb, where, "the caret did not land on the two rows the sweep aimed at")
    left, right = doc_column(a_side)
    changed: dict[int, tuple[int, float]] = {}
    floors: dict[int, int] = {}
    for index in (ra, rb, rc):
        top, bottom = row_band(a_side, index)
        r = a.rect(left, top, right, bottom)
        changed[index] = band_change(a.region(r), b.region(r))
        floors[index] = change_floor(bottom - top)
    led.measure("caret-band-pixels-changed", min(changed[ra][0], changed[rb][0]))
    led.measure("caret-band-mean-delta", min(changed[ra][1], changed[rb][1]))
    led.check(changed[rc][0] == 0, where, f"the control row {rc} changed on {changed[rc][0]} pixels while only the caret moved")
    led.check(
        changed[ra][0] >= floors[ra] and changed[rb][0] >= floors[rb],
        where,
        f"moving the caret off row {ra} onto row {rb} changed {changed[ra][0]}/{changed[rb][0]} pixels, under those rows' own "
        f"heights ({floors[ra]}/{floors[rb]}), while the control row changed {changed[rc][0]}",
    )


def selection_ab(led: Ledger, world: str, dpi: int, plain_side: dict, plain: Frame, sel_side: dict, sel: Frame, control: int) -> None:
    """TRUE A/B for the selection: the same row, selected and not.

    Both frames hold the caret on the same row at the same scroll; one of them
    also carries a selection reaching back two rows. So the selected rows must
    change and the control row must not.
    """
    where = f"{world}/selection-A-B@{dpi}"
    segments = sel_side["layout"].get("selection") or []
    if not led.check(len(segments) >= 2, where, f"the selection covers {len(segments)} rows; a multi-row selection is the subject"):
        return
    if not led.check(
        (plain_side["scroll_lines"], plain_side["scroll_px"]) == (sel_side["scroll_lines"], sel_side["scroll_px"])
        and plain_side["layout"]["caret"]["row"] == sel_side["layout"]["caret"]["row"],
        where,
        "the caret or the scroll moved between the selected and unselected frames",
    ):
        return
    led.check(plain_side["selection"] is None and sel_side["selection"] is not None, where, "the sidecar does not report exactly one of the two frames as selected")
    left, right = doc_column(plain_side)
    weakest: tuple[int, int, float] | None = None
    for segment in segments:
        index = segment["row"]
        top, bottom = row_band(plain_side, index)
        r = plain.rect(left, top, right, bottom)
        pixels, mean = band_change(plain.region(r), sel.region(r))
        floor = change_floor(bottom - top)
        if weakest is None or pixels - floor < weakest[0] - weakest[1]:
            weakest = (pixels, floor, mean)
    top, bottom = row_band(plain_side, control)
    rc = plain.rect(left, top, right, bottom)
    control_pixels, control_mean = band_change(plain.region(rc), sel.region(rc))
    assert weakest is not None
    led.measure("selection-band-pixels-changed", weakest[0])
    led.measure("selection-band-mean-delta", weakest[2])
    led.check(control_pixels == 0, where, f"the unselected control row changed on {control_pixels} pixels")
    led.check(
        weakest[0] >= weakest[1],
        where,
        f"the least-changed selected row moved on {weakest[0]} pixels, under its own shaped height of {weakest[1]}, "
        f"while the control row moved on {control_pixels} — the selection is not drawn on it",
    )


def picker_selection_ab(led: Ledger, world: str, dpi: int, a_side: dict, a: Frame, b_side: dict, b: Frame) -> None:
    """TRUE A/B for a picker row's selection — possible here and NOT possible
    on the theme picker, which is the whole reason both are in this sweep.

    The command palette previews nothing, so stepping its selection changes the
    selected row and the previously selected row and leaves every other row
    alone. A row far from both is the floor, and it must not move at all.
    """
    where = f"{world}/palette-selection-A-B@{dpi}"
    wa, wb = a_side["overlay"].get("window"), b_side["overlay"].get("window")
    if not (wa and wb and wa.get("rows") and wb.get("rows")):
        return
    if not led.check(wa["sel_row"] != wb["sel_row"], where, f"the selection did not move (both frames report row {wa['sel_row']})"):
        return
    if not led.check(len(wa["rows"]) == len(wb["rows"]), where, "the two frames planned different numbers of rows, so no row can be compared"):
        return
    moved = {wa["sel_row"], wb["sel_row"]}
    far = [i for i in range(len(wa["rows"])) if i not in moved]
    if not far:
        return
    control = far[-1]
    changed = []
    floors = []
    for i in sorted(moved):
        row = wa["rows"][i]
        r = a.rect(row["x"], row["y"], row["x"] + row["w"], row["y"] + row["h"])
        changed.append(band_change(a.region(r), b.region(r))[0])
        floors.append(change_floor(row["h"]))
    row = wa["rows"][control]
    rc = a.rect(row["x"], row["y"], row["x"] + row["w"], row["y"] + row["h"])
    control_pixels, _ = band_change(a.region(rc), b.region(rc))
    led.measure("palette-selection-pixels-changed", min(changed))
    led.check(control_pixels == 0, where, f"an untouched palette row {control} changed on {control_pixels} pixels")
    led.check(
        all(c >= f for c, f in zip(changed, floors)),
        where,
        f"stepping the selection changed rows {sorted(moved)} on {changed} pixels against their own heights {floors}, "
        f"while an untouched row changed on {control_pixels}",
    )


def theme_picker_weak(led: Ledger, world: str, dpi: int, side: dict, frame: Frame) -> None:
    """THE WEAKER GRADE, and the report says so.

    The theme picker cannot be A/B'd: moving its selection PREVIEWS the next
    world, so the two frames would be two different worlds and their difference
    would say nothing about selection. What is left is a WITHIN-FRAME probe: the
    selected row's TEXTLESS TAIL against its unselected neighbours' tails at the
    same x-span, with the frame supplying its own floor — how much two
    UNSELECTED tails differ from each other.

    AND ON MOST OF THIS ROSTER THAT FLOOR IS TOO HIGH TO GRADE AGAINST, which is
    a fact about the probe and not about the product. A card over a textured
    ground, or a card whose rows are staggered, makes two unselected rows differ
    from each other by as much as the selection treatment differs from either —
    measured, not assumed. So the probe grades HARD only where the frame's own
    unselected rows are pixel-interchangeable, and where they are not it ABSTAINS
    by name, records the measurement, and grades nothing. Reporting a number it
    cannot stand behind would be worse than reporting the gap.

    Even where it does grade, it is weaker than the A/B pairs above: it compares
    DIFFERENT ROWS in one frame rather than one row two ways, so it cannot rule
    out a world that treats some other row specially, and it says nothing about
    whether the row a reader would call selected is the row the picker means.
    """
    where = f"{world}/theme-picker-selection@{dpi}"
    window = side["overlay"].get("window")
    if not (window and window.get("rows")):
        return
    rows = window["rows"]
    sel = window["sel_row"]
    if not led.check(0 <= sel < len(rows), where, f"sel_row {sel} is outside the {len(rows)} published rows"):
        return
    led.check(rows[sel].get("item") is not None, where, f"sel_row {sel} points at a display line that carries no selectable item")
    # NEIGHBOURS, not rows from across the card. A world whose card sits over an
    # organic or gradient ground makes two DISTANT tails differ enormously on
    # their ground alone, which would drown any treatment; two ADJACENT tails
    # differ by one row pitch of that same ground, which is the smallest
    # ambient difference the frame can offer as a floor.
    neighbours = [i for i in (sel - 2, sel - 1, sel + 1, sel + 2) if 0 <= i < len(rows) and rows[i].get("item") is not None]
    if len(neighbours) < 3:
        led.note(f"{where}: only {len(neighbours)} unselected neighbours, too few to establish the frame's own floor — not graded")
        return
    sample = [sel] + neighbours
    span = textless_span([rows[i] for i in sample])
    if span is None:
        led.note(f"{where}: the rows leave no textless tail to compare — not graded")
        return
    x0, x1 = span
    height = min(rows[i]["h"] for i in sample)
    tails = {i: frame.region(row_rect(frame, rows[i], x0, x1, height)) for i in sample}
    floor_pixels = max(
        band_change(tails[a], tails[b])[0]
        for k, a in enumerate(neighbours)
        for b in neighbours[k + 1 :]
    )
    against = min(band_change(tails[sel], tails[i])[0] for i in neighbours)
    led.measure("theme-picker-tail-pixels-changed", against)
    led.measure("theme-picker-unselected-tail-floor", floor_pixels)
    if floor_pixels:
        led.note(
            f"{where}: NOT GRADED. Two unselected rows of this card already differ on {floor_pixels} pixels of their "
            f"textless tails (a textured ground or a staggered composition), so no within-frame difference — the "
            f"selected row's own is {against} — can be attributed to the selection. The selection's PUBLISHED row is "
            f"still asserted; its appearance is not."
        )
        return
    led.note(f"{where}: graded (this card's unselected rows are pixel-interchangeable, so the frame supplies a floor of zero)")
    led.check(
        against >= change_floor(height),
        where,
        f"the selected row's textless tail differs from unselected tails on {against} pixels, under this row's own "
        f"height of {change_floor(height)}, while two unselected tails are identical — the selection is not drawn "
        "as a treatment of the row",
    )


def row_rect(frame: Frame, row: dict, x0: float, x1: float, height: float) -> tuple[int, int, int, int]:
    """A row-anchored rect with an INTEGER height shared by every row it is
    asked for. Rounding each row's own top and bottom independently gives
    neighbouring rows rects one pixel apart in height, and two regions of
    different sizes cannot be compared at all — which reads as "no difference
    found" rather than as the arithmetic error it is."""
    top = int(round(row["y"]))
    rows_high = max(1, int(height))
    ix0 = max(0, min(frame.w - 1, int(round(x0))))
    ix1 = max(ix0 + 1, min(frame.w, int(round(x1))))
    iy0 = max(0, min(frame.h - 1, top))
    return ix0, iy0, ix1, min(frame.h, iy0 + rows_high)


def textless_span(rows: list[dict]) -> tuple[float, float] | None:
    """The widest x-span shared by these rows that NO lane of any of them draws
    text into. Mirror-agnostic by construction: it subtracts the lanes wherever
    they are rather than assuming the tail is on the right."""
    x0 = max(r["x"] for r in rows)
    x1 = min(r["x"] + r["w"] for r in rows)
    if x1 - x0 < 4:
        return None
    cuts: list[tuple[float, float]] = []
    for r in rows:
        for lane in ("label", "value", "rail"):
            cell = r.get(lane)
            if cell:
                cuts.append((cell["x"], cell["x"] + cell["w"]))
    free: list[tuple[float, float]] = [(x0, x1)]
    for cut in sorted(cuts):
        nxt: list[tuple[float, float]] = []
        for a, b in free:
            if cut[1] <= a or cut[0] >= b:
                nxt.append((a, b))
                continue
            if cut[0] > a:
                nxt.append((a, cut[0]))
            if cut[1] < b:
                nxt.append((cut[1], b))
        free = nxt
    free = [(a, b) for a, b in free if b - a >= 4]
    return max(free, key=lambda s: s[1] - s[0]) if free else None


# --------------------------------------------------------------------------
# The sweep
# --------------------------------------------------------------------------


def expectations(led: Ledger, where: str, jid: str, side: dict) -> None:
    """Did the journey actually arrive where its name claims?

    A capture that silently no-op'd its chords renders a perfectly valid frame
    of the WRONG state, and every check above would pass over it.
    """
    overlay = side["overlay"]
    if jid.startswith("doc") or jid == "menubar-doc":
        led.check(not overlay["active"], where, f"an overlay is summoned ({overlay['mode']}) on a document journey")
    if jid == "doc-top":
        led.check(side["scroll_top_px"] == 0 and side["cursor"] == {"line": 0, "col": 0}, where, "the document did not open at its top")
    if jid == "doc-end":
        led.check(side["scroll_top_px"] > 0, where, "walking to the end of the document did not scroll it — the fixture no longer outruns the window")
        led.check(side["cursor"]["line"] == side["line_count"] - 1, where, f"the caret is on line {side['cursor']['line']} of {side['line_count']}")
    if jid.startswith("palette") or jid == "menubar-palette":
        led.check(overlay["mode"] == "command", where, f"the command palette is not summoned (mode {overlay['mode']!r})")
    if jid == "palette-lens":
        active = [label for label, on in overlay["lens_strip"] if on]
        led.check(len(overlay["lens_strip"]) > 1, where, "the palette published no facet strip to step")
        led.check(active and active != [overlay["lens_strip"][0][0]], where, f"the facet lens did not step off its home ({active})")
    if jid == "palette-query":
        led.check(overlay["query"] == "set", where, f"the typed query did not reach the palette (query {overlay['query']!r})")
        led.check(bool(overlay["items"]), where, "the query matched nothing, so there is no filtered surface to grade")
    if jid == "theme-picker":
        led.check(overlay["mode"] == "theme", where, f"the theme picker is not summoned (mode {overlay['mode']!r})")
    if jid.startswith("menubar"):
        led.check(side["menubar"]["shown"], where, "the drawn menu bar is not shown despite a config that asks for it")
        led.check(bool(side["menubar"]["items"]), where, "the drawn menu bar carries no titles")
    if jid.startswith("settings"):
        led.check(overlay["mode"] == "settings", where, f"Settings is not summoned (mode {overlay['mode']!r})")
        led.check(overlay["workspace"], where, "Settings is not drawn as a workspace")
    if jid == "settings-narrow-detail":
        led.check(overlay["detail_focus"], where, "the content region did not take focus")
    if jid == "settings-wide":
        led.check(bool((overlay.get("window") or {}).get("rows")), where, "the wide Settings workspace published no rows, which the narrow regime alone may do")


@dataclass(frozen=True)
class WorldTask:
    """One world's whole sweep, self-contained so it can run in its own process.

    A world shares nothing with another world: its own captures, its own A/B
    pairs, its own findings. That independence is what makes `--jobs` safe, and
    it is also what keeps a finding readable — every message names the world it
    came from rather than a position in a merged stream.
    """

    world: str
    binary: Path
    journeys: tuple[Journey, ...]
    doc: Path
    empty_config: Path
    menubar_config: Path
    a_line: int
    b_line: int
    control_line: int


@dataclass
class WorldResult:
    led: Ledger
    captures: int
    door_zoom: dict[str, set[float]]
    ambient_menubar: bool | None


def run_world(task: WorldTask) -> WorldResult:
    world = task.world
    configs = {"empty": task.empty_config, "menubar": task.menubar_config}
    world_dir = RUN_DIR / world
    world_dir.mkdir(parents=True, exist_ok=True)
    led = Ledger()
    captures = 0
    door_zoom: dict[str, set[float]] = {}
    ambient_menubar: bool | None = None
    sides: dict[tuple[str, int], dict] = {}
    frames: dict[tuple[str, int], Frame] = {}

    for journey in task.journeys:
        keys = (
            journey.keys.replace("{a}", " ".join(["Down"] * task.a_line))
            .replace("{b}", " ".join(["Down"] * task.b_line))
            .strip()
        )
        for dpi in DPIS:
            out = world_dir / f"{journey.jid}@{dpi}.png"
            side = capture(task.binary, journey, world, dpi, out, task.doc, configs, keys)
            frame = Frame(out)
            captures += 1
            sides[(journey.jid, dpi)] = side
            frames[(journey.jid, dpi)] = frame
            where = f"{world}/{journey.jid}@{dpi}"
            door_zoom.setdefault(journey.door, set()).add(side["font"]["zoom"])
            if journey.config == "empty" and journey.door == ORDINARY and ambient_menubar is None:
                ambient_menubar = side["menubar"]["shown"]
            expectations(led, where, journey.jid, side)
            per_capture(led, where, journey, world, dpi, side, frame)

    have = {j.jid for j in task.journeys}
    for journey in task.journeys:
        one, two = sides.get((journey.jid, 1)), sides.get((journey.jid, 2))
        if one and two:
            dpi_parity(led, world, journey, one, two)
    for dpi in DPIS:
        if {"doc-mid-caret-a", "doc-mid-caret-b"} <= have:
            caret_ab(
                led, world, dpi,
                sides[("doc-mid-caret-a", dpi)], frames[("doc-mid-caret-a", dpi)],
                sides[("doc-mid-caret-b", dpi)], frames[("doc-mid-caret-b", dpi)],
                caret_rows(sides[("doc-mid-caret-a", dpi)], task, led, f"{world}@{dpi}"),
            )
        if {"doc-mid-caret-b", "doc-mid-selected"} <= have:
            rows = caret_rows(sides[("doc-mid-caret-b", dpi)], task, led, f"{world}@{dpi}")
            selection_ab(
                led, world, dpi,
                sides[("doc-mid-caret-b", dpi)], frames[("doc-mid-caret-b", dpi)],
                sides[("doc-mid-selected", dpi)], frames[("doc-mid-selected", dpi)],
                rows[2],
            )
        if {"palette", "palette-selected"} <= have:
            picker_selection_ab(
                led, world, dpi,
                sides[("palette", dpi)], frames[("palette", dpi)],
                sides[("palette-selected", dpi)], frames[("palette-selected", dpi)],
            )
        if "theme-picker" in have:
            theme_picker_weak(led, world, dpi, sides[("theme-picker", dpi)], frames[("theme-picker", dpi)])
        if set(STAGE_PAIR) <= have:
            stage_presence(led, world, dpi, sides)
    return WorldResult(led, captures, door_zoom, ambient_menubar)


def sweep(args: argparse.Namespace) -> int:
    started = time.time()
    binary = resolve_binary(args)
    worlds = roster(binary)
    if args.worlds:
        wanted = [w.strip() for w in args.worlds.split(",") if w.strip()]
        unknown = [w for w in wanted if w not in worlds]
        if unknown:
            raise SystemExit(f"error: --worlds names {unknown}, which the binary's roster does not carry: {worlds}")
        worlds = wanted
    journeys = list(JOURNEYS)
    if args.journeys:
        wanted = [j.strip() for j in args.journeys.split(",") if j.strip()]
        known = {j.jid for j in JOURNEYS}
        unknown = [j for j in wanted if j not in known]
        if unknown:
            raise SystemExit(f"error: --journeys names {unknown}; known journeys are {sorted(known)}")
        journeys = [j for j in JOURNEYS if j.jid in wanted]

    if RUN_DIR.exists():
        shutil.rmtree(RUN_DIR)
    RUN_DIR.mkdir(parents=True)

    # The captured document lives OUTSIDE the tree this sweep writes into, and
    # `--root` pins the project to its folder. A capture that lists a directory
    # must never be able to see the sweep's own output: a byte-identity check
    # once reported a difference caused by its own output landing in the folder
    # the file picker was showing.
    scratch = Path(tempfile.mkdtemp(prefix="awl-pretag-"))
    doc_dir = scratch / "room"
    doc_dir.mkdir()
    doc = doc_dir / SPECIMEN.name
    doc.write_bytes(SPECIMEN.read_bytes())
    cfg_dir = scratch / "config"
    cfg_dir.mkdir()
    configs = {"empty": cfg_dir / "empty.toml", "menubar": cfg_dir / "menubar.toml"}
    configs["empty"].write_text("")
    configs["menubar"].write_text("menu_bar = true\n")

    a_line = plain_run(SPECIMEN.read_text(), 7)
    b_line = a_line + 2
    control_line = a_line + 6

    led = Ledger()
    captures = 0
    ambient_menubar: bool | None = None
    door_zoom: dict[str, set[float]] = {}

    tasks = [
        WorldTask(world, binary, tuple(journeys), doc, configs["empty"], configs["menubar"], a_line, b_line, control_line)
        for world in worlds
    ]
    if args.jobs > 1:
        with ProcessPoolExecutor(max_workers=args.jobs) as pool:
            results = list(pool.map(run_world, tasks))
    else:
        results = [run_world(task) for task in tasks]
    for world, result in zip(worlds, results):
        led.checks += result.led.checks
        led.failures += result.led.failures
        led.notes += result.led.notes
        for name, values in result.led.measures.items():
            led.measures.setdefault(name, []).extend(values)
        captures += result.captures
        for door, zooms in result.door_zoom.items():
            door_zoom.setdefault(door, set()).update(zooms)
        if ambient_menubar is None:
            ambient_menubar = result.ambient_menubar
        print(f"    {world}: {result.captures} captures, {len(result.led.failures)} finding(s)", flush=True)

    shutil.rmtree(scratch, ignore_errors=True)
    elapsed = time.time() - started
    report = compose_report(args, binary, worlds, journeys, led, captures, elapsed, ambient_menubar, door_zoom, a_line, b_line, control_line)
    (RUN_DIR / "report.txt").write_text(report)
    (RUN_DIR / "report.json").write_text(json.dumps(
        {
            "captures": captures,
            "checks": led.checks,
            "failures": led.failures,
            "worlds": worlds,
            "journeys": [j.jid for j in journeys],
            "dpis": list(DPIS),
            "seconds": round(elapsed, 1),
            "measures": {k: {"min": min(v), "max": max(v), "n": len(v)} for k, v in sorted(led.measures.items())},
        },
        indent=2,
    ) + "\n")
    print(report)
    return 1 if led.failures else 0


def caret_rows(side: dict, task: "WorldTask", led: Ledger, where: str) -> tuple[int, int, int]:
    """Map the three fixture LINES onto visual ROWS in this frame, and refuse to
    guess. The sweep's fixture is written to fit the measure unwrapped, so a
    line that shaped into more than one row means the fixture or the measure
    moved and the A/B pairs would be comparing the wrong bands."""
    rows = side["layout"]["rows"]
    found = []
    for line in (task.a_line, task.b_line, task.control_line):
        hits = [i for i, r in enumerate(rows) if r["line"] == line]
        led.check(len(hits) == 1, where, f"fixture line {line} shaped into {len(hits)} visual rows; the A/B bands are only unambiguous while it shapes into one")
        found.append(hits[0] if hits else 0)
    return found[0], found[1], found[2]


def resolve_binary(args: argparse.Namespace) -> Path:
    if args.bin:
        binary = Path(args.bin).resolve()
        if not binary.is_file():
            raise SystemExit(f"error: --bin {binary} is not a file")
        args.profile = "supplied via --bin; this sweep did not build it and cannot name its profile"
        return binary
    profile = "debug" if args.debug else "release"
    args.profile = profile
    binary = ROOT / "target" / profile / "awl"
    print(f"==> building awl ({profile})", flush=True)
    build = ["cargo", "build"] + ([] if args.debug else ["--release"])
    if subprocess.run(build, cwd=ROOT).returncode != 0:
        raise SystemExit("error: cargo build failed")
    return binary


def compose_report(
    args: argparse.Namespace,
    binary: Path,
    worlds: list[str],
    journeys: list[Journey],
    led: Ledger,
    captures: int,
    elapsed: float,
    ambient_menubar: bool | None,
    door_zoom: dict[str, set[float]],
    a_line: int,
    b_line: int,
    control_line: int,
) -> str:
    try:
        head = subprocess.check_output(["git", "-C", str(ROOT), "rev-parse", "--short", "HEAD"], text=True).strip()
        dirty = bool(subprocess.check_output(["git", "-C", str(ROOT), "status", "--porcelain"], text=True).strip())
        tree = f"{head}{' (dirty working tree)' if dirty else ''}"
    except (OSError, subprocess.CalledProcessError):
        tree = "unknown"
    out: list[str] = []
    add = out.append
    add("=" * 78)
    add("PRE-TAG JOURNEY SWEEP")
    add("=" * 78)
    add("")
    add("THE CONFIGURATION THIS RUN ACTUALLY RAN IN")
    add("  A check that has only ever run in one configuration is an untested")
    add("  hypothesis, so here is this one, in full.")
    add(f"    binary            {binary}")
    add(f"    build profile     {args.profile} — dev frames are 10-20x slower; no timing claim is made here")
    add(f"    tree photographed {tree}")
    add(f"    host              {sys.platform}, {os.uname().machine}")
    add(f"    worlds            {len(worlds)}: {' '.join(worlds)}")
    add("    roster source     awl --list-worlds (theme::world_names() over theme::THEMES) — never a written list")
    add(f"    journeys          {len(journeys)} ({sum(1 for j in journeys if j.door == ORDINARY)} on --screenshot, {sum(1 for j in journeys if j.door == LIVE_APP)} on --screenshot-app)")
    add(f"    device scales     {' and '.join(f'--capture-dpi {d}' for d in DPIS)}, as the SAME logical window at each")
    add(f"    logical windows   wide {WIDE[1]}x{WIDE[2]} at measure {WIDE[3]}; narrow {NARROW[1]}x{NARROW[2]} at measure {NARROW[3]}")
    for door, zooms in sorted(door_zoom.items()):
        add(f"    zoom, {door:<10} {sorted(zooms)} (read off the captures, not assumed) — the doors are never differenced against each other")
    add(f"    config            empty file pinned on every capture except the menu-bar journeys, which pin menu_bar = true")
    add(f"    menu bar          ambient default under the empty config on THIS host: {ambient_menubar}; forced on for the menu-bar journeys")
    add(f"    fixture           {SPECIMEN.name}; A/B lines {a_line}/{b_line}, control line {control_line}, derived by scanning it for plain prose")
    add(f"    captures          {captures} PNG + sidecar pairs, {led.checks} checks, {elapsed:.0f}s")
    add("")
    add("JOURNEYS")
    for j in journeys:
        add(f"    {j.jid:<24} {j.door:<9} {j.shape[0]:<7} config={j.config:<8} keys={j.keys or '(none)':<26} {j.what}")
    add("")
    add("WHAT IS ASSERTED")
    add("    per capture   PNG size against the sidecar's own canvas; the sidecar names the world")
    add("                  it was asked for; the door names itself (replay / live-app); the replay")
    add("                  skipped no live-App-only effect; the journey reached the state its name")
    add("                  claims; every planned card row sits inside the canvas, MEETS its band")
    add("                  (overlap, not containment — a staggering composition steps its selected")
    add("                  row outward past the band on purpose), ends above the footer, keeps its")
    add("                  lanes inside itself with the two lanes not overlapping each other, and")
    add("                  never overlaps the row above; and the surface the journey is about")
    add("                  carries ink standing off its OWN modal ground.")
    add("    per DPI pair  the canvas doubles exactly; the scaled text metric and the candidate")
    add("                  band's pitch double within +/-2%; the document wraps to the SAME number")
    add("                  of visual rows and the card plans the SAME number of display lines.")
    add("    per A/B pair  caret: two frames two rows apart, scroll asserted equal, the two caret")
    add("                  bands must change and a control row must not change AT ALL.")
    add("                  selection: the same caret and scroll, selected versus not, the selected")
    add("                  rows must change and the control row must not.")
    add("                  palette: the selection stepped one row, the two affected rows must")
    add("                  change and an untouched row must not.")
    add("")
    graded = sum(1 for n in led.notes if "theme-picker-selection" in n and "NOT GRADED" not in n)
    abstained = sum(1 for n in led.notes if "theme-picker-selection" in n and "NOT GRADED" in n)
    add("WHAT IS GRADED MORE WEAKLY, AND WHY")
    add("    THE THEME PICKER'S SELECTION IS NOT A/B'D. It cannot be: moving the selection")
    add("    previews the next world, so the two frames are two different worlds and their")
    add("    difference says nothing about selection. The command palette, which previews")
    add("    nothing, carries the true A/B that the theme picker cannot — read the two results")
    add("    at different strengths.")
    add("")
    add("    What the theme picker gets instead is a within-frame probe: the selected row's")
    add("    textless tail against its unselected neighbours', with the frame's own")
    add("    unselected-versus-unselected difference as the floor. ON MOST OF THIS ROSTER THAT")
    add("    FLOOR IS TOO HIGH TO GRADE AGAINST — a textured ground or a staggered card makes")
    add("    two unselected rows differ from each other by as much as the selected row differs")
    add("    from either — so the probe ABSTAINS there rather than reporting a number it cannot")
    add(f"    stand behind. THIS RUN: hard-graded {graded} cell(s), abstained on {abstained}. The")
    add("    published sel_row is asserted everywhere; the APPEARANCE of the selection is")
    add("    asserted only where a cell was graded. Treat the rest as unverified BY THIS")
    add("    INSTRUMENT — which is no longer the same as unverified. That claim now has an")
    add("    owner one tier down, where the coupling this door cannot break is simply not")
    add("    present: the audition lives in the ACTION path, so a Rust render law holds the")
    add("    world FIXED while the selection moves and runs the true A/B on every world,")
    add("    textured and staggered alike, with an untouched control row that must stay")
    add("    byte-identical between the two frames. That law is")
    add("      render::tests::theme_picker_selection_law::")
    add("      the_theme_pickers_selected_row_is_drawn_as_selected_on_every_world")
    add("    and it grades all 20 worlds at both device scales, whatever this probe abstains")
    add("    on. What stays this door's own is the state claim above, on the real capture.")
    add("")
    add("    A ZERO-ROW SETTINGS STAGE AT A NARROW WIDTH IS NOT FLAGGED, because it is correct:")
    add("    the narrow regime shows one region at a time and the stage showing the other one")
    add("    publishes an empty row window faithfully. This reading has been filed as a defect")
    add("    once already and refuted (src/render/tests/workspace_stage_reach.rs). What IS")
    add("    asserted is the presence floor the pair needs: some stage must have rows.")
    add("")
    if led.measures:
        add("MEASURED MARGINS (the numbers behind the passes, so a reader can see how close they were)")
        for name, values in sorted(led.measures.items()):
            add(f"    {name:<38} min {min(values):>10.4f}   max {max(values):>10.4f}   n {len(values)}")
        add("")
    if led.notes:
        add("NOTES")
        for note in led.notes[:60]:
            add(f"    {note}")
        if len(led.notes) > 60:
            add(f"    ... and {len(led.notes) - 60} more (report.json carries the run in full)")
        add("")
    add("WHAT THIS SWEEP DID NOT COVER")
    add("  A bound that goes unstated reads as coverage, so these are the bounds.")
    add("    * ONE GPU. Every frame came off whatever adapter this host has; a capture carries")
    add("      no adapter name. Virtualised-GPU behaviour is untested by this sweep, and a")
    add("      software adapter is not a stand-in for that axis.")
    add("    * ONE BUILD. Native only. The browser build (wasm32, WebGPU and the WebGL2")
    add("      fallback) renders none of these frames and is not covered here at all.")
    add("    * NO WINDOW WAS OPENED. Nothing here reaches the event loop, the surface, or the")
    add("      compositor: no resize, no occlusion, no GPU fault, no real present. Feel over")
    add("      real time, animation, and every timing claim are outside this instrument.")
    add("    * ONE ZOOM PER DOOR. The replay door is photographed at 1.0 and the live-App door")
    add("      at its launch 0.8. The zoom BAND between and beyond them is not swept, and the")
    add("      two doors are never compared to each other.")
    add("    * TWO WINDOW SHAPES, not a continuum — one wide and one narrow. A staging")
    add("      threshold is crossed, but where it sits is not measured here.")
    add("    * A LATIN-ONLY SPECIMEN. No CJK ladder, no per-script fallback, no frontmatter")
    add("      language tag, no RTL.")
    add("    * KEYBOARD ONLY. No pointer, no drag, no click on a row this sweep proved clickable.")
    add("    * PRESENCE AND DIFFERENCE, NOT LEGIBILITY. The pixel arithmetic asks whether")
    add("      something is drawn and whether it changed. Contrast, legibility and taste are")
    add("      not graded here; contrast floors are Rust laws over theme roles.")
    add("")
    add("RESULT")
    if led.failures:
        add(f"    FAIL — {len(led.failures)} finding(s) over {led.checks} checks and {captures} captures:")
        for failure in led.failures:
            add(f"      * {failure}")
    else:
        add(f"    PASS — {led.checks} checks over {captures} captures, {len(worlds)} worlds, {len(journeys)} journeys, both device scales.")
    add("")
    add(f"    captures + sidecars   {RUN_DIR}")
    add(f"    this report           {RUN_DIR / 'report.txt'}  (and report.json)")
    add("    Not committed: /gallery is gitignored deliberately.")
    return "\n".join(out) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description="The pre-tag journey sweep across the world roster.")
    parser.add_argument("--debug", action="store_true", help="use (and build) the debug binary")
    parser.add_argument("--bin", help="use an already-built awl binary and skip the build")
    parser.add_argument("--worlds", help="comma-separated subset of the binary's roster")
    parser.add_argument("--journeys", help="comma-separated subset of the journey ids")
    parser.add_argument("--jobs", type=int, default=1, help="worlds to sweep concurrently (default 1: deterministic ordering, and polite on a shared host)")
    return sweep(parser.parse_args())


if __name__ == "__main__":
    sys.exit(main())
