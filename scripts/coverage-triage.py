#!/usr/bin/env python3
"""Turn LLVM's JSON into a short, changed-code missing-law reading list."""
import json
import pathlib
import re
import subprocess
import sys


HUNK = re.compile(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@")


def parse_changed_lines(diff: list[str]) -> dict[str, set[int]]:
    result: dict[str, set[int]] = {}
    path = None
    for line in diff:
        if line.startswith("+++ b/"):
            path = line[6:]
        elif line.startswith("@@") and path:
            match = HUNK.match(line)
            if not match:
                raise ValueError(f"unrecognized unified-diff hunk: {line}")
            first = int(match.group(1))
            size = int(match.group(2) or "1")
            result.setdefault(path, set()).update(range(first, first + size))
    return result


def changed_lines(root: pathlib.Path) -> dict[str, set[int]]:
    base = subprocess.run(
        ["git", "merge-base", "HEAD", "main"],
        cwd=root,
        check=True,
        text=True,
        capture_output=True,
    ).stdout.strip()
    diff = subprocess.run(
        ["git", "diff", "--unified=0", f"{base}...HEAD", "--", "src"],
        cwd=root,
        check=True,
        text=True,
        capture_output=True,
    ).stdout.splitlines()
    return parse_changed_lines(diff)


def uncovered(
    data: dict, root: pathlib.Path, changed: dict[str, set[int]]
) -> tuple[list[tuple[str, int]], list[tuple[str, int]]]:
    rows: list[tuple[str, int, str]] = []
    branches: list[tuple[str, int]] = []
    for item in data["data"][0]["files"]:
        name = item["filename"]
        try:
            rel = str(pathlib.Path(name).resolve().relative_to(root))
        except ValueError:
            continue
        for segment in item.get("segments", []):
            line, count, has_count = segment[0], segment[2], segment[3]
            if has_count and line in changed.get(rel, set()) and count == 0:
                rows.append((rel, line, "line"))
        for branch in item.get("branches", []):
            line, _col, _end_line, _end_col, count, *_ = branch
            if line in changed.get(rel, set()) and count == 0:
                branches.append((rel, line))
    return sorted({(path, line) for path, line, _ in rows}), sorted(set(branches))


def render(rows: list[tuple[str, int]], branches: list[tuple[str, int]]) -> str:
    lines = [
        "# Coverage changed-code triage",
        "",
        "This is a reading list, not a target. An uncovered line or branch is a prompt to inspect the contract; only a real behavior contract earns a law.",
        "",
        "## Uncovered changed production lines",
        "",
    ]
    lines += [f"- `{path}:{line}`" for path, line in rows] or ["- None."]
    lines += ["", "## Uncovered changed production branches", ""]
    lines += [f"- `{path}:{line}`" for path, line in branches] or ["- None."]
    return "\n".join(lines) + "\n"


def self_test() -> None:
    parsed = parse_changed_lines(
        [
            "+++ b/src/dateformat.rs",
            "@@ -4 +7 @@ one-line",
            "@@ -140,0 +145,3 @@ multi-line",
        ]
    )
    assert parsed == {"src/dateformat.rs": {7, 145, 146, 147}}

    root = pathlib.Path("/repo")
    changed = {"src/dateformat.rs": {147}}

    def report(count: int) -> dict:
        return {
            "data": [
                {
                    "files": [
                        {
                            "filename": "/repo/src/dateformat.rs",
                            # LLVM also emits a zero-count, non-counted boundary
                            # marker. It must not keep a covered line in triage.
                            "segments": [
                                [147, 29, count, True, True, False],
                                [147, 32, 0, False, False, False],
                            ],
                            "branches": [],
                        }
                    ]
                }
            ]
        }

    pre = render(*uncovered(report(0), root, changed))
    post = render(*uncovered(report(2), root, changed))
    assert "`src/dateformat.rs:147`" in pre
    assert "`src/dateformat.rs:147`" not in post
    assert pre != post


def main() -> None:
    if sys.argv[1:] == ["--self-test"]:
        self_test()
        return
    root = pathlib.Path(sys.argv[1]).resolve()
    report = pathlib.Path(sys.argv[2]).resolve()
    output = pathlib.Path(sys.argv[3]).resolve()
    rows, branches = uncovered(json.loads(report.read_text()), root, changed_lines(root))
    output.write_text(render(rows, branches))


if __name__ == "__main__":
    main()
