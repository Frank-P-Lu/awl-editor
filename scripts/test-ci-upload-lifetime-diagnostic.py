#!/usr/bin/env python3
"""The diagnostic must reject skipped, unmatched, failed and unbounded runs."""
import importlib.util
import hashlib
import json
import subprocess
import sys
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

sys.dont_write_bytecode = True

spec = importlib.util.spec_from_file_location(
    "diagnostic", Path(__file__).with_name("ci-upload-lifetime-diagnostic.py"))
diag = importlib.util.module_from_spec(spec)
spec.loader.exec_module(diag)
NAME, MARKER = next(iter(diag.LAWS.items()))


class Diagnostic(unittest.TestCase):
    def test_metadata_selects_only_real_library_test_executable(self):
        with tempfile.TemporaryDirectory() as temp:
            p = Path(temp) / "build.jsonl"
            p.write_text(json.dumps({"reason": "compiler-artifact", "profile": {"test": True},
                                    "target": {"kind": ["lib"]}, "executable": "/test"}) + "\n")
            self.assertEqual(diag.test_binary(p), "/test")
            p.write_text(p.read_text().replace('"lib"', '"bin"'))
            with self.assertRaises(RuntimeError):
                diag.test_binary(p)

    def test_missing_gpu_or_exact_test_never_passes(self):
        good = f"test {NAME} ... {MARKER} batches=[32,64] drained=0\n1 passed; 0 failed; 0 ignored"
        self.assertTrue(diag.passed(good, NAME, MARKER, 0, False))
        for output in [good.replace(MARKER, ""), good.replace(NAME, "other"),
                       good.replace("1 passed", "0 passed"), good + "\nskipping no adapter"]:
            self.assertFalse(diag.passed(output, NAME, MARKER, 0, False))
        self.assertFalse(diag.passed(good, NAME, MARKER, 1, False))
        self.assertFalse(diag.passed(good, NAME, MARKER, 0, True))

    def test_real_subprocess_output_and_rss_are_retained(self):
        with tempfile.TemporaryDirectory() as temp:
            folder = Path(temp)
            binary = folder / "fixture"
            binary.write_text(f'#!/usr/bin/env python3\nimport time\nprint({"test " + NAME + " ... " + MARKER!r}, flush=True)\ntime.sleep(0.6)\nprint("1 passed; 0 failed; 0 ignored", flush=True)\n')
            binary.chmod(0o755)
            result = diag.run_law(str(binary), NAME, MARKER, folder / "law.log", budget=3)
            self.assertTrue(result["passed_with_gpu_measurement"])
            self.assertGreater(result["sampled_peak_rss_kib"], 0)
            self.assertIn(MARKER, (folder / "law.log").read_text())
            self.assertTrue((folder / "law.json").exists())

    def test_timeout_kills_process_group_and_is_failure(self):
        with tempfile.TemporaryDirectory() as temp:
            folder = Path(temp)
            binary = folder / "fixture"
            binary.write_text('#!/usr/bin/env python3\nimport time\ntime.sleep(30)\n')
            binary.chmod(0o755)
            result = diag.run_law(str(binary), NAME, MARKER, folder / "law.log", budget=0.15)
            self.assertTrue(result["timed_out"])
            self.assertLess(result["elapsed_seconds"], 3)
            self.assertEqual(result["exit_code"], -9)
            self.assertFalse(result["passed_with_gpu_measurement"])

    def test_source_sha_and_executable_containment_are_enforced(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            binary = root / "target/debug/deps/law"
            binary.parent.mkdir(parents=True)
            binary.write_bytes(b"fixture executable")
            with patch.object(diag.subprocess, "check_output", return_value=diag.EXPECTED_SOURCE):
                identity = diag.validate_source(str(binary), root)
                self.assertEqual(identity["executable_sha256"], hashlib.sha256(binary.read_bytes()).hexdigest())
                with self.assertRaises(RuntimeError):
                    diag.validate_source(str(root / "outside"), root)
            with patch.object(diag.subprocess, "check_output", return_value="wrong-source"):
                with self.assertRaises(RuntimeError):
                    diag.validate_source(str(binary), root)

    def test_interruption_reaps_the_started_process(self):
        with tempfile.TemporaryDirectory() as temp:
            folder = Path(temp)
            binary = folder / "fixture"
            binary.write_text('#!/usr/bin/env python3\nimport time\ntime.sleep(30)\n')
            binary.chmod(0o755)
            created = []
            real_popen = subprocess.Popen

            def start(*args, **kwargs):
                proc = real_popen(*args, **kwargs)
                created.append(proc)
                return proc

            with patch.object(diag.subprocess, "Popen", side_effect=start), patch.object(
                    diag, "process_rss", side_effect=KeyboardInterrupt):
                with self.assertRaises(KeyboardInterrupt):
                    diag.run_law(str(binary), NAME, MARKER, folder / "law.log")
            self.assertEqual(created[0].poll(), -9)

    def test_timeout_kills_descendants_too(self):
        with tempfile.TemporaryDirectory() as temp:
            folder = Path(temp)
            binary = folder / "fixture"
            child_pid = folder / "child-pid"
            binary.write_text(f'#!/usr/bin/env python3\nimport subprocess,time\np=subprocess.Popen(["python3","-c","import time;time.sleep(30)"])\nopen({str(child_pid)!r},"w").write(str(p.pid))\ntime.sleep(30)\n')
            binary.chmod(0o755)
            result = diag.run_law(str(binary), NAME, MARKER, folder / "law.log", budget=0.6)
            self.assertTrue(result["timed_out"])
            state = subprocess.run(["ps", "-o", "stat=", "-p", child_pid.read_text()],
                                   capture_output=True, text=True)
            self.assertTrue(not state.stdout.strip() or state.stdout.strip().startswith("Z"))

    def test_manual_mode_preserves_regular_job_defaults_and_concurrency(self):
        workflow = Path(__file__).resolve().parent.parent / ".github/workflows/ci.yml"
        text = workflow.read_text()
        self.assertIn("type: boolean\n        default: false", text)
        self.assertEqual(text.count("if: ${{ !inputs.upload_lifetime_diagnostic }}"), 5)
        self.assertIn("github.event_name != 'pull_request' && !inputs.upload_lifetime_diagnostic", text)
        self.assertIn("if: github.event_name == 'workflow_dispatch' && inputs.upload_lifetime_diagnostic", text)
        self.assertIn("inputs.upload_lifetime_diagnostic && '-upload-lifetime' || ''", text)
        self.assertIn("ref: " + diag.EXPECTED_SOURCE, text)

    def test_rss_includes_descendants_and_excludes_other_processes(self):
        with patch.object(diag.subprocess, "check_output", return_value="1 0 100\n2 1 200\n3 2 300\n4 0 900\n"):
            self.assertEqual(diag.process_rss(1), 600)


if __name__ == "__main__":
    unittest.main()
