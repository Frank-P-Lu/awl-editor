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
import platform
import re
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
TARGET_OS = {"Darwin": "macos", "Linux": "linux"}.get(platform.system(), platform.system().lower())

# Only whole-line Rust comments are governed here. Inline comments are a
# separate syntax and must not silently enter this metric.
COMMENT_LINE = re.compile(r"^\s*(?:///|//!|//)")
# No trailing word boundary: indexed suffixes such as `116a` remain citations.
CITATION_KEYWORD = re.compile(r"\b(?:item|round)\s+\d+", re.IGNORECASE)
# Backticks distinguish commit-like tokens from ordinary hexadecimal values;
# requiring an a-f digit excludes long decimal examples.
CITATION_SHA = re.compile(r"`([0-9a-f]{7,40})`")
# Index-named test modules may use their filename as a durable test-family key.
TEST_FILENAME_ITEM_INDEX = re.compile(r"_item\d+[a-z]?\.rs$", re.IGNORECASE)
# Capture schema rows are a live append-only protocol ledger.
CAPTURE_SCHEMA_ROW = re.compile(r"^\s*///\s*`/\d+`")


def is_comment_citation_line(line: str) -> bool:
    """A whole-line Rust comment that cites a queue item, round, or sha."""
    if not COMMENT_LINE.match(line):
        return False
    if CITATION_KEYWORD.search(line):
        return True
    sha_match = CITATION_SHA.search(line)
    return bool(sha_match and any(c in "abcdef" for c in sha_match.group(1)))


def is_index_named_test_file(path: str) -> bool:
    return "/tests/" in path and bool(TEST_FILENAME_ITEM_INDEX.search(Path(path).name))


def is_index_named_test_citation(path: str, text: str) -> bool:
    if not is_index_named_test_file(path) or CITATION_SHA.search(text):
        return False
    filename_match = re.search(r"_item(\d+)[a-z]?\.rs$", Path(path).name, re.IGNORECASE)
    cited = re.findall(r"\bitem\s+(\d+)", text, re.IGNORECASE)
    return bool(filename_match and cited) and all(
        number == filename_match.group(1) for number in cited
    )


def is_capture_schema_history_row(path: str, text: str) -> bool:
    return path == "src/capture.rs" and bool(CAPTURE_SCHEMA_ROW.match(text))


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
    # Test code is exempt from the production ceilings however it is spelled:
    # a `tests/` directory, a sibling `tests.rs`, or a `*_test.rs`. Counting
    # `tests.rs` as production would also defeat the point of carving an inline
    # `mod tests` out of an oversized module — the lines would simply move from
    # one measured file to another.
    parts = Path(path).parts
    name = Path(path).name
    return (
        parts[0] == "src"
        and "tests" not in parts
        and name != "tests.rs"
        and not path.endswith("_test.rs")
    )


def diagnostic_key(entry: dict[str, Any]) -> tuple[str, str, int, str]:
    return (entry["lint"], entry["file"], entry["line"], entry["message"])


def load_manifest(
    path: Path = MANIFEST, target_os: str = TARGET_OS
) -> tuple[set[tuple[str, str, int, str]], list[str]]:
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
        entry_target = entry.get("target_os")
        if entry_target is not None and entry_target not in {"linux", "macos"}:
            failures.append(
                f"code-health: unsupported Clippy exception target_os {entry_target!r}"
            )
        if not entry["reason"].strip():
            failures.append(f"code-health: empty reason for {entry['lint']}:{entry['file']}:{entry['line']}")
        if entry_target is not None and entry_target != target_os:
            continue
        key = diagnostic_key(entry)
        if key in expected:
            failures.append(f"code-health: duplicate Clippy exception {entry['lint']}:{entry['file']}:{entry['line']}")
        expected.add(key)
    return expected, failures


def load_file_size_marks(
    path: Path = MANIFEST,
) -> tuple[dict[str, int], dict[str, str], list[str]]:
    data = tomllib.loads(path.read_text())
    failures: list[str] = []
    marks: dict[str, int] = {}
    reasons: dict[str, str] = {}
    for entry in data.get("file_size_mark", []):
        missing = {"file", "lines"} - entry.keys()
        if missing:
            failures.append(f"code-health: malformed file-size mark missing {sorted(missing)}")
            continue
        file = entry["file"]
        lines = entry["lines"]
        if not isinstance(file, str) or not file.endswith(".rs"):
            failures.append(f"code-health: invalid file-size mark path {file!r}")
            continue
        if not isinstance(lines, int) or isinstance(lines, bool) or lines < 0:
            failures.append(f"code-health: invalid file-size mark for {file}")
            continue
        if file in marks:
            failures.append(f"code-health: duplicate file-size mark for {file}")
            continue
        reason = entry.get("reason")
        if reason is not None:
            if not isinstance(reason, str) or not reason.strip():
                failures.append(f"code-health: empty file-size-mark reason for {file}")
            else:
                reasons[file] = reason
        marks[file] = lines
    return marks, reasons, failures


def resolve_merge_base(head_ref: str = "HEAD") -> tuple[str | None, str]:
    """The commit this branch actually forked from — `git merge-base
    head_ref main` (or `origin/main`) — and a status naming whether that
    comparison means anything. Every branch-delta check shares this resolver
    so head-is-main and unresolvable semantics stay in one place.

    Status is one of:
      "ok"           — `head_ref` differs from the resolved `main`/
                        `origin/main` commit and a merge base was found, so
                        the base sha is the branch's real fork point and a
                        diff against it is meaningful.
      "unresolvable" — `main`/`origin/main` didn't resolve, `head_ref` itself
                        doesn't resolve, or the two share no merge base (a
                        shallow or detached checkout, or unrelated
                        histories). Skips the audit rather than failing every
                        worker's gate — the same tolerance code-health.sh
                        already grants its own Linux-target Clippy arm when
                        that target isn't installed.
      "head_is_main" — `head_ref` already IS the resolved commit, so there is
                        no prior branch state to diff against. This is not a
                        clean audit, it is a vacuous one, and it is the
                        *normal* state in two of the three places this
                        script runs on this push-straight-to-`main` repo:
                        CI's `code-health.sh` job (always HEAD-on-`main`) and
                        the merge train's post-merge candidate. Comparisons
                        built on this function have real force in exactly
                        one place — a worker's worktree branch, checked
                        before landing.

    `head_ref` defaults to `HEAD` and is only overridden by a caller proving
    the `head_is_main` behavior without actually checking out `main`.
    """
    ref_sha = None
    for ref in ("main", "origin/main"):
        try:
            ref_sha = git("rev-parse", ref).strip()
            break
        except subprocess.CalledProcessError:
            continue
    if ref_sha is None:
        return None, "unresolvable"
    try:
        head_sha = git("rev-parse", head_ref).strip()
    except subprocess.CalledProcessError:
        return None, "unresolvable"
    if ref_sha == head_sha:
        return None, "head_is_main"
    try:
        base_sha = git("merge-base", head_sha, ref_sha).strip()
    except subprocess.CalledProcessError:
        return None, "unresolvable"
    return base_sha, "ok"


def previous_marks(
    manifest_path: str = "scripts/code-health.toml", head_ref: str = "HEAD"
) -> tuple[dict[str, int] | None, str]:
    """The file-size-mark table at the commit this branch actually forked
    from — `resolve_merge_base(head_ref)` — and a status naming whether that
    comparison means anything.

    Deliberately not `main`'s tip: this repo actively encourages paying a
    code-health bill by *lowering* a mark on `main` (extraction), and the tip
    moves the instant that lands. A worker whose branch forked before that
    lowering, and never touched the file itself, would then be compared
    against a baseline it never saw and never raised. The merge base is the one point both histories agree the
    branch actually started from, so a raise measured against it is a raise
    the branch itself made.

    Also deliberately not the frozen BASELINE commit: marks did not exist
    that far back (`git show BASELINE:scripts/code-health.toml` has no such
    path), and BASELINE's own file sizes are the separate, much larger
    ceiling `check_structural` enforces directly against history.

    Status comes straight from `resolve_merge_base` — see its docstring for
    what "ok"/"unresolvable"/"head_is_main" mean here. The raise audit this
    feeds has real force in exactly one place — a worker's worktree branch,
    checked before landing.

    `head_ref` defaults to `HEAD` and is only overridden by a caller proving
    the `head_is_main` behavior without actually checking out `main`.
    """
    base_sha, status = resolve_merge_base(head_ref)
    if status != "ok":
        return None, status
    text = git("show", f"{base_sha}:{manifest_path}")
    data = tomllib.loads(text)
    marks = {
        entry["file"]: entry["lines"]
        for entry in data.get("file_size_mark", [])
        if isinstance(entry.get("file"), str) and isinstance(entry.get("lines"), int)
    }
    return marks, "ok"


def check_mark_raises(
    marks: dict[str, int], reasons: dict[str, str], previous: dict[str, int] | None
) -> list[str]:
    """A mark may rise only with a reason recorded for that raise.

    Never exceeding the frozen baseline is `check_structural`'s job, checked
    directly against that fixed commit. This is the other half of the real
    invariant: a raise below that baseline is only legitimate when growth is
    unavoidable at a single-owner seam, and that judgment must be recorded,
    not silent. `previous` is `None` for either non-"ok" status `previous_marks`
    can return; neither an unresolvable reference nor a vacuous HEAD-is-main
    comparison can prove a raise happened, so neither is treated as one.
    """
    if previous is None:
        return []
    failures: list[str] = []
    for file, lines in sorted(marks.items()):
        prior = previous.get(file)
        if prior is not None and lines > prior and not reasons.get(file, "").strip():
            failures.append(
                f"{file}: file-size mark raised from {prior} to {lines} lines with no "
                "recorded reason (a raise is legitimate only below the frozen baseline, "
                "for growth unavoidable at a single-owner seam, named by a `reason`)"
            )
    return failures


def citation_exceptions(path: Path = MANIFEST) -> tuple[set[tuple[str, int, str]], list[str]]:
    """Named per-line exceptions for a legitimate new comment citation,
    alongside index-named test files and capture schema rows:
    a measured threshold or product constant whose provenance IS the cited
    fact (e.g. a perf number pinned to the commit that measured it). Same
    staleness shape as `structural_exceptions`: `text` must still match the
    file's current line, so an entry silently expires the moment the cited
    line moves rather than uselessly guarding dead ground. Kept as its own
    table rather than folded into `structural_exception` — that table's
    `kind` vocabulary is about the 100-column line rule, a different
    invariant, and conflating the two would let one entry silently excuse
    both without either lane meaning to grant the second.
    """
    data = tomllib.loads(path.read_text())
    failures: list[str] = []
    allowed: set[tuple[str, int, str]] = set()
    for entry in data.get("comment_citation_exception", []):
        missing = {"target", "line", "text", "reason"} - entry.keys()
        if missing:
            failures.append(f"code-health: malformed comment-citation exception missing {sorted(missing)}")
            continue
        if not entry["reason"].strip():
            failures.append(f"code-health: empty comment-citation-exception reason for {entry['target']}")
        target = ROOT / entry["target"]
        if not target.exists():
            failures.append(f"code-health: stale comment-citation exception {entry['target']}:{entry['line']}")
            continue
        lines = target.read_text().splitlines()
        number = entry["line"]
        if not isinstance(number, int) or number < 1 or number > len(lines) or lines[number - 1] != entry["text"]:
            failures.append(f"code-health: stale comment-citation exception {entry['target']}:{number}")
            continue
        allowed.add((entry["target"], number, entry["text"]))
    return allowed, failures


def new_comment_citations(head_ref: str = "HEAD") -> tuple[list[tuple[str, int, str]] | None, str]:
    """Comment-citation lines (`is_comment_citation_line`) present now under
    `src/` that were NOT present at `resolve_merge_base(head_ref)` — i.e.
    genuinely added by this branch. Comparison uses the fork point rather
    than `main`'s moving tip, so a
    branch that merely touches a file an unrelated lane already salted with
    archaeology is not blamed for lines it never wrote.

    Diffed against the WORKING TREE (`git diff <base> -- src`, no second
    ref), matching `check_structural`'s live-disk read elsewhere in this
    file: a citation added but not yet committed is still worth catching
    before the commit that would need `--self-test`/`code-health.sh` re-run
    to see it. `-M` requests rename detection so a pure file move does not
    read as a wholesale deletion+re-addition of every citation inside it.

    Returns `(None, status)` for the same non-"ok" statuses
    `resolve_merge_base` can report — an unresolvable reference or a vacuous
    HEAD-is-`main` comparison can prove nothing was added, so neither is
    treated as if it had been.
    """
    base_sha, status = resolve_merge_base(head_ref)
    if status != "ok":
        return None, status
    diff_text = git("diff", "-M", "--unified=0", base_sha, "--", "src")
    added: list[tuple[str, int, str]] = []
    current_file: str | None = None
    current_line: int | None = None
    for line in diff_text.splitlines():
        if line.startswith("+++ "):
            raw = line[4:]
            if raw == "/dev/null":
                current_file = None
            elif raw.startswith(("a/", "b/")):
                current_file = raw[2:]
            else:
                current_file = raw
            current_line = None
            continue
        if line.startswith("@@"):
            match = re.search(r"\+(\d+)", line)
            current_line = int(match.group(1)) if match else None
            continue
        if line.startswith("+++") or line.startswith("---"):
            continue
        if line.startswith("+"):
            text = line[1:]
            if current_file is not None and current_file.endswith(".rs") and current_line is not None:
                if is_comment_citation_line(text):
                    added.append((current_file, current_line, text))
                current_line += 1
            continue
        # "-" (removed) and other diff metadata lines don't advance the new
        # file's line counter.
    return added, "ok"


def check_comment_citations(
    added: list[tuple[str, int, str]] | None,
    exceptions: set[tuple[str, int, str]],
) -> list[str]:
    """A newly added citation fails unless it is a named exception: an
    index-named test file, `capture.rs`'s schema-history row, or a
    `comment_citation_exception` entry recording why. `added` is `None` for
    either non-"ok" status `new_comment_citations` can return.
    """
    if added is None:
        return []
    failures: list[str] = []
    for file, line, text in added:
        if is_index_named_test_citation(file, text):
            continue
        if is_capture_schema_history_row(file, text):
            continue
        if (file, line, text) in exceptions:
            continue
        failures.append(
            f"{file}:{line}: new comment cites queue-item/round/sha archaeology "
            f"({text.strip()!r}); CLAUDE.md's Conventions rule: comments state what "
            "the code can't say about itself, not history — remove the citation, or "
            "if it is a genuine exception (an index-named test file, capture.rs's "
            "schema ledger, or a measured threshold whose provenance is the fact "
            "itself) record it as a `comment_citation_exception` with a `reason`"
        )
    return failures


def comment_citation_backlog(paths: list[str] | None = None) -> dict[str, int]:
    """Existing comment-citation lines under `src/`, counted per top-level
    module. The ratchet stops growth; draining the backlog is separate work.
    """
    if paths is None:
        paths = tracked_rust()
    counts: dict[str, int] = {}
    for path in paths:
        if not path.startswith("src/") or not path.endswith(".rs"):
            continue
        try:
            text = (ROOT / path).read_text()
        except OSError:
            continue
        n = sum(1 for line in text.splitlines() if is_comment_citation_line(line))
        if n == 0:
            continue
        parts = Path(path).parts
        module = parts[1] if len(parts) > 2 else parts[-1]
        counts[module] = counts.get(module, 0) + n
    return counts


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


def check_structural(
    allowed: set[tuple[str, int, str]], file_size_marks: dict[str, int]
) -> list[str]:
    failures: list[str] = []
    tracked = set(tracked_rust())
    for path in sorted(tracked):
        current_path = ROOT / path
        if not current_path.is_file():
            failures.append(f"{path}: tracked Rust file is absent")
            continue
        current = current_path.read_text().splitlines()
        old = baseline(path)
        grandfathered_lines = {line for line in old if len(line) > LINE_LIMIT}
        for number, line in enumerate(current, 1):
            if len(line) > LINE_LIMIT and line not in grandfathered_lines and (path, number, line) not in allowed:
                failures.append(f"{path}:{number}: {len(line)} columns (Rust limit is {LINE_LIMIT}; {BASELINE_REASON})")
        if not production(path):
            continue
        old_size = len(old)
        mark = file_size_marks.get(path)
        if old_size > FILE_LIMIT:
            if mark is None:
                failures.append(
                    f"{path}: missing file-size mark for {old_size}-line grandfathered production file"
                )
                continue
            if mark > len(current):
                failures.append(
                    f"{path}: stored file-size mark is {mark} lines but current file is {len(current)}; marks may only decrease"
                )
            if len(current) > min(old_size, mark):
                failures.append(
                    f"{path}: {len(current)} lines (production limit is {FILE_LIMIT}; "
                    f"high-water mark is {min(old_size, mark)}, must not grow)"
                )
        elif len(current) > FILE_LIMIT:
            failures.append(f"{path}: {len(current)} lines (production limit is {FILE_LIMIT})")
    for path in sorted(file_size_marks.keys() - tracked):
        failures.append(f"{path}: stale file-size mark for untracked Rust file")
    return failures


def native_gate_audit(script: str, ci: str) -> list[str]:
    """Pin the one full-native definition outside Cargo's selectable targets.

    This intentionally lives in the Python health gate, not a Rust test: a
    `cargo test --bin awl` invocation cannot exclude the auditor that would
    otherwise catch its own scope loss.
    """
    failures: list[str] = []
    # Every shape check below reads the script with its comment lines removed.
    # Substring matching against the raw text passes on a requirement that has
    # merely been commented out — which is exactly how a bound gets disabled,
    # and it left this auditor green over a disabled thread bound once.
    script = "\n".join(
        line for line in script.splitlines() if not line.lstrip().startswith("#")
    )
    # The workflow gets the same treatment, and for the same reason. This audit
    # first shipped without it and was VACUOUS on its own first mutation: the
    # mac job's `AWL_NATIVE_GATE_BUDGET_SECONDS: 1500` could be deleted outright
    # and the requirement still passed, satisfied by a neighbouring comment that
    # merely mentioned the variable by name.
    ci = "\n".join(
        line for line in ci.splitlines() if not line.lstrip().startswith("#")
    )
    required_script_lines = {
        'canary_command=(cargo test --test native_gate_canary)':
            "native-gate-audit: missing named integration-only canary",
        'mac_command=(env AWL_CONVENTION_FORCE=mac cargo test)':
            "native-gate-audit: native suite command for mac must be unfiltered",
        'linux_command=(env AWL_CONVENTION_FORCE=linux cargo test)':
            "native-gate-audit: missing Linux full-suite convention",
        'export RUST_TEST_THREADS':
            "native-gate-audit: gate must bound per-convention test-thread concurrency",
        'native-gate-env cpus=':
            "native-gate-audit: gate must state the machine and bound it is about to load",
        # Flat memory and zero swap describe a deadlock and a livelock equally
        # well. CPU is the discriminator, and the load average alone is only
        # half of it: it says the box is busy, never which process is busy.
        'load1=%s cpu_count=%s':
            "native-gate-audit: the heartbeat must report the system load beside the core count that makes it readable",
        'ps -A -o pid=,pgid=,time=,comm=':
            "native-gate-audit: the heartbeat must sample cumulative per-process CPU time, which means the same thing on macOS and Linux; ps pcpu does not",
        '"$(gate_cpu_report)"':
            "native-gate-audit: the heartbeat must carry per-tracked-process CPU, not only the machine's load average",
        'ps -A -o pid=,ppid=,pgid=,etime=,time=,rss=,stat=,comm=':
            "native-gate-audit: the abort's process dump must carry CPU time beside elapsed time, or it cannot say whether the hung process was spinning",
        'if [[ -f "$gate_budget_marker" ]]; then':
            "native-gate-audit: an exhausted budget must suppress the receipt",
        'AWL_NATIVE_GATE_DEADLINE_EPOCH - gate_started_epoch':
            "native-gate-audit: the budget must honour an absolute caller deadline, not only its own duration",
        'kill "-$signal" "-$pgid"':
            "native-gate-audit: an exhausted budget must end whole process groups, not lone pids",
        '"$@" 2>&1 | gate_stamp_phases "$label"':
            "native-gate-audit: every convention's output must pass through the gate's own filter",
        "printf 'native-gate-phase label=%s event=%s elapsed_seconds=%s %s\\n'":
            "native-gate-audit: the gate must stamp its phase boundaries with elapsed time",
        'start_commit="$(git rev-parse HEAD)"':
            "native-gate-audit: receipt must capture HEAD before the suites",
        'end_commit="$(git rev-parse HEAD)"':
            "native-gate-audit: receipt must resolve HEAD after both suites",
        'wait "$mac_pid"':
            "native-gate-audit: gate must await the mac convention",
        'wait "$linux_pid"':
            "native-gate-audit: gate must await the Linux convention",
        'if (( mac_status != 0 || linux_status != 0 )); then':
            "native-gate-audit: either convention failure must suppress the receipt",
        "printf 'native-gate-receipt commit=%s conventions=mac,linux scope=all-targets\\n' \"$end_commit\"":
            "native-gate-audit: receipt must name the exact commit, both conventions, and all-target scope",
    }
    for required, failure in required_script_lines.items():
        if required not in script:
            failures.append(failure)

    canary = script.find('"${canary_command[@]}"')
    mac = script.find('"${mac_command[@]}"')
    linux = script.find('"${linux_command[@]}"')
    receipt = script.find("native-gate-receipt")
    mac_wait = script.find('wait "$mac_pid"')
    linux_wait = script.find('wait "$linux_pid"')
    if min(canary, mac, linux, mac_wait, linux_wait, receipt) < 0 or not (
        canary < mac < linux < mac_wait < linux_wait < receipt
    ):
        failures.append(
            "native-gate-audit: canary must precede both suites, both waits, and the receipt"
        )
    # A bound applied after the suites have launched binds nothing, and a
    # machine receipt printed after a starved runner has died is never read.
    bound = script.find("export RUST_TEST_THREADS")
    machine = script.find("native-gate-env cpus=")
    if bound < 0 or machine < 0 or not (bound < mac and machine < mac):
        failures.append(
            "native-gate-audit: the thread bound and the machine receipt must precede both suites"
        )
    if "if (( $# != 0 )); then" not in script:
        failures.append("native-gate-audit: gate must reject target-selection and test-name arguments")
    # A CPU reading is a DELTA between two samples, so the first heartbeat is
    # blind unless a baseline was taken before it. Sixty seconds of blindness
    # covers the whole canary phase on a warm runner.
    cpu_baseline = script.find('gate_cpu_sample >"$gate_cpu_prev"')
    cpu_report = script.find('"$(gate_cpu_report)"')
    if cpu_baseline < 0 or cpu_report < 0 or not cpu_baseline < cpu_report:
        failures.append(
            "native-gate-audit: the CPU baseline must be sampled before the first heartbeat, or that heartbeat reports no delta"
        )
    # A watchdog armed after the canary has returned cannot reach the phase that
    # compiles every dependency — the slowest thing on a cold hosted runner, and
    # the phase whose silence has no log at all.
    armed = script.find("gate_sleep_then \"$gate_budget_seconds\" gate_budget_expired")
    if armed < 0 or canary < 0 or not armed < canary:
        failures.append(
            "native-gate-audit: the budget must be armed before the canary, so it covers every phase"
        )

    for job in ("linux", "mac"):
        marker = f"  {job}:\n"
        start = ci.find(marker)
        if start < 0:
            failures.append(f"native-gate-audit: CI lacks the {job} native job")
            continue
        next_job = re.search(r"\n  [A-Za-z][^:\n]*:", ci[start + len(marker):])
        end = start + len(marker) + next_job.start() if next_job else len(ci)
        body = ci[start:end]
        if "run: scripts/native-gate.sh" not in body:
            failures.append(f"native-gate-audit: CI {job} job must call scripts/native-gate.sh")
        # Only the mac runner has the death this defends against: four losses at
        # job-minute 53-62, upstream actions/runner-images#13882, none of which
        # published a log. Both bounds must be present, because each is blind
        # to what the other sees — the duration cannot see earlier steps eating
        # the runner's clock, and the deadline cannot see a gate that starts
        # late but hangs early.
        if job == "mac":
            for setting, failure in (
                ("AWL_NATIVE_GATE_BUDGET_SECONDS",
                 "native-gate-audit: CI mac job must bound the gate's own duration"),
                ("AWL_NATIVE_GATE_DEADLINE_EPOCH",
                 "native-gate-audit: CI mac job must anchor the gate to the runner's clock"),
            ):
                if setting not in body:
                    failures.append(failure)
    return failures


def self_test() -> int:
    main = (ROOT / "src/main.rs").read_text()
    mas_gate = '#[cfg(all(feature = "mas", target_os = "macos"))]\nmod mas;'
    if mas_gate not in main:
        raise AssertionError(
            "MAS must be gated to macOS so Linux --all-features does not compile dead platform code"
        )
    current = {
        ("clippy::too_many_lines", "src/new.rs", 7, "this function has too many lines (101/100)"),
        ("clippy::cognitive_complexity", "src/new.rs", 30, "the function has a cognitive complexity of (26/25)"),
    }
    if len(check_clippy(current, set())) != 2:
        raise AssertionError("new high-signal diagnostics must fail")
    if len(check_clippy(set(), current)) != 2:
        raise AssertionError("missing metric diagnostics must make their exceptions stale")
    with tempfile.TemporaryDirectory() as directory:
        manifest = Path(directory) / "platform-health.toml"
        manifest.write_text(
            '[[clippy_exception]]\n'
            'lint = "clippy::cognitive_complexity"\n'
            'file = "src/apply.rs"\n'
            'line = 1\n'
            'message = "linux metric"\n'
            'target_os = "linux"\n'
            'reason = "platform-gated branch"\n\n'
            '[[clippy_exception]]\n'
            'lint = "clippy::cognitive_complexity"\n'
            'file = "src/apply.rs"\n'
            'line = 1\n'
            'message = "macOS metric"\n'
            'target_os = "macos"\n'
            'reason = "platform-gated branch"\n'
        )
        linux, failures = load_manifest(manifest, "linux")
        macos, macos_failures = load_manifest(manifest, "macos")
        if failures or macos_failures or len(linux) != 1 or len(macos) != 1 or linux == macos:
            raise AssertionError("platform-specific metric exceptions must select only their target")
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
    with tempfile.TemporaryDirectory() as directory:
        root = ROOT
        original_tracked_rust = tracked_rust
        original_baseline = baseline
        try:
            globals()["ROOT"] = Path(directory)
            large = ROOT / "src/large.rs"
            large.parent.mkdir()
            large.write_text("x\n" * 501)
            globals()["tracked_rust"] = lambda: ["src/large.rs", "src/absent.rs"]
            globals()["baseline"] = lambda path: ["x"] * 600 if path == "src/large.rs" else []
            marks = {"src/large.rs": 501}
            failures = check_structural(set(), marks)
            if failures != ["src/absent.rs: tracked Rust file is absent"]:
                raise AssertionError("a legitimate size mark and a tracked-but-absent file must be handled cleanly")
            large.write_text("x\n" * 502)
            failures = check_structural(set(), marks)
            if not any("high-water mark is 501, must not grow" in failure for failure in failures):
                raise AssertionError("one-line regrowth beyond a stored mark must fail")
            large.write_text("x\n" * 500)
            failures = check_structural(set(), {"src/large.rs": 500})
            if failures != ["src/absent.rs: tracked Rust file is absent"]:
                raise AssertionError("a legitimate shrink with a lowered mark must pass")
            failures = check_structural(set(), {"src/large.rs": 501})
            if not any("stored file-size mark is 501 lines but current file is 500" in failure for failure in failures):
                raise AssertionError(
                    "a mark that overstates the file's real current size (the manifest was not "
                    "lowered to match a shrink) must fail"
                )
            # This is NOT a general "raises are forbidden" check — see the point
            # below. A mark raised in lockstep with matching growth, staying
            # under the frozen baseline (600 here), is the exact technique item
            # 132 used, and check_structural alone accepts it: requiring a
            # reason for that is check_mark_raises's job, not this function's.
            large.write_text("x\n" * 501)
            failures = check_structural(set(), {"src/large.rs": 501})
            if failures != ["src/absent.rs: tracked Rust file is absent"]:
                raise AssertionError(
                    "a mark raised in lockstep with matching growth, still under the frozen "
                    "baseline, is structurally legitimate on its own"
                )
            large.write_text("x\n" * 500)
            new = ROOT / "src/new.rs"
            new.write_text("x\n" * 501)
            globals()["tracked_rust"] = lambda: ["src/large.rs", "src/absent.rs", "src/new.rs"]
            failures = check_structural(set(), {"src/large.rs": 500})
            if not any("src/new.rs: 501 lines (production limit is 500)" == failure for failure in failures):
                raise AssertionError("a new oversized production file must still fail")
        finally:
            globals()["ROOT"] = root
            globals()["tracked_rust"] = original_tracked_rust
            globals()["baseline"] = original_baseline
    # check_mark_raises is the half of the invariant check_structural does not
    # cover: a raise below the frozen baseline is only legitimate with a
    # recorded reason. Pure and directly testable, unlike previous_marks
    # (which shells out to git for the `main` reference).
    raised_no_reason = check_mark_raises({"src/x.rs": 105}, {}, {"src/x.rs": 100})
    if not any("raised from 100 to 105" in failure for failure in raised_no_reason):
        raise AssertionError("a mark raised on this branch with no recorded reason must fail")
    if check_mark_raises(
        {"src/x.rs": 105},
        {"src/x.rs": "item X: the one dispatch call site, irreducible"},
        {"src/x.rs": 100},
    ):
        raise AssertionError("a mark raised with a recorded reason must pass")
    if check_mark_raises({"src/x.rs": 95}, {}, {"src/x.rs": 100}):
        raise AssertionError("a lowered mark needs no reason")
    if check_mark_raises({"src/x.rs": 105}, {}, {"src/y.rs": 100}):
        raise AssertionError("a brand-new mark absent from main's table is not a raise")
    if check_mark_raises({"src/x.rs": 105}, {}, None):
        raise AssertionError("an unresolvable `main` reference must skip the raise audit, not fail closed")
    with tempfile.TemporaryDirectory() as directory:
        manifest = Path(directory) / "marks.toml"
        manifest.write_text(
            '[[file_size_mark]]\n'
            'file = "src/reasoned.rs"\n'
            'lines = 600\n'
            'reason = "item X: the one dispatch call site"\n\n'
            '[[file_size_mark]]\n'
            'file = "src/blank.rs"\n'
            'lines = 600\n'
            'reason = "   "\n'
        )
        _, reasons, failures = load_file_size_marks(manifest)
        if reasons.get("src/reasoned.rs") != "item X: the one dispatch call site":
            raise AssertionError("a non-empty file-size-mark reason must be captured")
        if not any("empty file-size-mark reason for src/blank.rs" in failure for failure in failures):
            raise AssertionError("a blank file-size-mark reason must fail")
    # previous_marks must distinguish a real prior branch state from the two
    # ways the comparison can mean nothing: an unresolvable reference, and the
    # vacuous case a comparison-to-itself hides — HEAD already being `main`'s
    # commit, which is the ordinary state for CI's push-to-`main` job and the
    # merge train's post-merge candidate. Fully mocked, deterministic
    # regardless of the environment running this self-test.
    original_git = git
    try:
        def fake_git_ok(*args: str) -> str:
            if args == ("rev-parse", "main"):
                return "mainsha\n"
            if args == ("rev-parse", "HEAD"):
                return "branchsha\n"
            # Exact-args, not args[0] == "show": if the implementation ever
            # regresses to reading main's tip instead of the merge base, this
            # mock raises instead of silently handing back a plausible table.
            if args == ("merge-base", "branchsha", "mainsha"):
                return "basesha\n"
            if args == ("show", "basesha:scripts/code-health.toml"):
                return '[[file_size_mark]]\nfile = "src/mocked.rs"\nlines = 42\n'
            raise subprocess.CalledProcessError(1, args)

        globals()["git"] = fake_git_ok
        marks, status = previous_marks()
        if status != "ok" or marks != {"src/mocked.rs": 42}:
            raise AssertionError(
                "previous_marks must resolve the merge-base table when HEAD diverges from main"
            )

        def fake_git_same(*args: str) -> str:
            if args in (("rev-parse", "main"), ("rev-parse", "HEAD")):
                return "samesha\n"
            raise subprocess.CalledProcessError(1, args)

        globals()["git"] = fake_git_same
        marks, status = previous_marks()
        if status != "head_is_main" or marks is not None:
            raise AssertionError(
                "HEAD identical to main's commit must report head_is_main, not a clean 'ok' audit"
            )

        def fake_git_unresolvable(*args: str) -> str:
            raise subprocess.CalledProcessError(1, args)

        globals()["git"] = fake_git_unresolvable
        marks, status = previous_marks()
        if status != "unresolvable" or marks is not None:
            raise AssertionError("an unresolvable main/origin-main must report unresolvable")

        def fake_git_no_common_ancestor(*args: str) -> str:
            if args == ("rev-parse", "main"):
                return "mainsha\n"
            if args == ("rev-parse", "HEAD"):
                return "branchsha\n"
            if args == ("merge-base", "branchsha", "mainsha"):
                raise subprocess.CalledProcessError(1, args)
            raise subprocess.CalledProcessError(1, args)

        globals()["git"] = fake_git_no_common_ancestor
        marks, status = previous_marks()
        if status != "unresolvable" or marks is not None:
            raise AssertionError(
                "a merge base that cannot be resolved must join the named unresolvable "
                "skip, never pass silently and never fail closed"
            )
    finally:
        globals()["git"] = original_git
    # And against the real repo, without checking out `main`: forcing
    # `head_ref="main"` makes HEAD-vs-main trivially a comparison against
    # itself, exercising the exact silent-vacuity shape live rather than
    # mocked. It must never come back "ok".
    marks, status = previous_marks(head_ref="main")
    if status == "ok":
        raise AssertionError("comparing main against itself must never report a real (ok) prior state")
    if status == "head_is_main" and marks is not None:
        raise AssertionError("head_is_main status must carry no marks")
    # The regression fixture, built as real git history rather than mocked
    # plumbing: a branch forks from main, main *lowers* a mark (the extraction
    # this board actively encourages — items 184/185's real shape), and the
    # forked branch never touches the file. Direction 1: that branch must
    # pass, because it raised nothing — comparing against main's tip instead
    # of the fork point is exactly the bug item 186 hit. Direction 2: the same
    # branch then genuinely raises the mark itself with no reason, and must
    # fail — proving the fix stopped comparing against the wrong commit, not
    # that it stopped checking raises at all.
    with tempfile.TemporaryDirectory() as directory:
        root = ROOT
        try:
            globals()["ROOT"] = Path(directory)
            git("init", "-q")
            git("config", "user.email", "code-health-selftest@example.com")
            git("config", "user.name", "code-health-selftest")
            git("checkout", "-q", "-b", "main")
            (Path(directory) / "scripts").mkdir()
            toml_path = Path(directory) / "scripts/code-health.toml"
            toml_path.write_text('[[file_size_mark]]\nfile = "src/big.rs"\nlines = 752\n')
            git("add", "-A")
            git("commit", "-q", "-m", "fork point: src/big.rs at 752")
            git("checkout", "-q", "-b", "feature")
            git("checkout", "-q", "main")
            toml_path.write_text('[[file_size_mark]]\nfile = "src/big.rs"\nlines = 747\n')
            git("commit", "-q", "-am", "pay the health bill by extraction: 752 -> 747")
            git("checkout", "-q", "feature")

            previous, status = previous_marks()
            if status != "ok" or previous != {"src/big.rs": 752}:
                raise AssertionError(
                    "a branch that forked before main lowered a mark must be judged "
                    f"against its own fork point (752), not main's tip: got {status!r} {previous!r}"
                )
            current_marks, current_reasons, load_failures = load_file_size_marks(toml_path)
            if load_failures:
                raise AssertionError(f"fixture manifest must load cleanly: {load_failures}")
            failures = check_mark_raises(current_marks, current_reasons, previous)
            if failures:
                raise AssertionError(
                    "lowering a mark on main must never fail an unrelated branch that "
                    f"never touched the file: {failures}"
                )

            toml_path.write_text('[[file_size_mark]]\nfile = "src/big.rs"\nlines = 760\n')
            git("commit", "-q", "-am", "raise the mark on this branch, no reason recorded")
            previous, status = previous_marks()
            if status != "ok" or previous != {"src/big.rs": 752}:
                raise AssertionError(
                    f"the fork point must stay 752 after a further commit on the branch: got {status!r} {previous!r}"
                )
            current_marks, current_reasons, load_failures = load_file_size_marks(toml_path)
            if load_failures:
                raise AssertionError(f"fixture manifest must load cleanly: {load_failures}")
            failures = check_mark_raises(current_marks, current_reasons, previous)
            if not any("src/big.rs: file-size mark raised from 752 to 760" in failure for failure in failures):
                raise AssertionError(
                    f"a branch that genuinely raises a mark with no reason must still fail: {failures}"
                )
        finally:
            globals()["ROOT"] = root
    # False-positive coverage matters as much as citation detection: a noisy
    # rule invites workarounds instead of better comments.
    citation_cases = [
        ("/// item 42: some archaeology", True),
        ("// ROUND 3: case-insensitive, same as the measuring grep's -i", True),
        ("    /// ITEM 116a — a sub-item suffix, no trailing word boundary", True),
        ("        // ITEM 131b — same suffix shape, lowercase item, uppercase ITEM", True),
        ("//! fixed in `24477b88`", True),
        ("// the active item in the menu", False),  # "item" with no number
        ("// round the corner radius to the nearest pixel", False),  # "round" with no number
        ("/// `1234567`", False),  # hud.rs's real thousands-separator example: all-digit, not a sha
        ("let item = 5; // item 9 trailing, not a whole-line comment", False),  # matches the grep's own scope, not the substring
        ("///     `/188` — permissive replay `replay_skips`.", False),  # capture.rs's own row shape carries no bare item/round/sha token
        ("    // ordinary present-tense comment, no citation", False),
    ]
    for text, expected in citation_cases:
        if is_comment_citation_line(text) != expected:
            raise AssertionError(f"is_comment_citation_line({text!r}) must be {expected}")
    if not is_index_named_test_file("src/render/tests/backgrounds_item158.rs"):
        raise AssertionError("a tests/ file whose name carries an item number must be index-named")
    if not is_index_named_test_file("src/actions/tests/alternate_accept_item116c.rs"):
        raise AssertionError("a trailing-letter item suffix (116c) must still count as index-named")
    if is_index_named_test_file("src/render/tests/nits.rs"):
        raise AssertionError("an ordinary test file with no item number must not be index-named")
    if is_index_named_test_file("src/item42.rs"):
        raise AssertionError("a production path outside tests/ must never be exempted by filename alone")
    if not is_index_named_test_citation(
        "src/render/tests/backgrounds_item158.rs", "// item 158: indexed test family"
    ):
        raise AssertionError("an index-named test may cite its own family")
    if is_index_named_test_citation(
        "src/render/tests/backgrounds_item158.rs", "// item 99: unrelated history"
    ):
        raise AssertionError("an index-named test must not excuse a different item")
    schema_row = "/// `/188` — permissive replay `replay_skips`."
    if not is_capture_schema_history_row("src/capture.rs", schema_row):
        raise AssertionError("capture.rs's own schema-history row shape must be recognized")
    if is_capture_schema_history_row("src/other.rs", schema_row):
        raise AssertionError("the schema-row exception must be scoped to capture.rs, not any file")
    if is_capture_schema_history_row("src/capture.rs", "/// an unrelated doc comment"):
        raise AssertionError("the schema-row exception must match the row's own shape, not the whole file")
    with tempfile.TemporaryDirectory() as directory:
        target = Path(directory) / "src/measured.rs"
        target.parent.mkdir()
        target.write_text("fn f() {}\n// item 9: a measured perf number pinned to its commit\n")
        manifest = Path(directory) / "health.toml"
        manifest.write_text(
            '[[comment_citation_exception]]\n'
            'target = "src/measured.rs"\n'
            'line = 2\n'
            'text = "// item 9: a measured perf number pinned to its commit"\n'
            'reason = "the cited item is the perf baseline itself, not narration"\n'
        )
        root = ROOT
        try:
            globals()["ROOT"] = Path(directory)
            allowed, failures = citation_exceptions(manifest)
            if failures or ("src/measured.rs", 2, "// item 9: a measured perf number pinned to its commit") not in allowed:
                raise AssertionError("a live comment-citation exception must be accepted")
            target.write_text("fn f() {}\n// item 9: the line moved or changed\n")
            _, failures = citation_exceptions(manifest)
            if not failures:
                raise AssertionError("a stale comment-citation exception (moved/changed line) must fail")
        finally:
            globals()["ROOT"] = root
    # check_comment_citations directly: each of the three named exceptions
    # dodges the failure on its own terms, and a plain new citation with none
    # of them still fails by name.
    added_cases = [
        ("src/render/tests/backgrounds_item158.rs", 3, "// item 158: index-named test file"),
        ("src/capture.rs", 6, "/// `/188` — permissive replay `replay_skips`."),
        ("src/measured.rs", 2, "// item 9: a measured perf number pinned to its commit"),
        ("src/plain.rs", 4, "// item 77: fresh archaeology with no exception"),
    ]
    direct_failures = check_comment_citations(
        added_cases,
        {("src/measured.rs", 2, "// item 9: a measured perf number pinned to its commit")},
    )
    if len(direct_failures) != 1 or "src/plain.rs:4" not in direct_failures[0]:
        raise AssertionError(
            f"exactly the unexempted new citation must fail, the three named exceptions must not: {direct_failures}"
        )
    if check_comment_citations(None, set()):
        raise AssertionError("a None added-list (non-ok status) must never fail")
    # The real-git fixture asserts both directions because a fix that
    # stops false-alarming and a fix that stops checking are indistinguishable
    # from outside unless both are tested.
    with tempfile.TemporaryDirectory() as directory:
        root = ROOT
        try:
            globals()["ROOT"] = Path(directory)
            git("init", "-q")
            git("config", "user.email", "code-health-selftest@example.com")
            git("config", "user.name", "code-health-selftest")
            git("checkout", "-q", "-b", "main")
            src_dir = Path(directory) / "src"
            src_dir.mkdir()
            existing = src_dir / "existing.rs"
            existing.write_text(
                "// plain production comment, no citation\n"
                "// item 1: pre-existing archaeology that predates this ratchet\n"
                "fn existing() {}\n"
            )
            git("add", "-A")
            git("commit", "-q", "-m", "fork point: existing.rs carries one old citation")
            git("checkout", "-q", "-b", "feature")

            # A DIFFERENT lane lands its own brand-new citation on `main`
            # after the fork — must never bleed into `feature`'s audit (the
            # unrelated file doesn't even exist in feature's working tree).
            git("checkout", "-q", "main")
            unrelated_on_main = src_dir / "unrelated.rs"
            unrelated_on_main.write_text(
                "// item 2: a different lane's citation, landed on main after the fork\n"
                "fn unrelated() {}\n"
            )
            git("add", "-A")
            git("commit", "-q", "-m", "a different lane lands its own citation on main")
            git("checkout", "-q", "feature")

            # Direction 1: touch existing.rs WITHOUT adding a new citation —
            # must not fail even though the file already carries one. This is
            # Compare against the fork point, not main's moving tip.
            existing.write_text(
                existing.read_text() + "fn touched() { /* unrelated new code, no citation */ }\n"
            )
            git("commit", "-q", "-am", "touch existing.rs without adding a citation")
            added, status = new_comment_citations()
            if status != "ok":
                raise AssertionError(f"fixture merge base must resolve: got {status!r}")
            failures = check_comment_citations(added, set())
            if failures:
                raise AssertionError(
                    "touching a file that already carries a citation, and an unrelated "
                    f"citation landing on main, must never fail this branch: {failures}"
                )

            # Direction 2: the SAME branch then genuinely adds a new citation
            # — must fail by name, proving direction 1 wasn't bought by
            # silently skipping the check altogether.
            existing.write_text(
                existing.read_text() + "// item 3: a fresh citation added by this branch\n"
            )
            git("commit", "-q", "-am", "add a fresh citation on this branch")
            added, status = new_comment_citations()
            if status != "ok":
                raise AssertionError(f"fixture merge base must resolve: got {status!r}")
            failures = check_comment_citations(added, set())
            if not any("existing.rs:5" in failure and "item 3" in failure for failure in failures):
                raise AssertionError(
                    f"a branch that genuinely adds a new citation must fail by name: {failures}"
                )
        finally:
            globals()["ROOT"] = root
    script = '''if (( $# != 0 )); then
canary_command=(cargo test --test native_gate_canary)
mac_command=(env AWL_CONVENTION_FORCE=mac cargo test)
linux_command=(env AWL_CONVENTION_FORCE=linux cargo test)
start_commit="$(git rev-parse HEAD)"
export RUST_TEST_THREADS
printf 'native-gate-env cpus=%s\\n' "$gate_cpus"
ps -A -o pid=,pgid=,time=,comm=
ps -A -o pid=,ppid=,pgid=,etime=,time=,rss=,stat=,comm=
gate_cpu_sample >"$gate_cpu_prev"
printf 'native-gate-vitals load1=%s cpu_count=%s %s\\n' "$(gate_load1)" "$gate_cpus" "$(gate_cpu_report)"
gate_budget_seconds=$(( AWL_NATIVE_GATE_DEADLINE_EPOCH - gate_started_epoch ))
printf 'native-gate-phase label=%s event=%s elapsed_seconds=%s %s\\n' "$1" "$2" "$(gate_elapsed)" "${3:-}"
"$@" 2>&1 | gate_stamp_phases "$label"
kill "-$signal" "-$pgid"
gate_launch budget_pid untracked gate_sleep_then "$gate_budget_seconds" gate_budget_expired
"${canary_command[@]}"
"${mac_command[@]}" &
mac_pid=$!
"${linux_command[@]}" &
linux_pid=$!
set +e
wait "$mac_pid"
mac_status=$?
wait "$linux_pid"
linux_status=$?
set -e
if [[ -f "$gate_budget_marker" ]]; then
  exit 1
fi
if (( mac_status != 0 || linux_status != 0 )); then
  exit 1
fi
end_commit="$(git rev-parse HEAD)"
printf 'native-gate-receipt commit=%s conventions=mac,linux scope=all-targets\\n' "$end_commit"
'''
    ci = '''  linux:
    steps:
      - run: scripts/native-gate.sh
  mac:
    steps:
      - run: echo "AWL_NATIVE_GATE_DEADLINE_EPOCH=$(( $(date +%s) + 2100 ))" >> "$GITHUB_ENV"
      - env:
          AWL_NATIVE_GATE_BUDGET_SECONDS: 1500
        run: scripts/native-gate.sh
'''
    if native_gate_audit(script, ci):
        raise AssertionError("canonical native-gate shape must pass its external audit")
    mutations = {
        "--bin": (script.replace("env AWL_CONVENTION_FORCE=mac cargo test", "cargo test --bin awl"), ci,
                   "native suite command for mac must be unfiltered"),
        "omitted Linux convention": (script.replace("linux_command=(env AWL_CONVENTION_FORCE=linux cargo test)\n", ""), ci,
                                       "missing Linux full-suite convention"),
        "skipped canary": (script.replace("canary_command=(cargo test --test native_gate_canary)\n", ""), ci,
                           "missing named integration-only canary"),
        "stale SHA": (script.replace('end_commit="$(git rev-parse HEAD)"', 'end_commit="$start_commit"'), ci,
                      "receipt must resolve HEAD after both suites"),
        "dropped mac status": (script.replace("mac_status != 0 || ", ""), ci,
                               "either convention failure must suppress the receipt"),
        "unawaited Linux": (script.replace('wait "$linux_pid"', 'true # dropped Linux wait'), ci,
                            "gate must await the Linux convention"),
        "CI bypass": (script, ci.replace("scripts/native-gate.sh", "cargo test"),
                      "CI linux job must call scripts/native-gate.sh"),
        "unbounded test threads": (script.replace("export RUST_TEST_THREADS\n", ""), ci,
                                   "must bound per-convention test-thread concurrency"),
        "silent machine": (script.replace("""printf 'native-gate-env cpus=%s\\n' "$gate_cpus"\n""", ""), ci,
                           "must state the machine and bound it is about to load"),
        "budget leaks a receipt": (script.replace('if [[ -f "$gate_budget_marker" ]]; then\n  exit 1\nfi\n', ""), ci,
                                   "an exhausted budget must suppress the receipt"),
        "bound applied after launch": (
            script.replace("export RUST_TEST_THREADS\n", "")
                  .replace('linux_pid=$!\n', 'linux_pid=$!\nexport RUST_TEST_THREADS\n'),
            ci,
            "thread bound and the machine receipt must precede both suites"),
        # The 2026-08-02 repairs. Each mutation is the shape the defect actually
        # had on a real runner, not an invented one.
        "budget blind to the runner's clock": (
            script.replace("gate_budget_seconds=$(( AWL_NATIVE_GATE_DEADLINE_EPOCH - gate_started_epoch ))\n", ""),
            ci,
            "must honour an absolute caller deadline"),
        "budget armed after the canary": (
            script.replace('gate_launch budget_pid untracked gate_sleep_then "$gate_budget_seconds" gate_budget_expired\n', "")
                  .replace('linux_pid=$!\n',
                           'linux_pid=$!\ngate_launch budget_pid untracked gate_sleep_then "$gate_budget_seconds" gate_budget_expired\n'),
            ci,
            "budget must be armed before the canary"),
        "kills lone pids": (
            script.replace('kill "-$signal" "-$pgid"', 'kill "-$signal" "$pgid"'), ci,
            "must end whole process groups, not lone pids"),
        "unfiltered convention output": (
            script.replace('"$@" 2>&1 | gate_stamp_phases "$label"', '"$@"'), ci,
            "output must pass through the gate's own filter"),
        "unstamped phases": (
            script.replace(
                """printf 'native-gate-phase label=%s event=%s elapsed_seconds=%s %s\\n' "$1" "$2" "$(gate_elapsed)" "${3:-}"\n""",
                ""),
            ci,
            "must stamp its phase boundaries with elapsed time"),
        "mac duration bound dropped": (
            script, ci.replace("          AWL_NATIVE_GATE_BUDGET_SECONDS: 1500\n", ""),
            "CI mac job must bound the gate's own duration"),
        "mac runner clock dropped": (
            script, ci.replace(
                '      - run: echo "AWL_NATIVE_GATE_DEADLINE_EPOCH=$(( $(date +%s) + 2100 ))" >> "$GITHUB_ENV"\n', ""),
            "CI mac job must anchor the gate to the runner's clock"),
        # Deadlock against livelock. Each requirement is mutated TWICE — deleted
        # outright, and merely commented out — because a substring matcher that
        # reads the raw text is satisfied by the comment, and that is how this
        # auditor was green over a disabled bound once already.
        "no load average": (
            script.replace("""printf 'native-gate-vitals load1=%s cpu_count=%s %s\\n' "$(gate_load1)" "$gate_cpus" "$(gate_cpu_report)"\n""", ""),
            ci,
            "must report the system load beside the core count"),
        "load average demoted to a comment": (
            script.replace(
                """printf 'native-gate-vitals load1=%s cpu_count=%s""",
                """# printf 'native-gate-vitals load1=%s cpu_count=%s"""),
            ci,
            "must report the system load beside the core count"),
        # `ps -o pcpu` is a lifetime average on Linux and a decayed one on
        # macOS: a suite that ran hot then hung reads ~9% on one and ~0% on the
        # other, and neither is about the interval in question.
        "per-process CPU replaced by ps pcpu": (
            script.replace("ps -A -o pid=,pgid=,time=,comm=", "ps -A -o pid=,pgid=,pcpu=,comm="), ci,
            "must sample cumulative per-process CPU time"),
        "per-process CPU sample demoted to a comment": (
            script.replace("ps -A -o pid=,pgid=,time=,comm=", "# ps -A -o pid=,pgid=,time=,comm="), ci,
            "must sample cumulative per-process CPU time"),
        # Load alone cannot say WHICH process is spinning, which is the whole
        # difference between "the runner is oversubscribed" and "attach a
        # debugger to this pid".
        "load average without per-process attribution": (
            script.replace(""" "$(gate_cpu_report)"\n""", "\n"), ci,
            "must carry per-tracked-process CPU"),
        "per-process CPU report demoted to a comment": (
            script.replace(
                """printf 'native-gate-vitals load1=%s cpu_count=%s %s\\n'""",
                """# printf 'native-gate-vitals load1=%s cpu_count=%s %s\\n'"""),
            ci,
            "must carry per-tracked-process CPU"),
        "CPU baseline taken after the first heartbeat": (
            script.replace('gate_cpu_sample >"$gate_cpu_prev"\n', "")
                  .replace('linux_pid=$!\n', 'linux_pid=$!\ngate_cpu_sample >"$gate_cpu_prev"\n'),
            ci,
            "CPU baseline must be sampled before the first heartbeat"),
        "CPU baseline demoted to a comment": (
            script.replace('gate_cpu_sample >"$gate_cpu_prev"', '# gate_cpu_sample >"$gate_cpu_prev"'),
            ci,
            "CPU baseline must be sampled before the first heartbeat"),
        "abort dump without CPU time": (
            script.replace("pid=,ppid=,pgid=,etime=,time=,rss=,stat=,comm=", "pid=,ppid=,pgid=,etime=,rss=,stat=,comm="),
            ci,
            "abort's process dump must carry CPU time beside elapsed time"),
        "abort dump demoted to a comment": (
            script.replace("ps -A -o pid=,ppid=,pgid=,etime=,time=", "# ps -A -o pid=,ppid=,pgid=,etime=,time="),
            ci,
            "abort's process dump must carry CPU time beside elapsed time"),
        # The mutation this audit failed on its first run: a deleted bound that
        # a comment mentioning the same variable silently stood in for.
        "mac bound demoted to a comment": (
            script, ci.replace(
                "          AWL_NATIVE_GATE_BUDGET_SECONDS: 1500",
                "          # AWL_NATIVE_GATE_BUDGET_SECONDS: 1500"),
            "CI mac job must bound the gate's own duration"),
    }
    for mutation, (bad_script, bad_ci, expected) in mutations.items():
        failures = native_gate_audit(bad_script, bad_ci)
        if not any(expected in failure for failure in failures):
            raise AssertionError(f"native-gate audit mutation {mutation!r} did not fail by name: {failures}")
    print("code-health: self-test clean")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--native-gate-audit", action="store_true")
    parser.add_argument(
        "--comment-citation-backlog",
        action="store_true",
        help="print the measured item/round/sha comment-citation backlog per top-level src module and exit",
    )
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    if args.comment_citation_backlog:
        counts = comment_citation_backlog()
        total = sum(counts.values())
        for module, count in sorted(counts.items(), key=lambda kv: (-kv[1], kv[0])):
            print(f"src/{module}: {count}")
        print(f"total: {total}")
        return 0
    if args.native_gate_audit:
        failures = native_gate_audit(
            (ROOT / "scripts/native-gate.sh").read_text(),
            (ROOT / ".github/workflows/ci.yml").read_text(),
        )
        if failures:
            print("\n".join(failures), file=sys.stderr)
            return 1
        print("native-gate-audit: canonical gate and CI wiring clean")
        return 0
    try:
        git("cat-file", "-e", f"{BASELINE}^{{commit}}")
    except subprocess.CalledProcessError as error:
        raise SystemExit(f"code-health: stale baseline {BASELINE}; refresh it deliberately") from error
    expected, failures = load_manifest()
    file_size_marks, mark_reasons, mark_failures = load_file_size_marks()
    failures.extend(mark_failures)
    allowed, structural_failures = structural_exceptions()
    failures.extend(structural_failures)
    failures.extend(check_structural(allowed, file_size_marks))
    previous, previous_status = previous_marks()
    if previous_status == "unresolvable":
        print(
            "code-health: SKIPPED the file-size-mark raise audit (`main`/`origin/main`, or the "
            "merge base between HEAD and it, not resolvable in this checkout; a raise is not "
            "verified against a reason this run).",
            file=sys.stderr,
        )
    elif previous_status == "head_is_main":
        print(
            "code-health: SKIPPED the file-size-mark raise audit — HEAD is already `main`'s "
            "commit, so there is no prior branch state to diff against (comparing a commit's "
            "manifest to itself). This is the normal state for CI's push-to-`main` run and the "
            "merge train's post-merge candidate; the audit only has force on a worktree branch "
            "checked before landing.",
            file=sys.stderr,
        )
    failures.extend(check_mark_raises(file_size_marks, mark_reasons, previous))
    new_citations, citation_status = new_comment_citations()
    if citation_status == "unresolvable":
        print(
            "code-health: SKIPPED the new-comment-citation ratchet (`main`/`origin/main`, or the "
            "merge base between HEAD and it, not resolvable in this checkout; a newly added "
            "queue-item/round/sha citation is not verified this run).",
            file=sys.stderr,
        )
    elif citation_status == "head_is_main":
        print(
            "code-health: SKIPPED the new-comment-citation ratchet — HEAD is already `main`'s "
            "commit, so there is no prior branch state to diff against. This is the normal state "
            "for CI's push-to-`main` run and the merge train's post-merge candidate; the ratchet "
            "only has force on a worktree branch checked before landing.",
            file=sys.stderr,
        )
    citation_allowed, citation_failures = citation_exceptions()
    failures.extend(citation_failures)
    failures.extend(check_comment_citations(new_citations, citation_allowed))
    failures.extend(
        native_gate_audit(
            (ROOT / "scripts/native-gate.sh").read_text(),
            (ROOT / ".github/workflows/ci.yml").read_text(),
        )
    )
    failures.extend(check_clippy(run_metric_clippy(), expected))
    if failures:
        print("code-health: policy check failed", file=sys.stderr)
        print("\n".join(failures), file=sys.stderr)
        return 1
    print(f"code-health: structural and Clippy ratchets clean (baseline {BASELINE}; {len(expected)} Clippy exceptions)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
