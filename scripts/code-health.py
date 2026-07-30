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


def previous_marks(
    manifest_path: str = "scripts/code-health.toml", head_ref: str = "HEAD"
) -> tuple[dict[str, int] | None, str]:
    """The file-size-mark table already committed on `main`, and a status
    naming whether that comparison means anything.

    Deliberately not the frozen BASELINE commit: marks did not exist that far
    back (`git show BASELINE:scripts/code-health.toml` has no such path), and
    BASELINE's own file sizes are the separate, much larger ceiling
    `check_structural` enforces directly against history.

    Status is one of:
      "ok"           — `head_ref` differs from the resolved `main`/
                        `origin/main` commit, so the returned table is a real
                        prior state and a raise against it is meaningful.
      "unresolvable" — neither `main` nor `origin/main` resolved (a shallow
                        or detached checkout, or `head_ref` itself doesn't
                        resolve). Skips the audit rather than failing every
                        worker's gate — the same tolerance code-health.sh
                        already grants its own Linux-target Clippy arm when
                        that target isn't installed.
      "head_is_main" — `head_ref` already IS the resolved commit, so there is
                        no prior branch state to diff against: `main`'s
                        manifest and the one being checked are the same
                        object, and a raise can never be observed no matter
                        what the table says. This is not a clean audit, it is
                        a vacuous one, and it is the *normal* state in two of
                        the three places this script runs on this
                        push-straight-to-`main` repo: CI's `code-health.sh`
                        job (always HEAD-on-`main`) and the merge train's
                        post-merge candidate. The raise audit has real force
                        in exactly one place — a worker's worktree branch,
                        checked before landing, which is also where item 132
                        would have been caught.

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
    text = git("show", f"{ref_sha}:{manifest_path}")
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
    required_script_lines = {
        'canary_command=(cargo test --test native_gate_canary)':
            "native-gate-audit: missing named integration-only canary",
        'mac_command=(env AWL_CONVENTION_FORCE=mac cargo test)':
            "native-gate-audit: native suite command for mac must be unfiltered",
        'linux_command=(env AWL_CONVENTION_FORCE=linux cargo test)':
            "native-gate-audit: missing Linux full-suite convention",
        'start_commit="$(git rev-parse HEAD)"':
            "native-gate-audit: receipt must capture HEAD before the suites",
        'end_commit="$(git rev-parse HEAD)"':
            "native-gate-audit: receipt must resolve HEAD after both suites",
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
    if min(canary, mac, linux, receipt) < 0 or not canary < mac < linux < receipt:
        failures.append(
            "native-gate-audit: canary, mac suite, Linux suite, and receipt must run in that order"
        )
    if "if (( $# != 0 )); then" not in script:
        failures.append("native-gate-audit: gate must reject target-selection and test-name arguments")

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
            if args[0] == "show":
                return '[[file_size_mark]]\nfile = "src/mocked.rs"\nlines = 42\n'
            raise subprocess.CalledProcessError(1, args)

        globals()["git"] = fake_git_ok
        marks, status = previous_marks()
        if status != "ok" or marks != {"src/mocked.rs": 42}:
            raise AssertionError("previous_marks must resolve a real prior table when HEAD differs from main")

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
    script = '''if (( $# != 0 )); then
canary_command=(cargo test --test native_gate_canary)
mac_command=(env AWL_CONVENTION_FORCE=mac cargo test)
linux_command=(env AWL_CONVENTION_FORCE=linux cargo test)
start_commit="$(git rev-parse HEAD)"
"${canary_command[@]}"
"${mac_command[@]}"
"${linux_command[@]}"
end_commit="$(git rev-parse HEAD)"
printf 'native-gate-receipt commit=%s conventions=mac,linux scope=all-targets\\n' "$end_commit"
'''
    ci = '''  linux:
    steps:
      - run: scripts/native-gate.sh
  mac:
    steps:
      - run: scripts/native-gate.sh
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
        "CI bypass": (script, ci.replace("scripts/native-gate.sh", "cargo test"),
                      "CI linux job must call scripts/native-gate.sh"),
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
    args = parser.parse_args()
    if args.self_test:
        return self_test()
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
            "code-health: SKIPPED the file-size-mark raise audit (`main`/`origin/main` not "
            "resolvable in this checkout; a raise is not verified against a reason this run).",
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
