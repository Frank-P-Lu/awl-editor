#!/usr/bin/env python3
"""Regenerate the Han-evidence bitsets `script::evidence` reads.

WHAT THIS ANSWERS: for every Han codepoint (CJK Unified Ideographs, Extension
A, and the Compatibility Ideographs — the same three ranges
`script::classify_char` calls `Script::Han`), is it encodable in the
Simplified-Chinese national charset (GB 2312) but NOT in either the
Traditional-Chinese charset (Big5) or the Japanese charset (JIS X 0208) — and
symmetrically for Big5-only. A codepoint answering yes to the first question
is unambiguous evidence the document is Simplified Chinese even with no
frontmatter tag; the second is the Traditional-Chinese mirror.

ZERO NETWORK, BY CONSTRUCTION: Unicode's own Unihan database (the
kIRG_GSource / kIRG_TSource / kIRG_JSource fields this classification is
conceptually drawn from) is not vendored in this repository and this script
never fetches it. It uses the THREE LEGACY CHARSET CODECS Python's standard
library ships with the interpreter itself (`gb2312`, `big5`, `shift_jis`) —
the same national encoding standards the Unihan source fields are keyed to —
so the table is exact, offline, and reproducible with nothing but a stock
Python 3 interpreter. `shift_jis` (not the `cp932`/`euc_jp` supersets, which
carry vendor or JIS X 0212 extensions) is used for the Japanese side so the
"neither JIS X 0208 nor Big5" half of the simplified-only rule is measured
against the plain standard, not a superset that would under-count evidence.

Run with no arguments; it overwrites the two checked-in bitsets in place:

    python3 scripts/regenerate-han-evidence.py

Each output is a bitset over the single contiguous span 0x3400..=0xFAFF (one
bit per codepoint, LSB-first within each byte, offset from 0x3400) — spanning
Extension A, the BMP CJK Unified block, and Compatibility Ideographs in one
run so `script::evidence`'s reader needs no per-range special-casing. Bits
outside `Script::Han`'s own three ranges (the two small gaps in that span)
are always zero and never read.
"""

from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT_DIR = ROOT / "assets/i18n"
SPAN_LO = 0x3400
SPAN_HI = 0xFAFF  # inclusive

SIMPLIFIED_ONLY_PATH = OUT_DIR / "han-simplified-only.bin"
TRADITIONAL_ONLY_PATH = OUT_DIR / "han-traditional-only.bin"


def encodable(ch: str, encoding: str) -> bool:
    try:
        ch.encode(encoding)
        return True
    except UnicodeEncodeError:
        return False


def build_bitset(predicate) -> bytes:
    n_bits = SPAN_HI - SPAN_LO + 1
    n_bytes = (n_bits + 7) // 8
    out = bytearray(n_bytes)
    for cp in range(SPAN_LO, SPAN_HI + 1):
        if predicate(chr(cp)):
            offset = cp - SPAN_LO
            out[offset // 8] |= 1 << (offset % 8)
    return bytes(out)


def main() -> None:
    def is_simplified_only(ch: str) -> bool:
        return (
            encodable(ch, "gb2312")
            and not encodable(ch, "big5")
            and not encodable(ch, "shift_jis")
        )

    def is_traditional_only(ch: str) -> bool:
        return (
            encodable(ch, "big5")
            and not encodable(ch, "gb2312")
            and not encodable(ch, "shift_jis")
        )

    simplified = build_bitset(is_simplified_only)
    traditional = build_bitset(is_traditional_only)

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    SIMPLIFIED_ONLY_PATH.write_bytes(simplified)
    TRADITIONAL_ONLY_PATH.write_bytes(traditional)

    s_count = sum(bin(b).count("1") for b in simplified)
    t_count = sum(bin(b).count("1") for b in traditional)
    print(f"wrote {SIMPLIFIED_ONLY_PATH} ({len(simplified)} bytes, {s_count} codepoints)")
    print(f"wrote {TRADITIONAL_ONLY_PATH} ({len(traditional)} bytes, {t_count} codepoints)")


if __name__ == "__main__":
    main()
