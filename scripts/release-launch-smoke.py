#!/usr/bin/env python3
"""Launch the actual packaged binary with isolated fixtures and its live GPU contract."""
from __future__ import annotations

import argparse
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile


def launch(binary: Path, seconds: int, timeout: float, appimage: bool) -> int:
    if not binary.is_file() or not os.access(binary, os.X_OK):
        raise ValueError("the built artifact must be an executable file")
    with tempfile.TemporaryDirectory(prefix="awl-release-launch-") as folder:
        root = Path(folder)
        env = dict(os.environ)
        for key, child in [
            ("XDG_CONFIG_HOME", "config"), ("XDG_DATA_HOME", "data"),
            ("XDG_CACHE_HOME", "cache"),
        ]:
            directory = root / child
            directory.mkdir()
            env[key] = str(directory)
        command = [str(binary)]
        if appimage:
            command.append("--appimage-extract-and-run")
        # SoakGpu owns its synthetic state and rejects file/config/folder flags.
        command.extend(["--soak-gpu", "--soak-gpu-seconds", str(seconds)])
        process = subprocess.Popen(
            command, cwd=root, env=env, start_new_session=True,
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
        )
        try:
            output, _ = process.communicate(timeout=timeout)
        except subprocess.TimeoutExpired:
            os.killpg(process.pid, signal.SIGTERM)
            try:
                output, _ = process.communicate(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                output, _ = process.communicate()
            print(output, end="")
            print("release-launch: timed out; owned process group retired", file=sys.stderr)
            return 1
        print(output, end="")
        if process.returncode:
            print(f"release-launch: built artifact failed with status {process.returncode}", file=sys.stderr)
            return 1
        # The app's Report::passed owns presentation, recovery and memory verdicts.
        print("release-launch: actual built artifact passed its live GPU contract")
        return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--seconds", type=int, default=25)
    parser.add_argument("--timeout", type=float)
    parser.add_argument("--appimage", action="store_true")
    parser.add_argument("--local", action="store_true", help="explicit local release validation with the same isolated probe")
    args = parser.parse_args()
    if args.seconds < 25 or (args.timeout is not None and args.timeout <= 0):
        parser.error("seconds must be at least 25 and timeout must be positive")
    if not os.environ.get("CI") and not args.local:
        parser.error("use CI or explicitly select --local release validation")
    try:
        return launch(args.binary.absolute(), args.seconds, args.timeout or (2 * args.seconds + 60), args.appimage)
    except (OSError, ValueError) as error:
        print(f"release-launch: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
