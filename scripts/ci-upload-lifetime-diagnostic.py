#!/usr/bin/env python3
"""Run exact upload laws in bounded fresh processes and record process-tree RSS."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import queue
import signal
import subprocess
import sys
import threading
import time

EXPECTED_SOURCE = "c1584e36c6c4566a0c69e065a96bd344638a01d9"
PREFIX = "render::tests::foot_band_no_clip::upload_lifetime::"
LAWS = {
    PREFIX + "repeated_uploads_to_one_retained_texture_need_submission":
        "retained-texture staging buffers:",
    PREFIX + "geometry_only_cells_do_not_retain_a_sweeps_uploads":
        "geometry-only retained buffers: 128 cells",
}


def test_binary(metadata: Path) -> str:
    binaries = set()
    for line in metadata.read_text().splitlines():
        value = json.loads(line)
        if (value.get("reason") == "compiler-artifact"
                and value.get("profile", {}).get("test")
                and "lib" in value.get("target", {}).get("kind", [])
                and value.get("executable")):
            binaries.add(value["executable"])
    if len(binaries) != 1:
        raise RuntimeError(f"expected one compiled library test executable, got {binaries}")
    return binaries.pop()


def validate_source(binary: str, root: Path | None = None) -> dict:
    commit = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
    path = Path(binary).resolve()
    if commit != EXPECTED_SOURCE:
        raise RuntimeError(f"wrong measured source: {commit}")
    root = root or Path.cwd()
    if not path.is_relative_to((root / "target/debug/deps").resolve()) or not path.is_file():
        raise RuntimeError("test executable must belong to this source checkout's debug deps")
    return {"source_commit": commit, "executable_sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


def stop_group(proc: subprocess.Popen) -> None:
    try:
        os.killpg(proc.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    try:
        proc.wait(timeout=10)
    except subprocess.TimeoutExpired:
        pass


def process_rss(pid: int) -> int:
    listing = subprocess.check_output(
        ["ps", "-axo", "pid=,ppid=,rss="], text=True, timeout=3
    )
    rows = [tuple(map(int, line.split())) for line in listing.splitlines() if line.strip()]
    family = {pid}
    while True:
        expanded = family | {p for p, parent, _ in rows if parent in family}
        if expanded == family:
            break
        family = expanded
    return sum(rss for p, _, rss in rows if p in family)


def passed(output: str, name: str, marker: str, exit_code: int, timed_out: bool) -> bool:
    return (exit_code == 0 and not timed_out and marker in output
            and f"test {name} ..." in output
            and "1 passed; 0 failed; 0 ignored" in output
            and "skipping " not in output.lower())


def run_law(binary: str, name: str, marker: str, destination: Path,
            budget: float = 90) -> dict:
    started = time.monotonic()
    text = []
    samples = []
    timed_out = False
    exit_code = None
    with destination.open("w") as log:
        proc = subprocess.Popen(
            [binary, name, "--exact", "--nocapture", "--test-threads=1"],
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
            env=dict(os.environ, AWL_CONVENTION_FORCE="mac"), start_new_session=True,
        )
        messages: queue.Queue[str] = queue.Queue()

        def read_output() -> None:
            assert proc.stdout is not None
            for line in proc.stdout:
                messages.put(line)

        reader = threading.Thread(target=read_output, daemon=True)
        reader.start()
        try:
            while proc.poll() is None:
                elapsed = time.monotonic() - started
                try:
                    rss = process_rss(proc.pid)
                    samples.append({"elapsed_seconds": round(elapsed, 3), "rss_kib": rss})
                    print(f"UPLOAD-RSS law={name} elapsed={elapsed:.3f}s rss_kib={rss}", flush=True)
                except (subprocess.SubprocessError, ValueError, OSError) as error:
                    log.write(f"RSS sampling error: {error}\n")
                while not messages.empty():
                    line = messages.get_nowait()
                    text.append(line)
                    log.write(line)
                    log.flush()
                    print(line, end="", flush=True)
                if elapsed >= budget:
                    timed_out = True
                    stop_group(proc)
                    break
                time.sleep(0.25)
            try:
                exit_code = proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                pass
        finally:
            stop_group(proc)
            reader.join(timeout=2)
            while not messages.empty():
                line = messages.get_nowait()
                text.append(line)
                log.write(line)
                print(line, end="", flush=True)
    output = "".join(text)
    positive = [s["rss_kib"] for s in samples if s["rss_kib"] > 0]
    result = {
        "law": name, "pid": proc.pid, "exit_code": exit_code,
        "timed_out": timed_out, "budget_seconds": budget,
        "elapsed_seconds": round(time.monotonic() - started, 3),
        "passed_with_gpu_measurement": passed(output, name, marker, exit_code, timed_out),
        "measurement_lines": [line for line in output.splitlines() if marker in line],
        "rss_scope": "test-process tree; excludes GPU driver/kernel memory",
        "rss_samples": samples, "sampled_peak_rss_kib": max(positive, default=None),
        "rss_measurement_available": bool(positive),
        "first_positive_rss_kib": positive[0] if positive else None,
        "last_positive_rss_kib": positive[-1] if positive else None,
    }
    destination.with_suffix(".json").write_text(json.dumps(result, indent=2) + "\n")
    print("UPLOAD-LAW RESULT " + json.dumps(result), flush=True)
    return result


def main() -> int:
    metadata, destination = map(Path, sys.argv[1:])
    destination.mkdir(parents=True, exist_ok=True)
    binary = test_binary(metadata)
    identity = validate_source(binary)
    results = []
    for name, marker in LAWS.items():
        results.append(run_law(binary, name, marker, destination / (name.rsplit("::", 1)[1] + ".log")))
        if results[-1]["exit_code"] is None:
            break
    destination.joinpath("upload-lifetime-summary.json").write_text(
        json.dumps({**identity, "laws": results}, indent=2) + "\n"
    )
    return 0 if len(results) == len(LAWS) and all(
        r["passed_with_gpu_measurement"] and r["rss_measurement_available"] for r in results
    ) else 1


if __name__ == "__main__":
    signal.signal(signal.SIGTERM, lambda _signal, _frame: sys.exit(143))
    sys.exit(main())
