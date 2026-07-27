#!/usr/bin/env python3
"""Turn LLVM's JSON into a short, changed-code missing-law reading list."""
import json
import pathlib
import subprocess
import sys


def changed_lines(root: pathlib.Path) -> dict[str, set[int]]:
    base = subprocess.run(
        ["git", "merge-base", "HEAD", "main"], cwd=root, check=True,
        text=True, capture_output=True,
    ).stdout.strip()
    diff = subprocess.run(
        ["git", "diff", "--unified=0", f"{base}...HEAD", "--", "src"],
        cwd=root, check=True, text=True, capture_output=True,
    ).stdout.splitlines()
    result: dict[str, set[int]] = {}
    path = None
    for line in diff:
        if line.startswith("+++ b/"):
            path = line[6:]
        elif line.startswith("@@") and path:
            plus = line.split("+")[1].split(" ")[0]
            start, _, count = plus[1:].partition(",")
            first = int(start)
            size = int(count or "1")
            result.setdefault(path, set()).update(range(first, first + size))
    return result


def main() -> None:
    root = pathlib.Path(sys.argv[1]).resolve()
    report = pathlib.Path(sys.argv[2]).resolve()
    output = pathlib.Path(sys.argv[3]).resolve()
    changed = changed_lines(root)
    data = json.loads(report.read_text())
    rows: list[tuple[str, int, str]] = []
    branches: list[tuple[str, int]] = []
    for item in data["data"][0]["files"]:
        name = item["filename"]
        try:
            rel = str(pathlib.Path(name).resolve().relative_to(root))
        except ValueError:
            continue
        for line, count, _ in item.get("segments", []):
            if line in changed.get(rel, set()) and count == 0:
                rows.append((rel, line, "line"))
        for branch in item.get("branches", []):
            line, _col, _end_line, _end_col, count, *_ = branch
            if line in changed.get(rel, set()) and count == 0:
                branches.append((rel, line))
    rows = sorted(set(rows))
    branches = sorted(set(branches))
    lines = [
        "# Coverage changed-code triage",
        "",
        "This is a reading list, not a target. An uncovered line or branch is a prompt to inspect the contract; only a real behavior contract earns a law.",
        "",
        "## Uncovered changed production lines",
        "",
    ]
    lines += [f"- `{path}:{line}`" for path, line, _ in rows] or ["- None."]
    lines += ["", "## Uncovered changed production branches", ""]
    lines += [f"- `{path}:{line}`" for path, line in branches] or ["- None."]
    output.write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
