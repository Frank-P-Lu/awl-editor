#!/usr/bin/env python3
"""Regenerate the bundled Awl Marks subset from an offline Nishiki-teki TTF."""

from __future__ import annotations

import argparse
import copy
import hashlib
import os
from pathlib import Path
import sys
import tempfile

try:
    from fontTools import subset
    from fontTools.ttLib import TTFont
except ImportError as exc:  # pragma: no cover - a developer setup failure
    raise SystemExit(
        "fontTools is required (the script never installs or downloads it)"
    ) from exc


ROOT = Path(__file__).resolve().parent.parent
ROSTER = ROOT / "assets/fonts/AwlMarks.roster.tsv"
DEFAULT_OUTPUT = ROOT / "assets/fonts/AwlMarks.ttf"


def parse_roster(path: Path) -> tuple[dict[str, str], list[int]]:
    metadata: dict[str, str] = {}
    codepoints: list[int] = []
    saw_header = False
    for lineno, raw in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if raw.startswith("# ") and "=" in raw:
            key, value = raw[2:].split("=", 1)
            metadata[key] = value
            continue
        if not raw or raw.startswith("#"):
            continue
        if not saw_header:
            expected = "codepoint\tname\troles\tsource_range"
            if raw != expected:
                raise SystemExit(f"{path}:{lineno}: expected header {expected!r}")
            saw_header = True
            continue
        fields = raw.split("\t")
        if len(fields) != 4 or not all(fields):
            raise SystemExit(f"{path}:{lineno}: expected four non-empty TSV fields")
        token, _name, roles, source_range = fields
        if not token.startswith("U+"):
            raise SystemExit(f"{path}:{lineno}: invalid codepoint {token!r}")
        try:
            codepoint = int(token[2:], 16)
        except ValueError as exc:
            raise SystemExit(f"{path}:{lineno}: invalid codepoint {token!r}") from exc
        if codepoint > 0x10FFFF or 0xD800 <= codepoint <= 0xDFFF:
            raise SystemExit(f"{path}:{lineno}: non-scalar codepoint {token}")
        if not roles.split(",") or not source_range.strip():
            raise SystemExit(f"{path}:{lineno}: roles and source range are required")
        codepoints.append(codepoint)

    required_metadata = {"upstream_sha256", "upstream_version", "derived_family"}
    missing = sorted(required_metadata - metadata.keys())
    if missing:
        raise SystemExit(f"{path}: missing metadata: {', '.join(missing)}")
    if not saw_header or not codepoints:
        raise SystemExit(f"{path}: roster is empty")
    if codepoints != sorted(codepoints):
        raise SystemExit(f"{path}: codepoints must be sorted")
    duplicates = sorted({cp for cp in codepoints if codepoints.count(cp) > 1})
    if duplicates:
        joined = ", ".join(f"U+{cp:04X}" for cp in duplicates)
        raise SystemExit(f"{path}: duplicate codepoints: {joined}")
    return metadata, codepoints


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def decoded_names(font: TTFont, name_id: int) -> list[str]:
    return sorted({record.toUnicode() for record in font["name"].names if record.nameID == name_id})


def rename_derived_face(font: TTFont, family: str, version: str) -> None:
    names = font["name"]
    replacements = {
        1: family,
        2: "Regular",
        3: f"{family} Regular {version}",
        4: family,
        6: family.replace(" ", ""),
        16: family,
        17: "Regular",
    }
    names.names = [record for record in names.names if record.nameID not in replacements]
    for name_id, value in replacements.items():
        names.setName(value, name_id, 3, 1, 0x0409)
        names.setName(value, name_id, 1, 0, 0)

    os2 = font["OS/2"]
    os2.usWeightClass = 400
    # Regular on, bold/italic off. Leave every unrelated selection bit intact.
    os2.fsSelection = (os2.fsSelection & ~((1 << 0) | (1 << 5))) | (1 << 6)
    font["head"].macStyle &= ~0b11
    font["post"].italicAngle = 0


def cmap_codepoints(font: TTFont) -> set[int]:
    points: set[int] = set()
    for table in font["cmap"].tables:
        if table.isUnicode():
            points.update(table.cmap)
    return points


def regenerate(upstream: Path, output: Path) -> None:
    metadata, codepoints = parse_roster(ROSTER)
    actual_sha = file_sha256(upstream)
    expected_sha = metadata["upstream_sha256"]
    if actual_sha != expected_sha:
        raise SystemExit(
            f"{upstream}: sha256 mismatch\nexpected {expected_sha}\nactual   {actual_sha}"
        )

    font = TTFont(upstream, recalcTimestamp=False)
    upstream_family = decoded_names(font, 1)
    if "Nishiki-teki" not in upstream_family:
        raise SystemExit(f"{upstream}: expected Nishiki-teki family, found {upstream_family!r}")
    upstream_version = decoded_names(font, 5)
    if not any(metadata["upstream_version"] in value for value in upstream_version):
        raise SystemExit(
            f"{upstream}: expected version {metadata['upstream_version']}, found {upstream_version!r}"
        )

    protected_name_ids = (0, 5, 13, 14)
    before_metadata = {name_id: decoded_names(font, name_id) for name_id in protected_name_ids}
    protected_name_records = [
        copy.deepcopy(record)
        for record in font["name"].names
        if record.nameID in protected_name_ids
    ]
    missing = sorted(set(codepoints) - cmap_codepoints(font))
    if missing:
        joined = ", ".join(f"U+{cp:04X}" for cp in missing)
        raise SystemExit(f"{upstream}: roster codepoints missing from upstream cmap: {joined}")

    # FontTools cannot subset FontForge's timestamp table and would drop it with
    # a warning. It carries no shaping or licence data, so make that deliberate.
    if "FFTM" in font:
        del font["FFTM"]
    options = subset.Options()
    options.name_IDs = ["*"]
    options.name_languages = ["*"]
    options.recalc_timestamp = False
    subsetter = subset.Subsetter(options=options)
    subsetter.populate(unicodes=codepoints)
    subsetter.subset(font)
    font["name"].names = [
        record for record in font["name"].names if record.nameID not in protected_name_ids
    ] + protected_name_records
    rename_derived_face(font, metadata["derived_family"], metadata["upstream_version"])

    actual_cmap = cmap_codepoints(font)
    expected_cmap = set(codepoints)
    if actual_cmap != expected_cmap:
        extra = sorted(actual_cmap - expected_cmap)
        absent = sorted(expected_cmap - actual_cmap)
        raise SystemExit(
            "subset cmap drift: "
            f"extra={[f'U+{cp:04X}' for cp in extra]} "
            f"missing={[f'U+{cp:04X}' for cp in absent]}"
        )
    after_metadata = {name_id: decoded_names(font, name_id) for name_id in protected_name_ids}
    if before_metadata != after_metadata:
        raise SystemExit("subsetting changed the upstream copyright/version/OFL metadata")
    if font["OS/2"].usWeightClass != 400:
        raise SystemExit("derived face did not normalise OS/2 weight to 400")

    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(dir=output.parent, suffix=".ttf", delete=False) as temp:
        temp_path = Path(temp.name)
    try:
        font.save(temp_path, reorderTables=False)
        os.replace(temp_path, output)
    finally:
        temp_path.unlink(missing_ok=True)
    print(
        f"wrote {output} ({len(codepoints)} cmap entries, family {metadata['derived_family']!r}, "
        f"weight 400, upstream sha256 {actual_sha})"
    )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("upstream", type=Path, help="offline Nishiki-teki 4.0.5 TTF path")
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT)
    args = parser.parse_args()
    if not args.upstream.is_file():
        parser.error(f"upstream font is not a file: {args.upstream}")
    regenerate(args.upstream.resolve(), args.output.resolve())


if __name__ == "__main__":
    main()
