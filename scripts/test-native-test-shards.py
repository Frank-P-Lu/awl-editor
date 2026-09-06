#!/usr/bin/env python3
"""Focused laws for native-test-shards.py."""

from __future__ import annotations

import importlib.util
import json
import pathlib
import sys
import tempfile
import unittest

# A by-path load writes scripts/__pycache__ next to the LOADED file, so the
# guard belongs here rather than in whatever invokes this script: the
# consumers are hand-run instruments and Rust tests, not one wrapper that
# could carry PYTHONDONTWRITEBYTECODE for all of them.
sys.dont_write_bytecode = True

SCRIPT = pathlib.Path(__file__).with_name("native-test-shards.py")
SPEC = importlib.util.spec_from_file_location("native_test_shards", SCRIPT)
assert SPEC and SPEC.loader
SHARDS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SHARDS)


class NativeTestShardsTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.temp.name)

    def tearDown(self) -> None:
        self.temp.cleanup()

    def write_list(self, name: str, tests: list[str]) -> pathlib.Path:
        path = self.root / name
        path.write_text("".join(f"{test}: test\n" for test in tests))
        return path

    def test_artifacts_selects_test_bin_and_every_integration(self) -> None:
        events = [
            {"reason": "compiler-artifact", "target": {"kind": ["lib"], "name": "awl"}, "profile": {"test": False}, "executable": "/tmp/lib"},
            {"reason": "compiler-artifact", "target": {"kind": ["bin"], "name": "awl"}, "profile": {"test": True}, "executable": "/tmp/awl-tests"},
            {"reason": "compiler-artifact", "target": {"kind": ["test"], "name": "alpha"}, "profile": {"test": True}, "executable": "/tmp/alpha"},
            {"reason": "compiler-artifact", "target": {"kind": ["test"], "name": "beta"}, "profile": {"test": True}, "executable": "/tmp/beta"},
        ]
        source = self.root / "cargo.json"
        source.write_text("not-json\n" + "\n".join(json.dumps(event) for event in events))
        output = self.root / "artifacts"
        SHARDS.artifacts(source, output)
        self.assertEqual(output.read_text(), "binary=/tmp/awl-tests\nintegration=alpha\nintegration=beta\n")

    def test_artifacts_refuses_missing_or_malformed_subjects(self) -> None:
        for content, message in (("garbage\n", "no awl binary"), (json.dumps({"reason": "compiler-artifact", "target": {"kind": ["bin"], "name": "awl"}, "profile": {"test": True}, "executable": "/tmp/awl"}), "no integration-test")):
            source = self.root / "bad.json"
            source.write_text(content)
            with self.assertRaisesRegex(SystemExit, message):
                SHARDS.artifacts(source, self.root / "out")

    def test_verify_accepts_only_an_exact_complete_partition(self) -> None:
        full = self.write_list("full", ["a::one", "b::two", "c::three"])
        one = self.write_list("one", ["a::one", "c::three"])
        two = self.write_list("two", ["b::two"])
        SHARDS.verify(full, [one, two])

    def test_verify_names_missing_duplicate_and_foreign_tests(self) -> None:
        full = self.write_list("full", ["a::one", "b::two"])
        cases = (
            ([self.write_list("missing", ["a::one"])], "missing=b::two"),
            ([self.write_list("dup1", ["a::one"]), self.write_list("dup2", ["a::one", "b::two"])], "duplicate=a::one"),
            ([self.write_list("foreign", ["a::one", "b::two", "z::alien"])], "foreign=z::alien"),
        )
        for listed, message in cases:
            with self.assertRaisesRegex(SystemExit, message):
                SHARDS.verify(full, listed)

    def test_substring_collisions_are_removed_by_live_skip_prefixes(self) -> None:
        groups = [{"run::"}, {"firstrun::", "render::tests::markdown::"}, {"markdown::"}]
        skips = [SHARDS.collision_skips(groups, index) for index in range(len(groups))]
        self.assertIn("firstrun::", skips[0])
        self.assertIn("render::tests::markdown::", skips[2])


if __name__ == "__main__":
    unittest.main()
