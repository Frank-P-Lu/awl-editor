#!/usr/bin/env python3
"""Structural and ratcheted-Clippy halves of the blocking Rust health gate.

The default Clippy command remains independently useful and warning-free. Two
high-signal metrics are run here with warnings enabled, then matched against a
reviewed manifest. An entry is an exact diagnostic identity, so it becomes
stale if the function disappears, moves, or grows.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

import tomllib

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "scripts/code-health.toml"
LINE_LIMIT = 100
FILE_LIMIT = 500
# The baseline must be reachable from pushed main: worktree branches never push.
BASELINE = "f12d04a"
BASELINE_REASON = "item 134 initial inventory; remove debt instead of extending it"
HIGH_SIGNAL_LINTS = {"clippy::too_many_lines", "clippy::cognitive_complexity"}


def git(*args: str) -> str:
    return subprocess.check_output(
        ["git", *args], cwd=ROOT, text=True, stderr=subprocess.DEVNULL
    )


def tracked_rust() -> list[str]:
    return [path for path in git("ls-files", "*.rs").splitlines() if path]


def baseline(path: str) -> list[str]:
    try:
        return git("show", f"{BASELINE}:{path}").splitlines()
    except subprocess.CalledProcessError:
        return []


def production(path: str) -> bool:
    parts = Path(path).parts
    return parts[0] == "src" and "tests" not in parts and not path.endswith("_test.rs")


def diagnostic_key(entry: dict[str, Any]) -> tuple[str, str, int, str]:
    return (entry["lint"], entry["file"], entry["line"], entry["message"])


def load_manifest(path: Path = MANIFEST) -> tuple[set[tuple[str, str, int, str]], list[str]]:
    data = tomllib.loads(path.read_text())
    entries = data.get("clippy_exception", [])
    failures: list[str] = []
    expected: set[tuple[str, str, int, str]] = set()
    for entry in entries:
        missing = {"lint", "file", "line", "message", "reason"} - entry.keys()
        if missing:
            failures.append(f"code-health: malformed Clippy exception missing {sorted(missing)}")
            continue
        if entry["lint"] not in HIGH_SIGNAL_LINTS:
            failures.append(f"code-health: unsupported Clippy exception lint {entry['lint']}")
        if not entry["reason"].strip():
            failures.append(f"code-health: empty reason for {entry['lint']}:{entry['file']}:{entry['line']}")
        key = diagnostic_key(entry)
        if key in expected:
            failures.append(f"code-health: duplicate Clippy exception {entry['lint']}:{entry['file']}:{entry['line']}")
        expected.add(key)
    return expected, failures


def clippy_diagnostics(output: str) -> set[tuple[str, str, int, str]]:
    found: set[tuple[str, str, int, str]] = set()
    for line in output.splitlines():
        try:
            message = json.loads(line).get("message", {})
        except json.JSONDecodeError:
            continue
        lint = message.get("code", {}).get("code")
        if lint not in HIGH_SIGNAL_LINTS:
            continue
        primary = next((span for span in message["spans"] if span["is_primary"]), None)
        if primary is None:
            continue
        file_name = Path(primary["file_name"])
        try:
            file_name = file_name.relative_to(ROOT)
        except ValueError:
            pass
        found.add((lint, file_name.as_posix(), primary["line_start"], message["message"]))
    return found


def check_clippy(current: set[tuple[str, str, int, str]], expected: set[tuple[str, str, int, str]]) -> list[str]:
    failures: list[str] = []
    for lint, path, line, message in sorted(current - expected):
        failures.append(f"{path}:{line}: {lint}: {message} (new diagnostic; add no exception without review)")
    for lint, path, line, message in sorted(expected - current):
        failures.append(f"{path}:{line}: {lint}: stale exception for {message!r}; remove it")
    return failures


def run_metric_clippy() -> set[tuple[str, str, int, str]]:
    command = [
        "cargo", "clippy", "--all-targets", "--all-features", "--message-format=json", "--",
        "-W", "clippy::too_many_lines", "-W", "clippy::cognitive_complexity",
    ]
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True)
    if result.returncode:
        raise SystemExit(result.stderr or "code-health: metric Clippy failed")
    return clippy_diagnostics(result.stdout)


def structural_exceptions(
    path: Path = MANIFEST,
) -> tuple[set[tuple[str, int, str]], list[str]]:
    data = tomllib.loads(path.read_text())
    failures: list[str] = []
    allowed: set[tuple[str, int, str]] = set()
    for entry in data.get("structural_exception", []):
        missing = {"target", "kind", "line", "text", "reason"} - entry.keys()
        if missing:
            failures.append(f"code-health: malformed structural exception missing {sorted(missing)}")
            continue
        target = ROOT / entry["target"]
        if not entry["reason"].strip():
            failures.append(f"code-health: empty structural-exception reason for {entry['target']}")
        if not target.exists():
            failures.append(f"code-health: stale structural exception {entry['kind']}:{entry['target']}")
            continue
        if entry["kind"] not in {"url", "unbreakable", "rustfmt-skip", "generated", "architectural"}:
            failures.append(f"code-health: unsupported structural exception kind {entry['kind']}")
            continue
        lines = target.read_text().splitlines()
        number = entry["line"]
        if not isinstance(number, int) or number < 1 or number > len(lines) or lines[number - 1] != entry["text"]:
            failures.append(f"code-health: stale structural exception {entry['kind']}:{entry['target']}:{number}")
            continue
        allowed.add((entry["target"], number, entry["text"]))
    return allowed, failures


def check_structural(allowed: set[tuple[str, int, str]]) -> list[str]:
    failures: list[str] = []
    for path in tracked_rust():
        current = (ROOT / path).read_text().splitlines()
        old = baseline(path)
        grandfathered_lines = {line for line in old if len(line) > LINE_LIMIT}
        for number, line in enumerate(current, 1):
            if len(line) > LINE_LIMIT and line not in grandfathered_lines and (path, number, line) not in allowed:
                failures.append(f"{path}:{number}: {len(line)} columns (Rust limit is {LINE_LIMIT}; {BASELINE_REASON})")
        if production(path) and len(current) > FILE_LIMIT:
            old_size = len(old)
            if len(current) > old_size:
                failures.append(f"{path}: {len(current)} lines (production limit is {FILE_LIMIT}; baseline is {old_size}, must not grow)")
            elif old_size <= FILE_LIMIT:
                failures.append(f"{path}: {len(current)} lines (production limit is {FILE_LIMIT})")
    return failures


def self_test() -> int:
    current = {
        ("clippy::too_many_lines", "src/new.rs", 7, "this function has too many lines (101/100)"),
        ("clippy::cognitive_complexity", "src/new.rs", 30, "the function has a cognitive complexity of (26/25)"),
    }
    if len(check_clippy(current, set())) != 2:
        raise AssertionError("new high-signal diagnostics must fail")
    if len(check_clippy(set(), current)) != 2:
        raise AssertionError("missing metric diagnostics must make their exceptions stale")
    with tempfile.TemporaryDirectory() as directory:
        root = ROOT
        try:
            globals()["ROOT"] = Path(directory)
            target = Path(directory) / "src/table.rs"
            target.parent.mkdir()
            target.write_text("x" * 101 + "\n")
            manifest = Path(directory) / "health.toml"
            manifest.write_text('[[structural_exception]]\ntarget = "src/table.rs"\nkind = "unbreakable"\nline = 1\ntext = "' + "x" * 101 + '"\nreason = "intentional unbreakable generated table"\n')
            allowed, failures = structural_exceptions(manifest)
            if failures or ("src/table.rs", 1, "x" * 101) not in allowed:
                raise AssertionError("live structural exception must be accepted")
            target.write_text("short\n")
            _, failures = structural_exceptions(manifest)
            if not failures:
                raise AssertionError("stale structural exception must fail")
        finally:
            globals()["ROOT"] = root
    print("code-health: self-test clean")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    try:
        git("cat-file", "-e", f"{BASELINE}^{{commit}}")
    except subprocess.CalledProcessError as error:
        raise SystemExit(f"code-health: stale baseline {BASELINE}; refresh it deliberately") from error
    expected, failures = load_manifest()
    allowed, structural_failures = structural_exceptions()
    failures.extend(structural_failures)
    failures.extend(check_structural(allowed))
    failures.extend(check_clippy(run_metric_clippy(), expected))
    if failures:
        print("code-health: policy check failed", file=sys.stderr)
        print("\n".join(failures), file=sys.stderr)
        return 1
    print(f"code-health: structural and Clippy ratchets clean (baseline {BASELINE}; {len(expected)} Clippy exceptions)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
