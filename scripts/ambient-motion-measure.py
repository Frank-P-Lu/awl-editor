#!/usr/bin/env python3
"""ambient-motion-measure.py — how far each ambient ground changes per real
second, measured off `scripts/capture-ambient-motion.sh`'s phase series.

Ambient motion contributes to idle visual energy, and the honest question
about ambient motion is not "does it move" (all five do, by construction) but
"how much of the ground has visibly changed by the time the writer glances up".
That is what this measures, against the t=0 frame:

  chg>1   fraction of right-margin pixels whose CIE L* differs from the t=0
          frame by more than 1.0 — roughly one just-noticeable step. "How much
          of the ground is somewhere else now."
  chg>3   the same at 3.0 L*, an unambiguous change rather than a shimmer.
  mean|d| mean absolute L* change against t=0 over the whole right margin.
  p99|d|  the 99th percentile of that change — the loudest single place.

Only the right margin is sampled, for the reason ground-contrast-measure.py gives:
the left margin carries the Outline rail and the gutter, which are ink, not
ground.

WHAT THIS IS NOT: a frame-rate or a feel measurement. These are deterministic
renders at explicit phases; they prove the trajectory, not the cadence and not
the calmness. `--release` live observation settles those, and
this script exists to make that live sitting cheap and specific, not to
replace it.
"""

import importlib.util
import os
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
_ROOT = os.path.dirname(_HERE)

# A by-path load writes scripts/__pycache__ next to the LOADED file, so the
# guard belongs here rather than in whatever invokes this script: the
# consumers are hand-run instruments and Rust tests, not one wrapper that
# could carry PYTHONDONTWRITEBYTECODE for all of them.
sys.dont_write_bytecode = True
_spec = importlib.util.spec_from_file_location(
    "awl_ground_contrast_measure", os.path.join(_HERE, "ground-contrast-measure.py")
)
_lm = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_lm)

RUN_DIR = os.path.join(_ROOT, "gallery", "ambient-motion")


def margin_lstars(png, sidecar):
    import json

    w, h, bpp, rows = _lm.decode_png(png)
    col = json.load(open(sidecar))["page"]["column"]
    x0 = int(col["left"]) + int(col["width"]) + 8
    x1 = w - 8
    px = _lm.band_pixels(rows, bpp, x0, x1, 8, h - 8, ystep=3)
    return [_lm.lstar(p) for p in px]


def main(argv):
    if not os.path.isdir(RUN_DIR):
        raise SystemExit(f"missing {RUN_DIR} — run scripts/capture-ambient-motion.sh")
    print("world\tt_s\tchg>1\tchg>3\tmean|d|\tp99|d|")
    for world in sorted(os.listdir(RUN_DIR)):
        d = os.path.join(RUN_DIR, world)
        if not os.path.isdir(d):
            continue
        base = margin_lstars(os.path.join(d, "t0.png"), os.path.join(d, "t0.json"))
        secs = sorted(
            int(f[1:-4]) for f in os.listdir(d) if f.startswith("t") and f.endswith(".png")
        )
        for s in secs:
            cur = margin_lstars(
                os.path.join(d, f"t{s}.png"), os.path.join(d, f"t{s}.json")
            )
            deltas = [abs(a - b) for a, b in zip(cur, base)]
            n = len(deltas)
            c1 = sum(1 for v in deltas if v > 1.0) / n
            c3 = sum(1 for v in deltas if v > 3.0) / n
            mean = sum(deltas) / n
            p99 = _lm.percentile(sorted(deltas), 0.99)
            print(f"{world}\t{s}\t{c1:.3f}\t{c3:.3f}\t{mean:.2f}\t{p99:.2f}")


if __name__ == "__main__":
    main(sys.argv)
