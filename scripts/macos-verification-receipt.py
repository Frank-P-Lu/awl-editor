#!/usr/bin/env python3
"""Bind completed Mac verification to one caller-owned directory and payload."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import tempfile

ARCHES = ("arm64", "x86_64")


def receipt_directory(path: Path) -> Path:
    if path.is_symlink() or not path.is_dir():
        raise ValueError("receipt directory must be an existing ordinary directory")
    return path


def sha256(path: Path) -> str:
    if path.is_symlink() or not path.is_file():
        raise ValueError(f"missing ordinary artifact: {path.name}")
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def record(args, arch: str) -> dict:
    artifacts = {}
    for suffix in ("dmg", "app.zip"):
        name = f"awl-{args.version}-macos-{arch}.{suffix}"
        if Path(name).name != name:
            raise ValueError("version must not introduce a path component")
        digest = sha256(args.dist / name)
        checksum = args.dist / (name + ".sha256")
        if checksum.is_symlink() or not checksum.is_file():
            raise ValueError(f"missing ordinary checksum: {checksum.name}")
        if checksum.read_text() != f"{digest}  {name}\n":
            raise ValueError(f"checksum mismatch: {name}")
        artifacts[name] = digest
    return {"schema": 1, "architecture": arch, "source_commit": args.source,
            "version": args.version, "build_version": args.build_version,
            "signing_policy": args.signing, "launch_context": args.launch,
            "artifacts": artifacts}


def write(args):
    directory = receipt_directory(args.receipt_dir)
    expected = set() if args.arch == "arm64" else {"arm64.json"}
    if {p.name for p in directory.iterdir()} != expected:
        raise ValueError("receipt directory is stale or architecture order is incomplete")
    completed = record(args, args.arch)
    # Publish a complete JSON document only after all artifact checks succeed.
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(mode="w", dir=directory, delete=False) as out:
            temporary = Path(out.name)
            json.dump(completed, out, sort_keys=True)
            out.write("\n")
            out.flush()
            os.fsync(out.fileno())
        os.replace(temporary, directory / f"{args.arch}.json")
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def check(args):
    directory = receipt_directory(args.receipt_dir)
    if {p.name for p in directory.iterdir()} != {f"{arch}.json" for arch in ARCHES}:
        raise ValueError("both current architecture receipts are required")
    for arch in ARCHES:
        receipt = directory / f"{arch}.json"
        if receipt.is_symlink() or not receipt.is_file():
            raise ValueError("receipt must be an ordinary file")
        if json.loads(receipt.read_text()) != record(args, arch):
            raise ValueError(f"receipt metadata or artifact mismatch: {arch}")
    print("macos-verification: both current architecture receipts and hashes passed")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    init = sub.add_parser("init")
    init.add_argument("--receipt-dir", required=True, type=Path)
    for command in ("write", "check"):
        entry = sub.add_parser(command)
        entry.add_argument("--receipt-dir", required=True, type=Path)
        entry.add_argument("--dist", required=True, type=Path)
        entry.add_argument("--version", required=True)
        entry.add_argument("--build-version", required=True)
        entry.add_argument("--source", required=True)
        entry.add_argument("--signing", choices=("signed", "unsigned"), required=True)
        entry.add_argument("--launch", choices=("ci", "local"), required=True)
        if command == "write":
            entry.add_argument("--arch", choices=ARCHES, required=True)
    args = parser.parse_args()
    try:
        if args.command == "init":
            if any(receipt_directory(args.receipt_dir).iterdir()):
                raise ValueError("receipt directory must be fresh and empty")
        elif args.command == "write":
            write(args)
        else:
            check(args)
    except (OSError, ValueError) as error:
        parser.exit(1, f"macos-verification: {error}\n")


if __name__ == "__main__":
    main()
