#!/usr/bin/env python3
"""Hermetic producer/consumer laws for optional Mac completion receipts."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parent.parent
HELPER = ROOT / "scripts/macos-verification-receipt.py"


class Receipts(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="awl-mac-receipt-law-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.dist = self.root / "synthetic dist"
        self.dist.mkdir()
        self.receipts = self.root / "fresh receipts"
        self.receipts.mkdir()
        self.args = ["--receipt-dir", str(self.receipts), "--dist", str(self.dist),
                     "--version", "0.13.0", "--build-version", "42.0.0",
                     "--source", "a" * 40, "--signing", "signed", "--launch", "ci"]
        for arch in ("arm64", "x86_64"):
            for suffix in ("dmg", "app.zip"):
                name = f"awl-0.13.0-macos-{arch}.{suffix}"
                (self.dist / name).write_bytes(b"synthetic package bytes")
                self.checksum(name)

    def checksum(self, name):
        digest = hashlib.sha256((self.dist / name).read_bytes()).hexdigest()
        (self.dist / (name + ".sha256")).write_text(f"{digest}  {name}\n")

    def command(self, verb, extra=(), args=None):
        return subprocess.run([sys.executable, str(HELPER), verb,
                               *(self.args if args is None else args), *extra],
                              capture_output=True, text=True, timeout=10)

    def complete(self):
        for arch in ("arm64", "x86_64"):
            result = self.command("write", ["--arch", arch])
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_both_current_receipts_and_payload_hashes_pass(self):
        self.complete()
        result = self.command("check")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual({p.name for p in self.receipts.iterdir()}, {"arm64.json", "x86_64.json"})

    def test_stale_eight_file_payload_and_partial_completion_fail(self):
        self.assertEqual(len(list(self.dist.iterdir())), 8)
        self.assertNotEqual(self.command("check").returncode, 0)
        self.assertEqual(self.command("write", ["--arch", "arm64"]).returncode, 0)
        self.assertNotEqual(self.command("check").returncode, 0)
        self.assertNotEqual(self.command("write", ["--arch", "arm64"]).returncode, 0)

    def test_reused_directory_is_rejected_without_destroying_old_evidence(self):
        self.complete()
        before = {p.name: p.read_bytes() for p in self.receipts.iterdir()}
        result = self.command("init", args=["--receipt-dir", str(self.receipts)])
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(before, {p.name: p.read_bytes() for p in self.receipts.iterdir()})

    def test_every_metadata_axis_is_bound_to_current_consumer(self):
        self.complete()
        for flag, value in [("--source", "b" * 40), ("--version", "0.14.0"),
                            ("--build-version", "43.0.0"), ("--signing", "unsigned"),
                            ("--launch", "local")]:
            with self.subTest(flag=flag):
                changed = list(self.args)
                changed[changed.index(flag) + 1] = value
                self.assertNotEqual(self.command("check", args=changed).returncode, 0)
        path = self.receipts / "x86_64.json"
        data = json.loads(path.read_text())
        data["architecture"] = "arm64"
        path.write_text(json.dumps(data))
        self.assertNotEqual(self.command("check").returncode, 0)

    def test_replaced_payload_fails_even_with_updated_checksum(self):
        self.complete()
        name = "awl-0.13.0-macos-x86_64.app.zip"
        (self.dist / name).write_bytes(b"different synthetic package")
        self.checksum(name)
        self.assertNotEqual(self.command("check").returncode, 0)

    def test_missing_or_wrong_checksum_cannot_emit_completion(self):
        for missing in (False, True):
            with self.subTest(missing=missing):
                checksum = self.dist / "awl-0.13.0-macos-arm64.dmg.sha256"
                checksum.unlink(missing_ok=True)
                if not missing:
                    checksum.write_text("wrong checksum\n")
                self.assertNotEqual(self.command("write", ["--arch", "arm64"]).returncode, 0)
                self.assertEqual(list(self.receipts.iterdir()), [])

    def test_unknown_partial_malformed_and_symlink_receipts_fail(self):
        self.complete()
        extra = self.receipts / "unfinished.tmp"
        extra.write_text("partial")
        self.assertNotEqual(self.command("check").returncode, 0)
        extra.unlink()
        path = self.receipts / "x86_64.json"
        original = path.read_bytes()
        path.write_text("{")
        self.assertNotEqual(self.command("check").returncode, 0)
        path.unlink()
        elsewhere = self.root / "old.json"
        elsewhere.write_bytes(original)
        path.symlink_to(elsewhere)
        self.assertNotEqual(self.command("check").returncode, 0)


if __name__ == "__main__":
    unittest.main(verbosity=2)
