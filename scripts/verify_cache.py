#!/usr/bin/env python3
"""Conservative result reuse for a SMALL, EXPLICIT registry of checks.

docs/verification.md: "A check's reusable evidence must include its inputs:
source and assets, tests and fixtures, build configuration, toolchain,
environment branches, and relevant hardware. A change to any of these
invalidates the affected result. Unknown dependencies mean rerun, not guessed
independence." This module is that mechanism, and nothing more:

  - Checks are registered by NAME in scripts/verify-cache.toml, one row per
    check, each with its OWN explicit file globs, environment-variable
    branches, and command. There is no dependency graph and no discovery: a
    check not in the manifest is never cached (`run` on an unregistered name
    is a hard error), and a glob that does not list a file gives that file no
    power to invalidate anything — enrollment is a conscious, reviewed edit
    to the manifest, the same shape as println_audit.rs's own table.
  - The reuse decision hashes every GIT-TRACKED file the check's globs match
    (source, tests, and config are recorded as separate categories, so a
    change in any one is visible, but they fold into one signature — a test
    file IS a source file for this purpose, and changing it invalidates the
    result exactly because it changed a file the signature covers), plus the
    resolved command, the toolchain identifier, the hardware class, and the
    declared environment-variable branch values. ANY difference reruns; nothing
    is inferred as safe.
  - A record is written ONLY after the real command has actually returned, by
    an atomic write (temp file + rename) under a per-check lock. A killed
    process (this script or the command it runs) therefore leaves no
    fabricated record: the old record, if any, is untouched, and the next
    invocation reruns for real. A cached FAILURE is never treated as
    reusable — only a recorded PASS with a matching signature is reused, so a
    failed run can only become green by actually passing again.
  - Reusing a result never rewrites its stored commit: the record continues
    to name the commit it was ACTUALLY produced on, and a reuse message
    prints that commit alongside the current one rather than silently
    relabeling the evidence as being about the new commit.

This is deliberately NOT wired into scripts/native-gate.sh's receipt path.
docs/verification.md: "Preserve the current full-candidate gate until the
replacement can prove equivalent coverage." This tool exists to be measured
against that gate, not to replace it yet.
"""

from __future__ import annotations

import argparse
import fcntl
import hashlib
import json
import os
import subprocess
import sys
import time
import tomllib
from pathlib import Path
from typing import Any


class CacheError(Exception):
    """A manifest or invocation problem — never a check FAILURE."""


def repo_root(explicit: str | None) -> Path:
    if explicit:
        return Path(explicit).resolve()
    out = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        capture_output=True,
        text=True,
        check=True,
    )
    return Path(out.stdout.strip()).resolve()


def load_manifest(path: Path) -> dict[str, dict[str, Any]]:
    try:
        raw = path.read_text()
    except FileNotFoundError as exc:
        raise CacheError(f"manifest not found: {path}") from exc
    try:
        data = tomllib.loads(raw)
    except tomllib.TOMLDecodeError as exc:
        raise CacheError(f"manifest is not valid TOML: {path}: {exc}") from exc
    checks: dict[str, dict[str, Any]] = {}
    for entry in data.get("check", []):
        missing = {"name", "command", "source_globs"} - entry.keys()
        if missing:
            raise CacheError(f"malformed check entry missing {sorted(missing)}: {entry}")
        if entry["name"] in checks:
            raise CacheError(f"duplicate check name in manifest: {entry['name']}")
        for field in ("source_globs", "tests", "config"):
            validate_globs(entry.get(field, []), entry["name"], field)
        checks[entry["name"]] = entry
    return checks


def validate_globs(globs: list[str], check_name: str, field: str) -> None:
    """Reject a `**` pathspec that forgot git's `:(glob)` magic. Without it,
    git's default pathspec fnmatch already lets a bare `*` cross `/` — so
    `src/**/*.rs` does NOT mean "every .rs file under src": it means "at
    least one literal path segment, then another `*.rs` segment", which
    EXCLUDES every file directly in src/ (src/render.rs, src/app.rs, ...).
    That is the dangerous direction: fewer files hashed than the author
    intended means a real change can go undetected and a stale PASS gets
    reused. This was caught live, once, exactly this way — a manifest
    written with plain `src/**/*.rs` silently dropped every top-level file
    in src/ from the signature. `:(glob)src/**/*.rs` gives `**` its
    conventional cross-directory meaning and restores plain `*` to
    single-segment matching."""
    for glob in globs:
        if "**" in glob and not glob.startswith(":(glob)"):
            raise CacheError(
                f"check {check_name!r} field {field!r} uses '**' without git's ':(glob)' "
                f"pathspec magic: {glob!r}. Without it, '**' does not mean recursive — it "
                "requires an extra literal path segment and SILENTLY EXCLUDES files directly "
                "in the parent directory. Prefix with ':(glob)' (e.g. ':(glob)src/**/*.rs')."
            )


def tracked_files(root: Path, globs: list[str]) -> list[str]:
    """Git-tracked files under root matching any of globs. Untracked files —
    build output, local scratch, an ignored fixture — can never contribute to
    a signature: only what git would commit is evidence."""
    if not globs:
        return []
    out = subprocess.run(
        ["git", "-C", str(root), "ls-files", "-z", "--", *globs],
        capture_output=True,
        check=True,
    )
    names = [n for n in out.stdout.decode("utf-8", "surrogateescape").split("\0") if n]
    return sorted(set(names))


def file_hash(root: Path, relpath: str) -> str:
    data = (root / relpath).read_bytes()
    return hashlib.sha256(data).hexdigest()


def toolchain_id() -> str:
    override = os.environ.get("AWL_VERIFY_CACHE_TOOLCHAIN_OVERRIDE")
    if override is not None:
        return override
    out = subprocess.run(
        ["rustc", "--version", "--verbose"], capture_output=True, text=True, check=True
    )
    return hashlib.sha256(out.stdout.encode()).hexdigest()[:16]


def hardware_class() -> str:
    override = os.environ.get("AWL_VERIFY_CACHE_HARDWARE_OVERRIDE")
    if override is not None:
        return override
    out = subprocess.run(["uname", "-srm"], capture_output=True, text=True, check=True)
    return out.stdout.strip()


def env_branch_values(names: list[str]) -> dict[str, str]:
    """The check's DECLARED environment dependencies only. A variable this
    check does not list has no power to invalidate its result — and none to
    hide behind, either: docs/verification.md's own tripwire is a check that
    reads an env var it did not declare/set, and the fix there is the same as
    here — state the axis at the check, explicitly, in the manifest."""
    return {name: os.environ.get(name, "") for name in sorted(names)}


def current_commit(root: Path) -> str:
    out = subprocess.run(
        ["git", "-C", str(root), "rev-parse", "HEAD"], capture_output=True, text=True, check=True
    )
    return out.stdout.strip()


def compute_signature(
    check: dict[str, Any], root: Path, toolchain: str, hw: str, env_vals: dict[str, str]
) -> tuple[str, dict[str, Any]]:
    categories = {
        "source": list(check.get("source_globs", [])),
        "tests": list(check.get("tests", [])),
        "config": list(check.get("config", [])),
    }
    all_globs = [g for globs in categories.values() for g in globs]
    files = tracked_files(root, all_globs)
    file_hashes = {f: file_hash(root, f) for f in files}
    payload = {
        "command": check["command"],
        "files": file_hashes,
        "toolchain": toolchain,
        "hardware_class": hw,
        "env": env_vals,
    }
    blob = json.dumps(payload, sort_keys=True).encode()
    signature = hashlib.sha256(blob).hexdigest()
    evidence = {
        "categories": categories,
        "files_hashed": len(files),
        "command": check["command"],
        "toolchain": toolchain,
        "hardware_class": hw,
        "env": env_vals,
    }
    return signature, evidence


def record_path(cache_dir: Path, name: str) -> Path:
    return cache_dir / f"{name}.json"


def load_record(path: Path) -> dict[str, Any] | None:
    """Missing or corrupt (bad JSON, unreadable, wrong shape) both read as
    "no usable record" — unknown state means rerun, never a guessed pass."""
    if not path.exists():
        return None
    try:
        data = json.loads(path.read_text())
    except (json.JSONDecodeError, OSError, UnicodeDecodeError):
        return None
    if not isinstance(data, dict) or "signature" not in data or "outcome" not in data:
        return None
    return data


def atomic_write(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(f"{path.name}.tmp{os.getpid()}")
    tmp.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")
    os.replace(tmp, path)  # atomic on the same filesystem


class CheckLock:
    """A per-check exclusive lock via fcntl.flock (POSIX syscall, available on
    both macOS and Linux) — NOT the Linux-only `flock(1)` CLI. Held across the
    whole read-decide-run-write sequence, so two concurrent invocations of the
    same check never both decide "cache miss" and run twice, and never
    interleave their writes."""

    def __init__(self, cache_dir: Path, name: str):
        cache_dir.mkdir(parents=True, exist_ok=True)
        self.path = cache_dir / f"{name}.lock"
        self.fh = open(self.path, "a+")

    def __enter__(self) -> "CheckLock":
        fcntl.flock(self.fh.fileno(), fcntl.LOCK_EX)
        return self

    def __exit__(self, *_exc: object) -> None:
        fcntl.flock(self.fh.fileno(), fcntl.LOCK_UN)
        self.fh.close()


def run_check(
    name: str, manifest_path: Path, root: Path, cache_dir: Path, force: bool, out=sys.stdout
) -> int:
    checks = load_manifest(manifest_path)
    if name not in checks:
        print(
            f"verify-cache: '{name}' is not a registered check in {manifest_path}; "
            f"registered checks: {sorted(checks) or '(none)'}. An unrecognised check is "
            "never cached — register it deliberately or run its command directly.",
            file=sys.stderr,
        )
        return 2
    check = checks[name]

    toolchain = toolchain_id()
    hw = hardware_class()
    env_vals = env_branch_values(check.get("env_branches", []))

    with CheckLock(cache_dir, name):
        signature, evidence = compute_signature(check, root, toolchain, hw, env_vals)
        rec_path = record_path(cache_dir, name)
        record = None if force else load_record(rec_path)

        if record is not None and record.get("signature") == signature:
            if record.get("outcome") == "pass":
                print(
                    f"verify-cache: REUSED {name} — inputs unchanged since it last ran "
                    f"at commit {record.get('commit', '?')[:12]} (now at "
                    f"{current_commit(root)[:12]}); not rerun. "
                    f"{evidence['files_hashed']} tracked files, toolchain {toolchain}, "
                    f"hardware {hw}.",
                    file=out,
                )
                return 0
            print(
                f"verify-cache: cached result for {name} is a FAILURE recorded at commit "
                f"{record.get('commit', '?')[:12]}; a failed run cannot become green through "
                "bookkeeping — rerunning for real.",
                file=out,
            )
        elif record is None:
            reason = "forced" if force else "no usable record (missing, corrupt, or never run)"
            print(f"verify-cache: {name} — {reason}; running.", file=out)
        else:
            print(f"verify-cache: {name} — inputs changed since the cached run; running.", file=out)

        start = time.monotonic()
        proc = subprocess.run(check["command"], cwd=root)
        elapsed = time.monotonic() - start
        outcome = "pass" if proc.returncode == 0 else "fail"

        # Written ONLY here, ONLY after the command has actually returned. A
        # cancellation of this process (SIGINT/SIGTERM/SIGKILL) during
        # subprocess.run never reaches this line, so a killed run leaves
        # whatever record already existed (or none) untouched.
        atomic_write(
            rec_path,
            {
                "check": name,
                "signature": signature,
                "evidence": evidence,
                "outcome": outcome,
                "commit": current_commit(root),
                "elapsed_seconds": round(elapsed, 3),
                "recorded_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            },
        )
        print(f"verify-cache: {name} {outcome.upper()} in {elapsed:.2f}s (recorded)", file=out)
        return proc.returncode


def show_record(name: str, cache_dir: Path, out=sys.stdout) -> int:
    rec = load_record(record_path(cache_dir, name))
    if rec is None:
        print(f"verify-cache: no usable record for {name}", file=out)
        return 1
    print(json.dumps(rec, indent=2, sort_keys=True), file=out)
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--manifest", default=None, help="path to the check manifest TOML")
    parser.add_argument("--root", default=None, help="repository root (default: git rev-parse --show-toplevel)")
    parser.add_argument("--cache-dir", default=None, help="where records/locks live (default: <root>/.verify-cache)")
    sub = parser.add_subparsers(dest="action", required=True)

    run_p = sub.add_parser("run", help="run a registered check, reusing a cached PASS if inputs are unchanged")
    run_p.add_argument("name")
    run_p.add_argument("--force", action="store_true", help="ignore any cached record; always run for real")

    show_p = sub.add_parser("show", help="print the raw cached record for a check")
    show_p.add_argument("name")

    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        root = repo_root(args.root)
    except subprocess.CalledProcessError:
        print("verify-cache: not inside a git repository and --root not given", file=sys.stderr)
        return 2
    manifest_path = Path(args.manifest) if args.manifest else root / "scripts" / "verify-cache.toml"
    cache_dir = Path(args.cache_dir) if args.cache_dir else root / ".verify-cache"

    try:
        if args.action == "run":
            return run_check(args.name, manifest_path, root, cache_dir, args.force)
        if args.action == "show":
            return show_record(args.name, cache_dir)
    except CacheError as exc:
        print(f"verify-cache: {exc}", file=sys.stderr)
        return 2
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
