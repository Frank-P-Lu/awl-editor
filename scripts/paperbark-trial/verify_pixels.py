#!/usr/bin/env python3
"""Pixel laws for the disposable Paperbark A–E real-app captures."""

from __future__ import annotations

import json
import sys
from itertools import combinations
from pathlib import Path

from PIL import Image, ImageChops


def fail(message: str) -> None:
    raise SystemExit(f"Paperbark pixel law failed: {message}")


def main() -> None:
    if len(sys.argv) != 2:
        fail("usage: verify_pixels.py OUT")
    out = Path(sys.argv[1]).resolve()
    profiles = [
        ("A", "broad-sheets"),
        ("B", "deckled-strata"),
        ("C", "loose-fibres"),
        ("D", "relief-print"),
        ("E", "peeling-curls"),
    ]
    reports: list[str] = []

    for width_class in ("wide", "narrow"):
        images: dict[str, Image.Image] = {}
        sidecars: dict[str, dict] = {}
        for profile_id, slug in profiles:
            stem = f"{profile_id.lower()}-{slug}-{width_class}"
            images[profile_id] = Image.open(out / "assets" / f"{stem}.png").convert("RGBA")
            sidecars[profile_id] = json.loads((out / "assets" / f"{stem}.json").read_text())

        widths = {image.width for image in images.values()}
        heights = {image.height for image in images.values()}
        if len(widths) != 1 or len(heights) != 1:
            fail(f"{width_class}: capture dimensions differ across profiles")
        canvas_w = next(iter(widths))
        canvas_h = next(iter(heights))
        column = sidecars["A"]["page"]["column"]
        left = int(round(column["left"]))
        right = int(round(column["left"] + column["width"]))
        if not (0 < left < right < canvas_w):
            fail(f"{width_class}: invalid page column [{left}, {right}) in {canvas_w}px canvas")

        # The background pipeline punches this exact column out. Thus the entire
        # opaque writing plane—including every glyph, caret pixel, and page-space
        # decoration—must be byte-identical across A–E.
        page_box = (left, 0, right, canvas_h)
        baseline_page = images["A"].crop(page_box)
        for profile_id, _ in profiles[1:]:
            if ImageChops.difference(baseline_page, images[profile_id].crop(page_box)).getbbox():
                fail(f"{width_class}: profile {profile_id} changed pixels inside the opaque page")

        margin_pixels = (left + canvas_w - right) * canvas_h
        if margin_pixels <= 0:
            fail(f"{width_class}: capture has no margin pixels to compare")
        pair_rates: list[float] = []
        for (left_id, _), (right_id, _) in combinations(profiles, 2):
            a = images[left_id]
            b = images[right_id]
            different = 0
            for box in ((0, 0, left, canvas_h), (right, 0, canvas_w, canvas_h)):
                diff = ImageChops.difference(a.crop(box), b.crop(box)).convert("RGB")
                # Count pixels with any changed RGB channel.
                different += sum(
                    1 for pixel in diff.get_flattened_data() if pixel != (0, 0, 0)
                )
            rate = different / margin_pixels
            pair_rates.append(rate)
            if rate < 0.02:
                fail(
                    f"{width_class}: profiles {left_id}/{right_id} differ on only "
                    f"{rate:.3%} of margin pixels"
                )
        reports.append(
            f"{width_class}: page [{left},{right}) byte-identical; "
            f"pairwise margin difference {min(pair_rates):.1%}–{max(pair_rates):.1%}"
        )

    print("Paperbark pixel laws passed: " + "; ".join(reports))


if __name__ == "__main__":
    main()
