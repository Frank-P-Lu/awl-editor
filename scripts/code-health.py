#!/usr/bin/env python3
"""Structural half of the blocking Rust health gate.

The initial debt is pinned to a reviewed tree, rather than hidden by globs. A
line already over the limit is grandfathered only while its exact text remains
unchanged; a production file already over the natural size limit may only
shrink. This gives old work a ratchet and makes every new violation actionable.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LINE_LIMIT = 100
FILE_LIMIT = 500
BASELINE = "98e1f06"
BASELINE_REASON = "item 134 initial inventory; remove debt instead of extending it"


def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True)


def tracked_rust() -> list[str]:
    return [path for path in git("ls-files", "*.rs").splitlines() if path]


def baseline(path: str) -> list[str]:
    try:
        return git("show", f"{BASELINE}:{path}").splitlines()
    except subprocess.CalledProcessError as error:
        raise SystemExit(
            f"code-health: stale baseline {BASELINE}: missing tracked {path}; "
            "refresh the one pinned baseline deliberately"
        ) from error


def production(path: str) -> bool:
    parts = Path(path).parts
    return parts[0] == "src" and "tests" not in parts and not path.endswith("_test.rs")


def main() -> int:
    failures: list[str] = []
    for path in tracked_rust():
        current = (ROOT / path).read_text().splitlines()
        old = baseline(path)
        grandfathered_lines = {line for line in old if len(line) > LINE_LIMIT}
        for number, line in enumerate(current, 1):
            width = len(line)
            if width <= LINE_LIMIT:
                continue
            if line in grandfathered_lines:
                continue
            failures.append(
                f"{path}:{number}: {width} columns (Rust limit is {LINE_LIMIT}; {BASELINE_REASON})"
            )
        if production(path) and len(current) > FILE_LIMIT:
            old_size = len(old)
            if len(current) > old_size:
                failures.append(
                    f"{path}: {len(current)} lines (production limit is {FILE_LIMIT}; "
                    f"baseline is {old_size}, must not grow)"
                )
            elif old_size <= FILE_LIMIT:
                failures.append(f"{path}: {len(current)} lines (production limit is {FILE_LIMIT})")

    if failures:
        print("code-health: structural check failed", file=sys.stderr)
        print("\n".join(failures), file=sys.stderr)
        return 1
    print(f"code-health: structural ratchets clean (baseline {BASELINE})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
